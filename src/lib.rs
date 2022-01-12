/// Weblab-interpreter
///
/// Influences:
/// - Thorsten Ball - Writing an interpreter in Go
/// - How JavaScript Objects are Implemented - https://www.infoq.com/presentations/javascript-objects-spidermonkey/
///
use futures::stream::{Peekable, StreamExt};
use futures::SinkExt;
use lazy_static::lazy_static;
use std::cell::RefCell;
use std::pin::Pin;
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
    let mut es_config: EsConfig = Default::default();
    es_config.top_level_await = true;
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
