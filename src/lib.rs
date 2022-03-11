/// Weblab-interpreter
///
/// Influences:
/// - Thorsten Ball - Writing an interpreter in Go
/// - How JavaScript Objects are Implemented - https://www.infoq.com/presentations/javascript-objects-spidermonkey/
///
use futures::stream::{Peekable, StreamExt};
use futures::SinkExt;
use js_sys::Error;
use lazy_static::lazy_static;
use std::cell::RefCell;
use std::pin::Pin;
use value::Value;
use wasm_bindgen::prelude::*;

use futures::channel::mpsc;
use futures::lock::Mutex;

pub use crate::environment::Environments;
use swc_common::sync::Lrc;
use swc_common::{
    errors::{ColorConfig, Handler},
    FileName, SourceMap,
};
use swc_ecma_parser::{lexer::Lexer, EsConfig, Parser, StringInput, Syntax};

mod builtin;
mod environment;
mod evaluator;
mod js;
mod transform;
mod utils;
mod value;

#[cfg(test)]
mod test;

#[wasm_bindgen(start)]
pub fn start() {
    utils::set_panic_hook();
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

thread_local! {
    pub static ENVS: RefCell<Environments> = RefCell::new(Environments::new());
}

lazy_static! {
    pub static ref CHANNEL: (
        Mutex<mpsc::UnboundedSender<String>>,
        Mutex<Peekable<mpsc::UnboundedReceiver<String>>>
    ) = {
        let channel = mpsc::unbounded::<String>();
        (Mutex::new(channel.0), Mutex::new(channel.1.peekable()))
    };
}

#[wasm_bindgen]
pub fn reset_envs() {
    ENVS.with(|f| {
        f.replace(Environments::new());
    });
}

#[wasm_bindgen]
pub async fn eval_cell(input: String) -> Result<JsValue, JsValue> {
    async {
        let mut tx = async { CHANNEL.0.lock().await.clone() }.await;
        tx.send(input.clone()).await.unwrap();
    }
    .await;
    let lock = CHANNEL.1.lock().await;
    let mut pin = Pin::new(lock);
    pin.as_mut().next_if_eq(&input).await;
    let mut envs = Environments::empty();
    ENVS.with(|f| {
        envs = f.replace(Environments::empty());
    });
    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    let fm = cm.new_source_file(FileName::Custom("Weblab_cell".into()), input.into());
    let es_config: EsConfig = Default::default();
    let lexer = Lexer::new(
        Syntax::Es(es_config),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);

    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    let result = match parser.parse_module() {
        Ok(module) => evaluator::eval_module(module.body, &mut envs)
            .await
            .and_then(|x| x.borrow().output())
            .or_else(|err| Ok(JsValue::from(err.message()))),
        Err(err) => Ok(JsValue::from(
            js_sys::SyntaxError::new(&err.kind().msg()).message(),
        )),
    };
    ENVS.with(|f| {
        f.replace(envs);
    });
    result
}

#[wasm_bindgen]
pub fn list_properties(object: &str) -> js_sys::Object {
    let mut properties = None;
    ENVS.with(|f| {
        properties = if object == "window" {
            f.borrow().stack[0]
                .iter()
                .map(|x| {
                    Ok([
                        JsValue::from_str(&x.0),
                        crate::evaluator::expressions::unary::eval_typeof_operator(&x.1.borrow())?
                            .into(),
                    ]
                    .into_iter()
                    .collect::<js_sys::Array>())
                })
                .collect::<Result<js_sys::Array, Error>>()
                .ok()
        } else {
            f.borrow().stack[0]
                .get(object)
                .and_then(|value| match &*value.borrow() {
                    Value::Object(obj) => Some(
                        js_sys::Object::entries(obj)
                            .iter()
                            .map(|tuple| {
                                let tuple = js_sys::Array::from(&tuple);
                                let value = tuple.pop();
                                tuple.push(&value.js_typeof());
                                tuple
                            })
                            .collect::<js_sys::Array>(),
                    ),
                    _ => None,
                })
        }
    });
    properties
        .and_then(|x| js_sys::Object::from_entries(&x).ok())
        .unwrap_or(js_sys::Object::new())
}

#[wasm_bindgen]
pub fn get_type(object: &str) -> JsValue {
    let mut type_ = None;
    ENVS.with(|f| {
        type_ = f.borrow().stack[0]
            .get(object)
            .map(|value| value.borrow().as_ref().js_typeof())
    });
    type_.unwrap_or(JsValue::from_str("undefined"))
}
