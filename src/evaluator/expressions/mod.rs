use crate::environment::Environments;

use crate::evaluator::*;

use futures::future::{FutureExt, LocalBoxFuture};
use futures::stream::{self, StreamExt};
use js_sys::Error;
use wasm_bindgen_futures::JsFuture;

mod binary;
pub mod unary;

pub(crate) fn eval_expr<'a>(
    expr: Expr,
    envs: &'a mut Environments,
) -> LocalBoxFuture<'a, Result<RcValue, Error>> {
    async move {
        match expr {
            Expr::This(_) => Ok(envs.get("this")?),
            Expr::Lit(lit) => match lit {
                Lit::Num(number) => Ok(Value::Number(JsNumber::from(number.value)).into()),
                Lit::Bool(boolean) => Ok(Value::Bool(JsBool::from(boolean.value)).into()),
                Lit::Str(stringlit) => Ok(Value::String(JsString::from(&*stringlit.value)).into()),
                Lit::Null(_) => Ok(Value::Null(JsValue::null()).into()),
                Lit::Regex(regex) => Ok(Value::Object(js_sys::Object::from(js_sys::RegExp::new(
                    &regex.exp,
                    &regex.flags,
                )))
                .into()),
                _ => Err(Error::new(&format!(
                    "ERROR: Literal {:?} is not supported.",
                    lit
                ))),
            },
            Expr::Array(arraylit) => {
                let array = stream::iter(arraylit.elems)
                    .fold(
                        Ok((js_sys::Array::new(), envs)),
                        |acc: Result<(js_sys::Array, &mut Environments), Error>,
                         x: Option<ExprOrSpread>| async move {
                            let (arr, envs) = acc?;
                            match x {
                                Some(y) => match y.spread {
                                    Some(_) => {
                                        let spread = eval_expr(*y.expr, envs).await?;
                                        let spread = spread.borrow();
                                        crate::builtin::iterator::get_iterator(spread.as_ref())?
                                            .into_iter()
                                            .for_each(|z| {
                                                z.iter().for_each(|z| {
                                                    arr.push(&z);
                                                })
                                            })
                                    }
                                    None => {
                                        let val = eval_expr(*y.expr, envs).await?;
                                        arr.push(val.borrow().as_ref());
                                    }
                                },
                                None => {
                                    arr.push(&JsValue::undefined());
                                }
                            };
                            Ok((arr, envs))
                        },
                    )
                    .await?
                    .0;
                Ok(Value::Object(JsObject::from(array)).into())
            }
            Expr::Ident(ident) => Ok(envs.get(&ident.sym)?),
            Expr::Unary(unary) => Ok(unary::eval_unary_expression(
                unary.op,
                &eval_expr(*unary.arg, envs).await?.borrow(),
            )?
            .into()),
            Expr::Bin(binary) => Ok(binary::eval_binary_expression(
                binary.op,
                &eval_expr(*binary.left, envs).await?.borrow(),
                &eval_expr(*binary.right, envs).await?.borrow(),
            )?
            .into()),
            Expr::Arrow(array_func_expr) => {
                let body = functions::arrow_func_body(array_func_expr.body);
                functions::function_declaration(
                    array_func_expr.params,
                    body,
                    array_func_expr.is_async,
                    envs,
                    None,
                )
                .await
                .map(|x| x.into())
            }
            Expr::Fn(funexp) => {
                let body = match funexp.function.body {
                    Some(body) => Ok(body.stmts),
                    None => Err(Error::from(js_sys::TypeError::new(&format!(
                        "TypeError: Expected function body, got {:?}.",
                        funexp.function.body
                    )))),
                }?;
                functions::function_declaration(
                    funexp.function.params.into_iter().map(|x| x.pat).collect(),
                    body,
                    funexp.function.is_async,
                    envs,
                    None,
                )
                .await
                .map(|x| x.into())
            }
            Expr::Call(call) => functions::call_function(call, envs).await,
            Expr::Object(objlit) => objects::eval_obj_lit_expr(objlit, envs).await,
            Expr::Member(memexpr) => objects::eval_member_expr(memexpr, envs).await,
            Expr::New(newexpr) => objects::eval_new_expr(newexpr, envs).await,
            Expr::Paren(parexpr) => eval_expr(*parexpr.expr, envs).await,
            Expr::Await(awaitexpr) => match &*eval_expr(*awaitexpr.arg, envs).await?.borrow() {
                Value::Object(obj) => Ok(Value::from(
                    JsFuture::from(js_sys::Promise::from(JsValue::from(obj))).await?,
                )
                .into()),
                _ => Ok(Value::Undefined(JsValue::undefined()).into()),
            },
            Expr::Assign(assignexpr) => expressions::eval_assign_expr(assignexpr, envs).await,
            Expr::Update(update_expr) => {
                expressions::eval_update_expression(update_expr, envs).await
            }
            Expr::Seq(seq) => Ok(stream::iter(seq.exprs)
                .fold(
                    Ok::<_, Error>((Value::Undefined(JsValue::undefined()).into(), envs)),
                    |acc, x| async move {
                        let (_, envs) = acc?;
                        let new = eval_expr(*x, envs).await?;
                        Ok((new, envs))
                    },
                )
                .await?
                .0
                .into()),
            Expr::Cond(cond) => {
                if eval_expr(*cond.test, envs)
                    .await?
                    .borrow()
                    .as_ref()
                    .is_truthy()
                {
                    eval_expr(*cond.cons, envs).await
                } else {
                    eval_expr(*cond.alt, envs).await
                }
            }
            Expr::Tpl(template) => {
                let last = template.quasis[template.quasis.len() - 1].clone();
                let quasis_iter = stream::iter(template.quasis);
                let str = stream::iter(template.exprs)
                    .zip(quasis_iter)
                    .fold(
                        Ok::<_, Error>((JsString::from(JsValue::from_str("")), envs)),
                        |acc, x| async move {
                            let (string, envs) = acc?;
                            let expr = eval_expr(*x.0, envs).await?;
                            let next = JsString::from(JsValue::from_str(&match x.1.cooked {
                                Some(y) => y.value,
                                None => x.1.raw.value,
                            }))
                            .concat(expr.borrow().as_ref());
                            let new = string.concat(&next);
                            Ok((new, envs))
                        },
                    )
                    .await?
                    .0;
                let str = if last.tail {
                    str.concat(&JsString::from(JsValue::from_str(match &last.cooked {
                        Some(y) => &y.value,
                        None => &last.raw.value,
                    })))
                } else {
                    str
                };
                Ok(Value::String(str).into())
            }
            Expr::Class(class_expr) => class::eval_class(class_expr.class, envs)
                .await
                .map(|x| Value::JsFunction(x).into()),
            _ => Err(Error::new(&format!(
                "ERROR: Expression {:?} is not supported.",
                expr
            ))),
        }
    }
    .boxed_local()
}

pub(crate) async fn eval_pat(pat: Pat, envs: &mut Environments) -> Result<RcValue, Error> {
    match pat {
        Pat::Ident(ident) => envs.get(&ident.id.sym),
        Pat::Expr(expr) => eval_expr(*expr, envs).await,
        _ => Err(Error::new(&format!(
            "ERROR: Assignment pattern {:?} not supported.",
            pat
        ))),
    }
}

pub(crate) async fn eval_pat_or_exp(
    pat_or_exp: PatOrExpr,
    envs: &mut Environments,
) -> Result<RcValue, Error> {
    match pat_or_exp {
        PatOrExpr::Pat(pat) => eval_pat(*pat, envs).await,
        PatOrExpr::Expr(expr) => eval_expr(*expr, envs).await,
    }
}

pub(crate) async fn eval_assign_expr(
    assignexpr: AssignExpr,
    envs: &mut Environments,
) -> Result<RcValue, Error> {
    let left = eval_pat_or_exp(assignexpr.left.clone(), envs).await;
    let right = eval_expr(*assignexpr.right, envs).await;
    let new = match assignexpr.op {
        AssignOp::Assign => right,
        AssignOp::AddAssign => {
            binary::eval_plus_operator(&left?.borrow(), &right?.borrow()).map(|x| x.into())
        }
        AssignOp::SubAssign => {
            binary::eval_minus_operator(&left?.borrow(), &right?.borrow()).map(|x| x.into())
        }
        AssignOp::MulAssign => {
            binary::eval_multiplication_operator(&left?.borrow(), &right?.borrow())
                .map(|x| x.into())
        }
        AssignOp::DivAssign => {
            binary::eval_division_operator(&left?.borrow(), &right?.borrow()).map(|x| x.into())
        }
        AssignOp::ModAssign => {
            binary::eval_mod_operator(&left?.borrow(), &right?.borrow()).map(|x| x.into())
        }
        AssignOp::LShiftAssign => {
            binary::left_shift_operator(&left?.borrow(), &right?.borrow()).map(|x| x.into())
        }
        AssignOp::RShiftAssign => {
            binary::right_shift_operator(&left?.borrow(), &right?.borrow()).map(|x| x.into())
        }
        AssignOp::ZeroFillRShiftAssign => {
            binary::zero_right_shift_operator(&left?.borrow(), &right?.borrow()).map(|x| x.into())
        }
        AssignOp::BitOrAssign => {
            binary::bitwise_or_operator(&left?.borrow(), &right?.borrow()).map(|x| x.into())
        }
        AssignOp::BitAndAssign => {
            binary::bitwise_and_operator(&left?.borrow(), &right?.borrow()).map(|x| x.into())
        }
        AssignOp::BitXorAssign => {
            binary::bitwise_xor_operator(&left?.borrow(), &right?.borrow()).map(|x| x.into())
        }
        AssignOp::ExpAssign => {
            binary::eval_power_operator(&left?.borrow(), &right?.borrow()).map(|x| x.into())
        }
        AssignOp::AndAssign => {
            binary::eval_and_operator(&left?.borrow(), &right?.borrow()).map(|x| x.into())
        }
        AssignOp::OrAssign => {
            binary::eval_or_operator(&left?.borrow(), &right?.borrow()).map(|x| x.into())
        }
        AssignOp::NullishAssign => {
            binary::nullish_coalescing_operator(&left?.borrow(), &right?.borrow()).map(|x| x.into())
        }
    }?;
    match assignexpr.left {
        PatOrExpr::Pat(pat) => {
            statements::decl::set_pat(*pat, new, envs, statements::decl::DeclOrAssign::Assign).await
        }
        PatOrExpr::Expr(expr) => match *expr {
            Expr::Ident(ident) => {
                envs.set(&ident.sym, new)?;
                envs.get(&ident.sym)
            }
            Expr::Member(memberexpr) => objects::assign_member_expr(memberexpr, new, envs).await,
            _ => Err(Error::new(&format!(
                "ERROR: Expression {:?} not supported for assignment.",
                expr
            ))),
        },
    }
}

#[inline]
async fn eval_update_expression(
    update_expr: UpdateExpr,
    envs: &mut Environments,
) -> Result<RcValue, Error> {
    let prefix = if !update_expr.prefix {
        Some(eval_expr(*update_expr.arg.clone(), envs).await)
    } else {
        None
    };
    let update = match update_expr.op {
        UpdateOp::PlusPlus => eval_assign_expr(
            AssignExpr {
                span: update_expr.span,
                op: AssignOp::AddAssign,
                left: PatOrExpr::Expr(update_expr.arg),
                right: Box::new(Expr::Lit(Lit::Num(Number {
                    span: update_expr.span,
                    value: 1.0,
                }))),
            },
            envs,
        ),
        UpdateOp::MinusMinus => eval_assign_expr(
            AssignExpr {
                span: update_expr.span,
                op: AssignOp::SubAssign,
                left: PatOrExpr::Expr(update_expr.arg),
                right: Box::new(Expr::Lit(Lit::Num(Number {
                    span: update_expr.span,
                    value: 1.0,
                }))),
            },
            envs,
        ),
    }
    .await;
    match prefix {
        Some(result) => result,
        None => update,
    }
}
