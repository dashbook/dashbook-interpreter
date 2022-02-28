use crate::environment::{ClosedEnvironment, Environments};
use crate::evaluator;
use crate::value::{Function, *};

use futures::future::{FutureExt, LocalBoxFuture};
use futures::stream::{self, StreamExt};
use js_sys::Error;
use regex::Regex;
use std::cell::RefCell;
use std::rc::Rc;
use swc_common::{BytePos, Span, SyntaxContext};
use swc_ecma_ast::*;
use wasm_bindgen::prelude::*;

pub async fn function_declaration(
    args: Vec<Pat>,
    body: Vec<Stmt>,
    is_async: bool,
    envs: &mut Environments,
    prototype: Option<js_sys::Object>,
) -> Result<Value, Error> {
    let (args, env, _envs) = stream::iter(args)
        .fold(
            Ok((Vec::new(), envs.closure(), envs)),
            |acc: Result<(Vec<_>, ClosedEnvironment, &mut Environments), Error>, x| async move {
                let (mut args, mut env, envs) = acc?;
                args.append(&mut pat_to_string(x, &mut env, envs).await?);
                Ok((args, env, envs))
            },
        )
        .await?;
    let js_func = if is_async {
        JsValue::undefined()
    } else {
        new_jsfunction(&args, &body, &env)?
    };
    Ok(Value::Function(Function::new(
        args, body, env, is_async, js_func, prototype,
    )))
}

fn pat_to_string<'a>(
    pat: Pat,
    env: &'a mut ClosedEnvironment,
    envs: &'a mut Environments,
) -> LocalBoxFuture<'a, Result<Vec<String>, Error>> {
    async move {
        match pat {
            Pat::Ident(ident) => Ok(vec![String::from(&*ident.id.sym)]),
            Pat::Rest(rest) => pat_to_string(*rest.arg, env, envs).await,
            Pat::Assign(assign_pat) => {
                let names = pat_to_string(*assign_pat.left, env, envs).await?;
                env.insert(
                    &names[0],
                    evaluator::expressions::eval_expr(*assign_pat.right, envs).await?,
                );
                Ok(names)
            }
            Pat::Array(array_pat) => Ok(stream::iter(array_pat.elems)
                .fold(Ok::<_, Error>((vec![], env, envs)), |acc, x| async move {
                    let (mut vec, env, envs) = acc?;
                    match x {
                        Some(pat) => vec.append(&mut pat_to_string(pat, env, envs).await?),
                        None => (),
                    };
                    Ok((vec, env, envs))
                })
                .await?
                .0),
            _ => Err(Error::new(&format!(
                "ERROR: Pattern {:?} is not supported for functions arguments.",
                pat
            ))),
        }
    }
    .boxed_local()
}

pub fn arrow_func_body(arrow_body: BlockStmtOrExpr) -> Vec<Stmt> {
    match arrow_body {
        BlockStmtOrExpr::BlockStmt(block) => block.stmts,
        BlockStmtOrExpr::Expr(expr) => vec![Stmt::Expr(ExprStmt {
            span: Span::new(BytePos(0), BytePos(0), SyntaxContext::empty()),
            expr: expr,
        })],
    }
}

pub async fn call_function<'env>(
    call: CallExpr,
    envs: &mut Environments,
) -> Result<RcValue, Error> {
    let mut this = Rc::new(RefCell::new(Value::Undefined(JsValue::undefined())));
    let function = match call.callee {
        Callee::Expr(expr) => match *expr {
            Expr::Member(memexpr) => {
                this = evaluator::expressions::eval_expr(*memexpr.obj.clone(), envs).await?;
                evaluator::objects::eval_member_expr(memexpr, envs).await
            }
            _ => evaluator::expressions::eval_expr(*expr, envs).await,
        },
        Callee::Super(_) => Err(Error::new(&format!(
            "ERROR: Function callee super {:?} is not supported.",
            call.callee
        ))),
        Callee::Import(_) => Err(Error::new(&format!(
            "ERROR: Dynamic import for {:?} is not supported.",
            call.callee
        ))),
    }?;
    let len = call.args.len();
    let (mut args, _envs) = stream::iter(call.args)
        .fold(Ok((Vec::with_capacity(len), envs)), |acc, x| async move {
            let (mut vec, envs) = acc?;
            match x.spread {
                Some(_) => {
                    let spread = evaluator::expressions::eval_expr(*x.expr, envs).await?;
                    let spread = spread.borrow();
                    crate::builtin::iterator::get_iterator(spread.as_ref())?
                        .into_iter()
                        .for_each(|z| {
                            z.into_iter().for_each(|z| {
                                vec.push(Value::from(z).into());
                            })
                        })
                }
                None => {
                    vec.push(evaluator::expressions::eval_expr(*x.expr, envs).await?);
                }
            }

            Ok::<(Vec<_>, &mut Environments), Error>((vec, envs))
        })
        .await?;
    let result = match &*function.borrow() {
        Value::Function(func) => {
            let mut func_env = Environments::from_closed_env(func.env.clone());
            func_env.push_env();

            match &(*this.clone().borrow()) {
                Value::Object(_) => {
                    func_env.insert("this", this)?;
                }
                _ => (),
            }

            if args.len() == func.args.len() {
                args.into_iter().zip(func.args.iter()).for_each(|(x, y)| {
                    let _ = func_env.insert(y, x);
                });
            } else if args.len() > func.args.len() {
                let n = func.args.len() - 1;
                let rest = args
                    .split_off(n)
                    .into_iter()
                    .map(|x| x.borrow().as_ref().clone())
                    .collect::<js_sys::Array>();
                args.into_iter()
                    .zip(func.args.iter().take(n))
                    .for_each(|(x, y)| {
                        let _ = func_env.insert(y, x);
                    });
                func_env.insert(
                    &func
                        .args
                        .last()
                        .ok_or(Error::new("ERROR: Too few functions arguments."))?,
                    Rc::new(RefCell::new(Value::Object(js_sys::Object::from(rest)))),
                )?;
            } else {
                func_env.pop_env();
                return Err(Error::new(&format!(
                    "ERROR: Functions requires {:} arguments, {:} where given.",
                    func.args.len(),
                    args.len()
                )));
            };
            let result = if func.async_ {
                evaluator::eval(func.body.clone(), &mut func_env)
                    .await
                    .map(|x| {
                        Rc::new(RefCell::new(Value::from(JsValue::from(
                            js_sys::Object::from(js_sys::Promise::resolve(x.borrow().as_ref())),
                        ))))
                    })
            } else {
                evaluator::eval(func.body.clone(), &mut func_env).await
            };
            func_env.pop_env();
            result
        }
        Value::JsFunction(func) => Ok(Rc::new(RefCell::new(Value::from(
            func.apply(
                this.borrow().as_ref(),
                &args
                    .iter()
                    .map(|x| JsValue::from(&x.borrow() as &Value))
                    .collect::<js_sys::Array>(),
            )?,
        )))),
        _ => Err(Error::from(js_sys::TypeError::new(&format!(
            "TypeError: {} is not a function.",
            function.borrow()
        )))),
    };
    result
}

pub fn new_jsfunction(
    args: &Vec<String>,
    body: &Vec<Stmt>,
    env: &ClosedEnvironment,
) -> Result<JsValue, JsValue> {
    let mut func_body = body.clone();
    func_body.pop().map(|x| {
        let ret = if let Stmt::Expr(expr) = x {
            Stmt::Return(ReturnStmt {
                span: expr.span,
                arg: Some(expr.expr),
            })
        } else {
            x
        };
        func_body.push(ret);
    });
    let body = crate::transform::emit_js(func_body)?;
    let mut new_args: Vec<String> = args
        .iter()
        .map(|x| {
            let mut new = String::from("weblab_");
            new.push_str(x);
            new
        })
        .collect();
    let env_bindings = env.bindings();
    let extra = get_variable_names(&body).and_then(|x| {
        let arr = x
            .into_iter()
            .filter(|y| {
                !(new_args.iter().any(|z| y == z))
                    && (env_bindings.contains_key(y.split_at(7).1)
                        || in_global_this(y.split_at(7).1))
            })
            .collect::<Vec<String>>();
        if arr.len() != 0 {
            Some(arr)
        } else {
            None
        }
    });
    let values = extra.clone().and_then(|x| {
        x.iter()
            .map(|y| env.get(y.split_at(7).1))
            .collect::<Result<Vec<RcValue>, JsValue>>()
            .ok()
    });
    extra.map(|mut x| {
        x.append(&mut new_args);
        new_args = x
    });
    let mut new_args = new_args.iter().fold(String::from(""), |mut acc, x| {
        acc.push_str(x);
        acc.push_str(",");
        acc
    });
    new_args.pop();
    match values {
        Some(arr) => Ok(JsValue::from(arr.iter().fold(
            js_sys::Function::new_with_args(&new_args, &body),
            |acc, x| acc.bind1(&JsValue::null(), x.borrow().as_ref()),
        ))),
        None => Ok(JsValue::from(js_sys::Function::new_with_args(
            &new_args, &body,
        ))),
    }
}

fn get_variable_names(input: &str) -> Option<Vec<String>> {
    let re = Regex::new(r"weblab_[a-zA-Z_$][0-9a-zA-Z_$]*").ok()?;
    re.captures_iter(input)
        .map(|x| x.get(0).map(|y| String::from(y.as_str())))
        .collect::<Option<Vec<String>>>()
        .map(|mut x| {
            x.sort();
            x.dedup();
            x
        })
}

fn in_global_this(input: &str) -> bool {
    if input == "fetch"
        || input == "console"
        || input == "undefined"
        || input == "Number"
        || input == "Boolean"
        || input == "Object"
        || input == "Function"
        || input == "String"
        || input == "Generator"
        || input == "Iterator"
        || input == "Symbol"
        || input == "Math"
        || input == "Date"
        || input == "RegExp"
        || input == "Array"
        || input == "Int8Array"
        || input == "Uint8Array"
        || input == "Uint8ClampedArray"
        || input == "Int16Array"
        || input == "Uint16Array"
        || input == "Int32Array"
        || input == "Uint32Array"
        || input == "Float32Array"
        || input == "Float64Array"
        || input == "BigInt64Array"
        || input == "BigUint64Array"
        || input == "Promise"
        || input == "Map"
        || input == "Set"
        || input == "WeakMap"
        || input == "WeakSet"
        || input == "ArrayBuffer"
        || input == "SharedArrayBuffer"
        || input == "Error"
        || input == "EvalError"
        || input == "RangeError"
        || input == "ReferenceError"
        || input == "SyntaxError"
        || input == "TypeError"
        || input == "DataView"
        || input == "JSON"
        || input == "parseFloat"
        || input == "parseInt"
        || input == "NaN"
        || input == "isNaN"
        || input == "Infinity"
        || input == "isFinite"
        || input == "ImageData"
        || input == "URL"
        || input == "createImageBitmap"
        || input == "setTimeout"
        || input == "clearTimeout"
        || input == "setInterval"
        || input == "clearInterval"
        || input == "TextEncoder"
        || input == "TextDecoder"
    {
        true
    } else {
        false
    }
}
