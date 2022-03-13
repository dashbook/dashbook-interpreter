use futures::{stream, StreamExt};
use js_sys::Error;
use swc_ecma_ast::*;
use wasm_bindgen::JsValue;

use crate::{value::Value, Environments};

use super::{expressions::eval_expr, functions, objects};

pub(crate) async fn eval_class(
    class: swc_ecma_ast::Class,
    envs: &mut Environments,
) -> Result<js_sys::Function, Error> {
    let constructor = match class.body.iter().find(|x| match *x {
        ClassMember::Constructor(_) => true,
        _ => false,
    }) {
        Some(ClassMember::Constructor(constructor)) => Ok(constructor.clone()),
        None => Ok(Constructor {
            span: swc_common::Span::default(),
            key: PropName::Ident(Ident {
                span: swc_common::Span::default(),
                sym: String::from("constructor").into(),
                optional: false,
            }),
            params: vec![],
            body: None,
            accessibility: None,
            is_optional: false,
        }),
        _ => Err(Error::new("Class definition has no constructor.")),
    }?;
    let body = match constructor.body {
        Some(body) => body.stmts,
        None => Vec::new(),
    };
    let (args, env, _envs) = functions::args_to_string(
        constructor
            .params
            .into_iter()
            .map(|x| match x {
                ParamOrTsParamProp::Param(param) => Ok(param.pat),
                _ => Err(Error::new(&format!("Parameters {:?} not supported.", x))),
            })
            .collect::<Result<Vec<_>, _>>()?,
        envs,
    )
    .await?;
    let function = functions::new_jsfunction(&args, &body, &env)?;
    let prototype = match class.super_class {
        Some(super_class) => match &*eval_expr(*super_class, envs).await?.borrow() {
            Value::JsFunction(proto) => {
                let obj = js_sys::Object::new();
                Ok(js_sys::Object::set_prototype_of(
                    &obj,
                    &js_sys::Object::from(js_sys::Reflect::get(
                        proto,
                        &JsValue::from_str(&"prototype"),
                    )?),
                ))
            }
            _ => Err(js_sys::Error::from(js_sys::TypeError::new(""))),
        },
        None => Ok(js_sys::Object::new()),
    }?;
    let (function,prototype, _envs) = stream::iter(class.body)
        .fold(
            Ok((function, prototype, envs)),
            |acc: Result<(js_sys::Function, js_sys::Object, &mut Environments), Error>, x| async move {
                let (func, obj, envs) = acc?;
                match x {
                    ClassMember::Method(method) => {
                        let body = method
                            .function
                            .body
                            .ok_or(Error::new(&format!(
                                "ERROR: Function body of method property invalid."
                            )))?
                            .stmts;
                        let (args, env, envs) = functions::args_to_string(
                            method.function.params.into_iter().map(|x| x.pat).collect(),
                            envs,
                        )
                        .await?;
                        let function = functions::new_jsfunction(&args, &body, &env)?;
                        if method.is_static {
                            js_sys::Object::define_property(
                                func.as_ref(),
                                objects::get_prop_name(method.key, envs)
                                    .await?
                                    .borrow()
                                    .as_ref(),
                                &objects::create_object_from_entries(vec![
                                    (
                                        JsValue::from_str("configurable"),
                                        &JsValue::from_bool(false),
                                    ),
                                    (JsValue::from_str("enumerable"), &JsValue::from_bool(true)),
                                    (JsValue::from_str("writable"), &JsValue::from_bool(true)),
                                    (JsValue::from_str("value"), &function),
                                ])?,
                            );
                        } else {
                            js_sys::Object::define_property(
                                obj.as_ref(),
                                objects::get_prop_name(method.key, envs)
                                    .await?
                                    .borrow()
                                    .as_ref(),
                                &objects::create_object_from_entries(vec![
                                    (
                                        JsValue::from_str("configurable"),
                                        &JsValue::from_bool(false),
                                    ),
                                    (JsValue::from_str("enumerable"), &JsValue::from_bool(true)),
                                    (JsValue::from_str("writable"), &JsValue::from_bool(true)),
                                    (JsValue::from_str("value"), &function),
                                ])?,
                            );
                        }
                        Ok((func, obj, envs))
                    }
                    ClassMember::ClassProp(prop) => {
                        let value = match prop.value {
                            Some(prop) => eval_expr(*prop, envs).await?,
                            None => Value::Undefined(JsValue::undefined()).into(),
                        };
                        if prop.is_static {
                            js_sys::Object::define_property(
                                func.as_ref(),
                                objects::get_prop_name(prop.key, envs)
                                    .await?
                                    .borrow()
                                    .as_ref(),
                                &objects::create_object_from_entries(vec![
                                    (
                                        JsValue::from_str("configurable"),
                                        &JsValue::from_bool(false),
                                    ),
                                    (JsValue::from_str("enumerable"), &JsValue::from_bool(true)),
                                    (JsValue::from_str("writable"), &JsValue::from_bool(true)),
                                    (JsValue::from_str("value"), &value.borrow().as_ref()),
                                ])?,
                            );
                        } else {
                            js_sys::Object::define_property(
                                obj.as_ref(),
                                objects::get_prop_name(prop.key, envs)
                                    .await?
                                    .borrow()
                                    .as_ref(),
                                &objects::create_object_from_entries(vec![
                                    (
                                        JsValue::from_str("configurable"),
                                        &JsValue::from_bool(false),
                                    ),
                                    (JsValue::from_str("enumerable"), &JsValue::from_bool(true)),
                                    (JsValue::from_str("writable"), &JsValue::from_bool(true)),
                                    (JsValue::from_str("value"), &value.borrow().as_ref()),
                                ])?,
                            );
                        }
                        
                        Ok((func, obj, envs))
                    }
                    _ => Ok((func, obj, envs)),
                }
            },
        )
        .await?;
    js_sys::Object::define_property(
        &prototype,
        &JsValue::from_str("constructor"),
        &objects::create_object_from_entries(vec![
            (
                JsValue::from_str("configurable"),
                &JsValue::from_bool(false),
            ),
            (JsValue::from_str("enumerable"), &JsValue::from_bool(false)),
            (JsValue::from_str("writable"), &JsValue::from_bool(false)),
            (JsValue::from_str("value"), &function),
        ])?,
    );
    js_sys::Object::define_property(
        &function,
        &JsValue::from_str("prototype"),
        &objects::create_object_from_entries(vec![
            (
                JsValue::from_str("configurable"),
                &JsValue::from_bool(false),
            ),
            (JsValue::from_str("enumerable"), &JsValue::from_bool(false)),
            (JsValue::from_str("writable"), &JsValue::from_bool(false)),
            (JsValue::from_str("value"), &prototype),
        ])?,
    );
    Ok(function)
}
