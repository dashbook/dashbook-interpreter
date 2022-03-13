use crate::environment::Environments;
use crate::evaluator;
use crate::evaluator::functions;
use crate::value::*;

use futures::stream::{self, StreamExt};
use js_sys::{Error, Reflect};
use swc_ecma_ast::*;
use wasm_bindgen::prelude::*;

use super::expressions::eval_expr;

#[inline]
pub async fn eval_obj_lit_expr(
    objlit: ObjectLit,
    envs: &mut Environments,
) -> Result<RcValue, Error> {
    Ok(Value::Object(
        stream::iter(objlit.props)
            .fold(
                Ok::<(JsObject, &mut Environments), Error>((JsObject::new(), envs)),
                |acc, x| async move {
                    let (obj, envs) = acc?;
                    let new = match x {
                        PropOrSpread::Prop(prop) => match *prop {
                            Prop::Shorthand(ident) => Ok(js_sys::Object::define_property(
                                &obj,
                                &JsValue::from_str(&ident.sym),
                                &create_object_from_entries(vec![
                                    (JsValue::from_str("configurable"), &JsValue::from_bool(true)),
                                    (JsValue::from_str("enumerable"), &JsValue::from_bool(true)),
                                    (JsValue::from_str("writable"), &JsValue::from_bool(true)),
                                    (
                                        JsValue::from_str("value"),
                                        envs.get(&ident.sym)?.borrow().as_ref(),
                                    ),
                                ])?,
                            )),
                            Prop::KeyValue(kv) => match eval_expr(*kv.value, envs).await {
                                Ok(prop) => Ok(js_sys::Object::define_property(
                                    &obj,
                                    get_prop_name(kv.key, envs).await?.borrow().as_ref(),
                                    &create_object_from_entries(vec![
                                        (
                                            JsValue::from_str("configurable"),
                                            &JsValue::from_bool(true),
                                        ),
                                        (
                                            JsValue::from_str("enumerable"),
                                            &JsValue::from_bool(true),
                                        ),
                                        (JsValue::from_str("writable"), &JsValue::from_bool(true)),
                                        (JsValue::from_str("value"), prop.borrow().as_ref()),
                                    ])?,
                                )),
                                Err(err) => Err(err),
                            },
                            Prop::Assign(prop_assign) => match eval_expr(*prop_assign.value, envs)
                                .await
                            {
                                Ok(prop) => Ok(js_sys::Object::define_property(
                                    &obj,
                                    &JsValue::from_str(&prop_assign.key.sym),
                                    &create_object_from_entries(vec![
                                        (
                                            JsValue::from_str("configurable"),
                                            &JsValue::from_bool(true),
                                        ),
                                        (
                                            JsValue::from_str("enumerable"),
                                            &JsValue::from_bool(true),
                                        ),
                                        (JsValue::from_str("writable"), &JsValue::from_bool(true)),
                                        (JsValue::from_str("value"), prop.borrow().as_ref()),
                                    ])?,
                                )),
                                Err(err) => Err(err),
                            },
                            Prop::Method(method_prop) => {
                                let body = method_prop
                                    .function
                                    .body
                                    .ok_or(Error::new(&format!(
                                        "ERROR: Function body of method property invalid."
                                    )))?
                                    .stmts;
                                let function: RcValue = functions::function_declaration(
                                    method_prop
                                        .function
                                        .params
                                        .into_iter()
                                        .map(|x| x.pat)
                                        .collect(),
                                    body,
                                    method_prop.function.is_async,
                                    envs,
                                    None,
                                )
                                .await
                                .map(|x| x.into())?;
                                let new = js_sys::Object::define_property(
                                    &obj,
                                    get_prop_name(method_prop.key, envs)
                                        .await?
                                        .borrow()
                                        .as_ref(),
                                    &create_object_from_entries(vec![
                                        (
                                            JsValue::from_str("configurable"),
                                            &JsValue::from_bool(true),
                                        ),
                                        (
                                            JsValue::from_str("enumerable"),
                                            &JsValue::from_bool(true),
                                        ),
                                        (JsValue::from_str("writable"), &JsValue::from_bool(true)),
                                        (JsValue::from_str("value"), function.borrow().as_ref()),
                                    ])?,
                                );
                                Ok(new)
                            }
                            Prop::Getter(getter_prop) => {
                                let body = getter_prop
                                    .body
                                    .ok_or(Error::new(&format!(
                                        "ERROR: Function body of method property invalid."
                                    )))?
                                    .stmts;
                                let function: RcValue = functions::function_declaration(
                                    vec![],
                                    body,
                                    false,
                                    envs,
                                    None,
                                )
                                .await
                                .map(|x| x.into())?;
                                let new = js_sys::Object::define_property(
                                    &obj,
                                    get_prop_name(getter_prop.key, envs)
                                        .await?
                                        .borrow()
                                        .as_ref(),
                                    &create_object_from_entries(vec![
                                        (
                                            JsValue::from_str("configurable"),
                                            &JsValue::from_bool(true),
                                        ),
                                        (
                                            JsValue::from_str("enumerable"),
                                            &JsValue::from_bool(true),
                                        ),
                                        (JsValue::from_str("get"), function.borrow().as_ref()),
                                    ])?,
                                );
                                Ok(new)
                            }
                            Prop::Setter(setter_prop) => {
                                let body = setter_prop
                                    .body
                                    .ok_or(Error::new(&format!(
                                        "ERROR: Function body of method property invalid."
                                    )))?
                                    .stmts;
                                let function: RcValue = functions::function_declaration(
                                    vec![setter_prop.param],
                                    body,
                                    false,
                                    envs,
                                    None,
                                )
                                .await
                                .map(|x| x.into())?;
                                let new = js_sys::Object::define_property(
                                    &obj,
                                    get_prop_name(setter_prop.key, envs)
                                        .await?
                                        .borrow()
                                        .as_ref(),
                                    &create_object_from_entries(vec![
                                        (
                                            JsValue::from_str("configurable"),
                                            &JsValue::from_bool(true),
                                        ),
                                        (
                                            JsValue::from_str("enumerable"),
                                            &JsValue::from_bool(true),
                                        ),
                                        (JsValue::from_str("set"), function.borrow().as_ref()),
                                    ])?,
                                );
                                Ok(new)
                            }
                        },
                        PropOrSpread::Spread(spread) => {
                            match &*evaluator::expressions::eval_expr(*spread.expr, envs)
                                .await?
                                .borrow()
                            {
                                Value::Object(spread) => js_sys::Object::entries(&spread)
                                    .iter()
                                    .into_iter()
                                    .fold(Ok(obj), |acc, x| {
                                        let arr = js_sys::Array::from(&x);
                                        Ok(js_sys::Object::define_property(
                                            &acc?,
                                            &arr.get(0),
                                            &create_object_from_entries(vec![
                                                (
                                                    JsValue::from_str("configurable"),
                                                    &JsValue::from_bool(true),
                                                ),
                                                (
                                                    JsValue::from_str("enumerable"),
                                                    &JsValue::from_bool(true),
                                                ),
                                                (
                                                    JsValue::from_str("writable"),
                                                    &JsValue::from_bool(true),
                                                ),
                                                (JsValue::from_str("value"), &arr.get(1)),
                                            ])?,
                                        ))
                                    }),
                                y => Err(Error::new(&format!(
                                    "ERROR: Spread operator not supported for {}.",
                                    y
                                ))),
                            }
                        }
                    }?;
                    Ok((new, envs))
                },
            )
            .await?
            .0,
    )
    .into())
}

pub(crate) async fn get_prop_name(
    prop_name: PropName,
    envs: &mut Environments,
) -> Result<RcValue, Error> {
    match prop_name {
        PropName::Ident(ident) => Ok(Value::String(JsString::from(ident.sym.to_string())).into()),
        PropName::Str(str) => Ok(Value::String(JsString::from(str.value.to_string())).into()),
        PropName::Num(number) => Ok(Value::Number(JsNumber::from(number.value)).into()),
        PropName::Computed(computed) => eval_expr(*computed.expr, envs).await,
        PropName::BigInt(_) => Err(Error::new("BigInt not supported as property name.")),
    }
}

#[inline]
pub async fn eval_member_expr(
    memexpr: MemberExpr,
    envs: &mut Environments,
) -> Result<RcValue, Error> {
    match &*evaluator::expressions::eval_expr(*memexpr.obj.clone(), envs)
        .await?
        .borrow()
    {
        Value::Object(obj) => match memexpr.prop {
            MemberProp::Ident(ident) => {
                match Reflect::get(obj.as_ref(), &JsValue::from(ident.sym.to_string()))
                    .map_err(|x| Error::from(x))
                {
                    Ok(js) => Ok(Value::from(js).into()),
                    Err(err) => Err(err),
                }
            }
            MemberProp::Computed(computed) => {
                let prop = evaluator::expressions::eval_expr(*computed.expr, envs).await?;
                let prop = prop.borrow();
                match Reflect::get(obj.as_ref(), prop.as_ref()).map_err(|x| Error::from(x)) {
                    Ok(js) => Ok(Value::from(js).into()),
                    Err(err) => Err(err),
                }
            }
            _ => Err(Error::new(&format!(
                "ERROR: Private member expression could not be evaluated."
            ))),
        },
        Value::JsFunction(obj) => match memexpr.prop {
            MemberProp::Ident(ident) => {
                match Reflect::get(obj.as_ref(), &JsValue::from(ident.sym.to_string()))
                    .map_err(|x| Error::from(x))
                {
                    Ok(js) => Ok(Value::from(js).into()),
                    Err(err) => Err(err),
                }
            }
            MemberProp::Computed(computed) => {
                let prop = evaluator::expressions::eval_expr(*computed.expr, envs).await?;
                let prop = prop.borrow();
                match Reflect::get(obj.as_ref(), prop.as_ref()).map_err(|x| Error::from(x)) {
                    Ok(js) => Ok(Value::from(js).into()),
                    Err(err) => Err(err),
                }
            }
            _ => Err(Error::new(&format!(
                "ERROR: Private member expression could not be evaluated."
            ))),
        },
        Value::Undefined(_) => match *memexpr.obj {
            Expr::Ident(ident) => Err(js_sys::ReferenceError::new(&format!(
                "ERROR: {} is not defined.",
                ident.sym
            ))
            .into()),
            _ => Err(js_sys::ReferenceError::new(&format!(
                "ERROR: {:?} is undefined.",
                memexpr.obj
            ))
            .into()),
        },
        _ => Err(
            js_sys::TypeError::new(&format!("ERROR: {:?} is not an object.", memexpr.obj)).into(),
        ),
    }
}

#[inline]
pub async fn assign_member_expr(
    memexpr: MemberExpr,
    rhsexpr: RcValue,
    envs: &mut Environments,
) -> Result<RcValue, Error> {
    let obj = evaluator::expressions::eval_expr(*memexpr.obj, envs).await?;
    match memexpr.prop {
        MemberProp::Ident(ident) => {
            Reflect::set(
                &JsValue::from(obj.borrow().as_ref()),
                &JsValue::from(ident.sym.to_string()),
                (rhsexpr.borrow()).as_ref(),
            )
            .map_err(|x| Error::from(x))?;
            Ok(rhsexpr)
        }
        MemberProp::Computed(computed) => {
            let prop = evaluator::expressions::eval_expr(*computed.expr, envs).await?;
            let prop = prop.borrow();
            Reflect::set(
                &JsValue::from(obj.borrow().as_ref()),
                &prop.as_ref(),
                (rhsexpr.borrow()).as_ref(),
            )
            .map_err(|x| Error::from(x))?;
            Ok(rhsexpr)
        }
        _ => Err(Error::new(&format!(
            "ERROR: Private member expression could not be evaluated."
        ))),
    }
}

#[inline]
pub async fn eval_new_expr(newexpr: NewExpr, envs: &mut Environments) -> Result<RcValue, Error> {
    let function = evaluator::expressions::eval_expr(*newexpr.callee, envs).await?;
    let (args, _envs) = match newexpr.args {
        Some(input) => {
            let len = input.len();
            stream::iter(input)
                .fold(Ok((Vec::with_capacity(len), envs)), |acc, x| async move {
                    let (mut vec, envs) = acc?;
                    vec.push(evaluator::expressions::eval_expr(*x.expr, envs).await?);
                    Ok::<(Vec<_>, &mut Environments), Error>((vec, envs))
                })
                .await?
        }
        None => (Vec::new(), envs),
    };
    let result = match &*function.borrow() {
        Value::Function(func) => {
            let this: RcValue = Value::Object(match &func.prototype {
                Some(proto) => proto.clone(),
                None => JsObject::new(),
            })
            .into();
            if args.len() != func.args.len() {
                Err(Error::new(&format!(
                    "ERROR: Functions requires {:} arguments, {:} where given.",
                    func.args.len(),
                    args.len()
                )))
            } else {
                let mut func_env = Environments::from_closed_env(func.env.clone());
                func_env.push_env();
                args.into_iter().zip(func.args.iter()).for_each(|(x, y)| {
                    let _ = func_env.insert(y, x);
                });
                match &(*this.clone().borrow()) {
                    Value::Object(_) => {
                        func_env.insert("this", this)?;
                    }
                    _ => (),
                }
                let result = if func.async_ {
                    evaluator::eval(func.body.clone(), &mut func_env)
                        .await
                        .map(|x| {
                            Value::from(JsValue::from(js_sys::Object::from(
                                js_sys::Promise::resolve(x.borrow().as_ref()),
                            )))
                            .into()
                        })
                } else {
                    evaluator::eval(func.body.clone(), &mut func_env).await
                };
                func_env.pop_env();
                result
            }
        }
        Value::JsFunction(func) => Ok(Value::from(Reflect::construct(
            &func,
            &args
                .iter()
                .map(|x| JsValue::from(&x.borrow() as &Value))
                .collect::<js_sys::Array>(),
        )?)
        .into()),
        _ => Err(Error::from(js_sys::TypeError::new(&format!(
            "TypeError: {} is not a function.",
            function.borrow()
        )))),
    };
    result
}

pub fn create_object_from_entries(entries: Vec<(JsValue, &JsValue)>) -> Result<JsObject, Error> {
    let obj = js_sys::Object::new();
    entries
        .into_iter()
        .map(|x| js_sys::Reflect::set(&obj, &x.0, x.1))
        .all(|y| y.is_ok());
    Ok(obj)
}
