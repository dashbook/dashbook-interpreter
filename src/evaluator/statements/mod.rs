use crate::environment::Environments;
use crate::evaluator::*;

use futures::future::{FutureExt, LocalBoxFuture};
use js_sys::Error;

pub(crate) mod decl;

pub(crate) fn eval_stmt<'a>(
    stmt: Stmt,
    envs: &'a mut Environments,
) -> LocalBoxFuture<'a, Result<RcValue, Error>> {
    async move {
        match stmt {
            Stmt::Decl(decl) => {
                decl::eval_decl(decl, envs).await?;
                Ok(Value::Undefined(JsValue::undefined()).into())
            }
            Stmt::Expr(expr) => expressions::eval_expr(*expr.expr, envs).await,
            Stmt::Block(block) => eval(block.stmts, envs).await,
            Stmt::If(ifstmt) => eval_if_stmt(ifstmt, envs).await,
            Stmt::Return(expr) => match expr.arg {
                Some(e) => expressions::eval_expr(*e, envs).await,
                None => Ok(Value::Null(JsValue::null()).into()),
            },
            Stmt::Empty(_) => Ok(Value::Null(JsValue::null()).into()),
            Stmt::Throw(throw) => Err(Error::new(&format!(
                "Error: Exception thrown with {}",
                expressions::eval_expr(*throw.arg, envs).await?.borrow()
            ))),
            Stmt::Try(try_stmt) => {
                let handler = try_stmt.handler;
                let result = {
                    envs.push_env();
                    let result = eval(try_stmt.block.stmts, envs).await;
                    envs.pop_env();
                    result
                };
                future::ready(result)
                    .or_else(|x| async move {
                        match handler {
                            Some(y) => {
                                future::ok(y)
                                    .and_then(|y| async move {
                                        envs.push_env();
                                        y.param.and_then(|z| match z {
                                            Pat::Ident(ident) => envs
                                                .insert(
                                                    &ident.id.sym,
                                                    Value::Object(JsObject::from(x)).into(),
                                                )
                                                .ok(),
                                            _ => None,
                                        });
                                        let result = eval(y.body.stmts, envs).await;
                                        envs.pop_env();
                                        result
                                    })
                                    .await
                            }
                            None => Err(x),
                        }
                    })
                    .await
            }
            Stmt::While(while_stmt) => {
                while match &expressions::eval_expr(*while_stmt.test.clone(), envs)
                    .await?
                    .borrow() as &Value
                {
                    Value::Bool(bool) => bool.value_of(),
                    _ => false,
                } {
                    envs.push_env();
                    let body = *while_stmt.body.clone();
                    match body {
                        Stmt::Block(block_stmt) => {
                            match eval_block(block_stmt.stmts, envs).await? {
                                LoopBlock::Break => {
                                    envs.pop_env();
                                    break;
                                }
                                LoopBlock::Continue => {
                                    envs.pop_env();
                                    continue;
                                }
                                LoopBlock::Normal(_) => (),
                            }
                        }
                        Stmt::Continue(_) => {
                            envs.pop_env();
                            continue;
                        }
                        Stmt::Break(_) => {
                            envs.pop_env();
                            break;
                        }
                        _ => {
                            eval_stmt(body, envs).await?;
                        }
                    };
                    envs.pop_env();
                }
                Ok(Value::Undefined(JsValue::undefined()).into())
            }
            Stmt::DoWhile(while_stmt) => {
                loop {
                    envs.push_env();
                    let body = *while_stmt.body.clone();
                    match body {
                        Stmt::Block(block_stmt) => {
                            match eval_block(block_stmt.stmts, envs).await? {
                                LoopBlock::Break => {
                                    envs.pop_env();
                                    break;
                                }
                                LoopBlock::Continue => {
                                    envs.pop_env();
                                    continue;
                                }
                                LoopBlock::Normal(_) => (),
                            }
                        }
                        Stmt::Continue(_) => {
                            envs.pop_env();
                            continue;
                        }
                        Stmt::Break(_) => {
                            envs.pop_env();
                            break;
                        }
                        _ => {
                            eval_stmt(body, envs).await?;
                        }
                    };
                    if match &expressions::eval_expr(*while_stmt.test.clone(), envs)
                        .await?
                        .borrow() as &Value
                    {
                        Value::Bool(bool) => !bool.value_of(),
                        _ => !false,
                    } {
                        envs.pop_env();
                        break;
                    }
                }
                Ok(Value::Undefined(JsValue::undefined()).into())
            }
            Stmt::For(for_stmt) => {
                match for_stmt.init {
                    Some(x) => {
                        match x {
                            VarDeclOrExpr::VarDecl(var_decl) => {
                                decl::eval_decl(Decl::Var(var_decl), envs).await
                            }
                            VarDeclOrExpr::Expr(expr) => expressions::eval_expr(*expr, envs).await,
                        }?;
                        ()
                    }
                    None => (),
                };
                loop {
                    match for_stmt.test.clone() {
                        Some(x) => {
                            if match &expressions::eval_expr(*x, envs).await?.borrow() as &Value {
                                Value::Bool(bool) => !bool.value_of(),
                                _ => !false,
                            } {
                                break;
                            };
                            ()
                        }
                        None => (),
                    }
                    envs.push_env();
                    let body = *for_stmt.body.clone();
                    match body {
                        Stmt::Block(block_stmt) => {
                            match eval_block(block_stmt.stmts, envs).await? {
                                LoopBlock::Break => {
                                    break;
                                }
                                LoopBlock::Continue => {
                                    continue;
                                }
                                LoopBlock::Normal(_) => (),
                            }
                        }
                        Stmt::Continue(_) => {
                            envs.pop_env();
                            continue;
                        }
                        Stmt::Break(_) => {
                            envs.pop_env();
                            break;
                        }
                        _ => {
                            eval_stmt(body, envs).await?;
                        }
                    };
                    match for_stmt.update.clone() {
                        Some(x) => {
                            expressions::eval_expr(*x, envs).await?;
                            ()
                        }
                        None => (),
                    }
                    envs.pop_env();
                }
                Ok(Value::Undefined(JsValue::undefined()).into())
            }
            Stmt::ForOf(for_of_stmt) => {
                let object = expressions::eval_expr(*for_of_stmt.right, envs).await?;
                let iterator = crate::builtin::iterator::get_iterator(object.borrow().as_ref())?;
                for x in iterator.into_iter() {
                    envs.push_env();
                    match for_of_stmt.left.clone() {
                        ForHead::Pat(pat) => {
                            decl::set_pat(
                                *pat,
                                Value::from(x?).into(),
                                envs,
                                decl::DeclOrAssign::Decl,
                            )
                            .await
                        }
                        ForHead::VarDecl(var_decls) => {
                            decl::set_pat(
                                var_decls.decls[0].name.clone(),
                                Value::from(x?).into(),
                                envs,
                                decl::DeclOrAssign::Decl,
                            )
                            .await
                        }
                        ForHead::UsingDecl(_) => {
                            Err(Error::new(&format!("Using decl not supported.")))
                        }
                    }?;
                    let body = *for_of_stmt.body.clone();
                    match body {
                        Stmt::Block(block_stmt) => {
                            match eval_block(block_stmt.stmts, envs).await? {
                                LoopBlock::Break => {
                                    break;
                                }
                                LoopBlock::Continue => {
                                    continue;
                                }
                                LoopBlock::Normal(_) => (),
                            }
                        }
                        Stmt::Continue(_) => {
                            envs.pop_env();
                            continue;
                        }
                        Stmt::Break(_) => {
                            envs.pop_env();
                            break;
                        }
                        _ => {
                            eval_stmt(body, envs).await?;
                        }
                    };
                    envs.pop_env();
                }
                Ok(Value::Undefined(JsValue::undefined()).into())
            }
            Stmt::ForIn(for_in_stmt) => {
                let right =
                    js_sys::Object::keys(match &*expressions::eval_expr(*for_in_stmt.right, envs)
                        .await?
                        .borrow()
                    {
                        Value::Object(x) => Ok(x),
                        y => Err(Error::new(&format!(
                            "Error: Object {:?} in for .. in loop is not enumerable.",
                            y
                        ))),
                    }?);
                for x in right.iter().into_iter() {
                    envs.push_env();
                    match for_in_stmt.left.clone() {
                        ForHead::Pat(pat) => {
                            decl::set_pat(
                                *pat,
                                Value::from(x).into(),
                                envs,
                                decl::DeclOrAssign::Decl,
                            )
                            .await
                        }
                        ForHead::VarDecl(var_decls) => {
                            decl::set_pat(
                                var_decls.decls[0].name.clone(),
                                Value::from(x).into(),
                                envs,
                                decl::DeclOrAssign::Decl,
                            )
                            .await
                        }
                        ForHead::UsingDecl(_) => {
                            Err(Error::new(&format!("Using decl not supported.")))
                        }
                    }?;
                    let body = *for_in_stmt.body.clone();
                    match body {
                        Stmt::Block(block_stmt) => {
                            match eval_block(block_stmt.stmts, envs).await? {
                                LoopBlock::Break => {
                                    break;
                                }
                                LoopBlock::Continue => {
                                    continue;
                                }
                                LoopBlock::Normal(_) => (),
                            }
                        }
                        Stmt::Continue(_) => {
                            envs.pop_env();
                            continue;
                        }
                        Stmt::Break(_) => {
                            envs.pop_env();
                            break;
                        }
                        _ => {
                            eval_stmt(body, envs).await?;
                        }
                    };
                    envs.pop_env();
                }
                Ok(Value::Undefined(JsValue::undefined()).into())
            }
            _ => Err(Error::new(&format!(
                "ERROR: Statement {:?} is not supported.",
                stmt
            ))),
        }
    }
    .boxed_local()
}

enum LoopBlock {
    Break,
    Continue,
    Normal(RcValue),
}

#[inline]
async fn eval_block<'env>(stmts: Vec<Stmt>, envs: &mut Environments) -> Result<LoopBlock, Error> {
    stream::iter(stmts)
        .fold(
            Ok((
                LoopBlock::Normal(Value::Undefined(JsValue::undefined()).into()),
                envs,
            )),
            |acc, x| async move {
                let acc = acc?;
                let result = match acc.0 {
                    LoopBlock::Normal(_) => match x {
                        Stmt::Break(_) => LoopBlock::Break,
                        Stmt::Continue(_) => LoopBlock::Continue,
                        _ => LoopBlock::Normal(statements::eval_stmt(x, acc.1).await?),
                    },
                    y => y,
                };
                Ok((result, acc.1))
            },
        )
        .await
        .map(|x| x.0)
}

#[inline]
async fn eval_if_stmt(ifstmt: IfStmt, envs: &mut Environments) -> Result<RcValue, Error> {
    let test = JsBool::from(JsValue::from(
        &expressions::eval_expr(*ifstmt.test, envs).await?.borrow() as &Value,
    ))
    .value_of();
    if test {
        statements::eval_stmt(*ifstmt.cons, envs).await
    } else {
        match ifstmt.alt {
            Some(x) => statements::eval_stmt(*x, envs).await,
            None => Ok(Value::Undefined(JsValue::undefined()).into()),
        }
    }
}
