use crate::environment::Environments;

use crate::value::*;

use futures::future::Either;
use futures::prelude::*;
use futures::stream::{self, StreamExt};
use js_sys::Error;
use std::cell::RefCell;
use std::rc::Rc;
use swc_ecma_ast::*;
use wasm_bindgen::prelude::*;

pub mod class;
pub mod expressions;
pub mod functions;
mod objects;
mod statements;

pub async fn eval_module(
    stmts: Vec<ModuleItem>,
    envs: &mut Environments,
) -> Result<RcValue, Error> {
    stream::iter(stmts)
        .fold(
            Ok((
                Rc::new(RefCell::new(Value::Undefined(JsValue::undefined()))),
                envs,
            )),
            |acc, x| async move {
                let acc = acc?;
                match x {
                    ModuleItem::ModuleDecl(moddecl) => {
                        Either::Left(eval_module_decl(moddecl, acc.1))
                    }
                    ModuleItem::Stmt(stmt) => Either::Right(statements::eval_stmt(stmt, acc.1)),
                }
                .await
                .map(|x| {
                    let _ = &acc;
                    (x, acc.1)
                })
            },
        )
        .await
        .map(|x| x.0)
}

async fn eval<'env>(stmts: Vec<Stmt>, envs: &mut Environments) -> Result<RcValue, Error> {
    stream::iter(stmts)
        .fold(
            Ok((
                Rc::new(RefCell::new(Value::Undefined(JsValue::undefined()))),
                envs,
            )),
            |acc, x| async move {
                let acc = acc?;
                statements::eval_stmt(x, acc.1).await.map(|x| {
                    let _ = &acc;
                    (x, acc.1)
                })
            },
        )
        .await
        .map(|x| x.0)
}

#[inline]
async fn eval_module_decl(moddecl: ModuleDecl, envs: &mut Environments) -> Result<RcValue, Error> {
    match moddecl {
        ModuleDecl::Import(importdecl) => {
            let module = crate::js::import(&importdecl.src.value.to_string())
                .await
                .map(|x| Value::from(x))
                .map(|y| Rc::new(RefCell::new(y)))?;
            let mut module_specifier = None;
            importdecl
                .specifiers
                .iter()
                .map(|x| match x {
                    ImportSpecifier::Namespace(specifier) => {
                        module_specifier = Some(specifier.local.sym.clone());
                        Ok(())
                    }
                    ImportSpecifier::Named(specifier) => {
                        let obj = js_sys::Reflect::get(
                            module.borrow().as_ref(),
                            &JsValue::from_str(
                                &specifier
                                    .imported
                                    .clone()
                                    .map(|x| match x {
                                        ModuleExportName::Ident(ident) => ident.sym,
                                        ModuleExportName::Str(str) => str.value,
                                    })
                                    .unwrap_or(specifier.local.clone().sym),
                            ),
                        )?;
                        envs.insert(
                            &specifier.local.sym,
                            Rc::new(RefCell::new(Value::from(obj))),
                        )?;
                        Ok(())
                    }
                    ImportSpecifier::Default(specifier) => {
                        let obj = js_sys::Reflect::get(
                            module.borrow().as_ref(),
                            &JsValue::from_str(&specifier.local.sym),
                        )?;
                        envs.insert(
                            &specifier.local.sym,
                            Rc::new(RefCell::new(Value::from(obj))),
                        )?;
                        Ok(())
                    }
                })
                .collect::<Result<(), JsValue>>()?;
            module_specifier.and_then(|x| {
                let global = js_sys::global();
                js_sys::Object::define_property(
                    &global,
                    &JsValue::from_str(&x),
                    &objects::create_object_from_entries(vec![
                        (JsValue::from_str("configurable"), &JsValue::from_bool(true)),
                        (JsValue::from_str("enumerable"), &JsValue::from_bool(true)),
                        (JsValue::from_str("writable"), &JsValue::from_bool(false)),
                        (JsValue::from_str("value"), module.borrow().as_ref()),
                    ])
                    .ok()?,
                );
                envs.insert(&x, module).ok()
            });
            Ok(Rc::new(RefCell::new(
                Value::Undefined(JsValue::undefined()),
            )))
        }
        _ => Err(Error::new(&format!(
            "ERROR: Module declaration {:?} not supported.",
            moddecl
        ))),
    }
}
