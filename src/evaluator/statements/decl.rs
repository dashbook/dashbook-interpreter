use crate::{environment::Environments};
use crate::evaluator::*;

use futures::future::{FutureExt, LocalBoxFuture};

use js_sys::Error;

#[inline]
pub(crate) async fn eval_decl(decl: Decl, envs: &mut Environments) -> Result<RcValue, Error> {
    match decl {
        Decl::Var(decl) => match decl.kind {
            _ => {
                let mut result = Err(Error::from(js_sys::ReferenceError::new(
                    "ReferenceError: No Variables in declaration.",
                )));
                for var in decl.decls {
                    let obj = match var.init {
                        Some(expr) => expressions::eval_expr(*expr, envs).await?,
                        None => Value::Undefined(JsValue::undefined()).into(),
                    };
                    result = set_pat(var.name, obj, envs, DeclOrAssign::Decl).await;
                }
                Ok(result?.into())
            }
        },
        Decl::Fn(decl) => {
            let body = match decl.function.body {
                Some(body) => Ok(body.stmts),
                None => Err(Error::from(js_sys::TypeError::new(&format!(
                    "TypeError: Expected function body, got {:?}.",
                    decl.function.body
                )))),
            }?;
            let function = functions::function_declaration(
                decl.function.params.into_iter().map(|x| x.pat).collect(),
                body,
                decl.function.is_async,
                envs,
                None,
            )
            .await
            .map(|x| x.into())?;
            envs.insert(&decl.ident.sym, function)?;
            Ok(Value::Undefined(JsValue::undefined()).into())
        }
        Decl::Class(classdecl) => {
            let ident = classdecl.ident;
            let class = class::eval_class(*classdecl.class, envs).await?;
            envs.insert(&ident.sym, Value::JsFunction(class).into())?;
            Ok(Value::Undefined(JsValue::undefined()).into())
        }
        _ => Err(Error::new(&format!(
            "ERROR: Declaration {:?} is not supported.",
            decl
        ))),
    }
}

#[derive(Clone, Copy)]
pub(crate) enum DeclOrAssign {
    Decl,
    Assign,
}

pub(crate) fn set_pat<'a>(
    pat: Pat,
    rhs: RcValue,
    envs: &'a mut Environments,
    variant: DeclOrAssign,
) -> LocalBoxFuture<'a, Result<RcValue, Error>> {
    async move {
        match pat {
            Pat::Ident(a) => {
                match variant {
                    DeclOrAssign::Decl => {
                        envs.insert(&a.id.sym, rhs)?;
                        Ok(Value::Undefined(JsValue::undefined()).into())
                    },
                    DeclOrAssign::Assign => {
                        envs.set(&a.id.sym, rhs.clone())?;
                        Ok(rhs)
                    }
                }
                
            }
            Pat::Assign(assign) => {
                if rhs.borrow().as_ref().is_undefined() {
                    let value = expressions::eval_expr(*assign.right, envs).await?;
                    set_pat(*assign.left, value, envs, variant).await
                } else {
                    set_pat(*assign.left, rhs, envs, variant).await
                }
            }
            Pat::Array(patterns) => {
                if js_sys::Array::is_array(rhs.borrow().as_ref()) {
                    let values = js_sys::Array::from(rhs.borrow().as_ref());
                    let mut values_iter = values.iter();

                    let len_patterns = patterns.elems.len();

                    for opt in patterns.elems {
                        let value: RcValue = match values_iter.next() {
                            Some(x) => Value::from(x).into(),
                            None => Value::Undefined(JsValue::undefined()).into(),
                        };
                        match opt {
                            Some(el) => match el {
                                Pat::Rest(rest) => {
                                    let rest_values = values.slice(
                                        (len_patterns - 1).try_into().unwrap(),
                                        values.length(),
                                    );
                                    set_pat(
                                        *rest.arg,
                                        Value::Object(JsObject::from(rest_values)).into(),
                                        envs,variant
                                    )
                                    .await?;
                                }
                                _ => {
                                    set_pat(el, value, envs, variant.clone()).await?;
                                }
                            },
                            None => (),
                        }
                    }
                    match variant {
                        DeclOrAssign::Decl => {
                            Ok(Value::Undefined(JsValue::undefined()).into())
                        },
                        DeclOrAssign::Assign => {
                            Ok(rhs)
                        }
                    }
                } else {
                    Err(Error::new(&format!(
                        "ERROR: Expression {:?} on right-hand-side of declaration is not an array.",
                        rhs
                    )))
                }
            }
            Pat::Object(pattern) => match &*rhs.clone().borrow() {
                Value::Object(obj) => {
                    let obj: RcValue =
                        Value::Object(JsObject::assign(&JsObject::new(), obj)).into();
                    for pat in pattern.props {
                        match pat {
                            ObjectPatProp::Assign(prop) => match prop.value {
                                Some(x) => {
                                    let value = expressions::eval_expr(*x, envs).await?;
                                    match variant {
                                        DeclOrAssign::Decl => {
                                            envs.insert(&prop.key.sym, value)?;
                                        },
                                        DeclOrAssign::Assign => {
                                            envs.set(&prop.key.sym, value)?;
                                        }
                                    }
                                }
                                None => {
                                    let value = js_sys::Reflect::get(
                                        obj.borrow().as_ref(),
                                        &JsValue::from_str(&prop.key.sym),
                                    )?;
                                    js_sys::Reflect::delete_property(
                                        match &*obj.borrow() {
                                            Value::Object(x) => Ok(x),
                                            _ => Err(Error::new(&format!(
                                                "ERROR: Expression {:?} on right-hand-side of declaration is not an object.",
                                                rhs
                                            )))
                                        }?,
                                        &JsValue::from_str(&prop.key.sym),
                                    )?;
                                    match variant {
                                        DeclOrAssign::Decl => {
                                            envs.insert(&prop.key.sym, Value::from(value).into())?;
                                        },
                                        DeclOrAssign::Assign => {
                                            envs.set(&prop.key.sym, Value::from(value).into())?;
                                        }
                                    }
                                }
                            },
                            ObjectPatProp::KeyValue(kv) => {
                                let key = objects::get_prop_name(kv.key, envs).await?;
                                let value = js_sys::Reflect::get(
                                    obj.borrow().as_ref(),
                                    key.borrow().as_ref(),
                                )?;
                                js_sys::Reflect::delete_property(
                                    match &*obj.borrow() {
                                        Value::Object(x) => Ok(x),
                                        _ => Err(Error::new(&format!(
                                            "ERROR: Expression {:?} on right-hand-side of declaration is not an object.",
                                            rhs
                                        )))
                                    }?,
                                    key.borrow().as_ref(),
                                )?;
                                set_pat(*kv.value, Value::from(value).into(), envs, variant).await?;
                            }
                            ObjectPatProp::Rest(rest) => {
                                set_pat(*rest.arg, obj.clone(), envs, variant.clone()).await?;
                            }
                        }
                    }
                    match variant {
                        DeclOrAssign::Decl => {
                            Ok(Value::Undefined(JsValue::undefined()).into())
                        },
                        DeclOrAssign::Assign => {
                            Ok(rhs)
                        }
                    }
                }
                _ => Err(Error::new(&format!(
                    "ERROR: Expression {:?} on right-hand-side of declaration is not an object.",
                    rhs
                ))),
            },
            Pat::Expr(expr) => {
                match variant {
                    DeclOrAssign::Assign => match *expr {
                        Expr::Ident(ident) => {
                            match variant {
                                DeclOrAssign::Decl => {
                                    envs.insert(&ident.sym, rhs)?;
                                    Ok(Value::Undefined(JsValue::undefined()).into())
                                },
                                DeclOrAssign::Assign => {
                                    envs.set(&ident.sym, rhs.clone())?;
                                    Ok(rhs)
                                }
                            }
                        }
                        Expr::Member(memexpr) => {
                            objects::assign_member_expr(memexpr, rhs, envs).await
                        }
                        _ => Err(Error::new(&format!(
                            "ERROR: Expression {:?} is not supported as a pattern.",
                            expr
                        ))),
                    },
                    DeclOrAssign::Decl => Err(Error::new(&format!(
                        "ERROR: Expression {:?} is not supported as a pattern in declaration.",
                        expr
                    )))
                }
                
            }
            _ => Err(Error::new(&format!(
                "ERROR: Pattern {:?} is not supported.",
                pat
            ))),
        }
    }
    .boxed_local()
}
