use crate::environment::Environments;
use crate::evaluator;
use crate::value::Value;

use std::cell::RefCell;
use std::rc::Rc;

use js_sys::Error;
use swc_common::sync::Lrc;
use swc_common::{
    errors::{ColorConfig, Handler},
    FileName, SourceMap,
};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};

pub async fn test_eval(input: &str, envs: &mut Environments) -> Result<Rc<RefCell<Value>>, Error> {
    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));
    let fm = cm.new_source_file(FileName::Custom("Weblab_cell".into()), input.into());
    let lexer = Lexer::new(
        Syntax::Es(Default::default()),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );
    let mut parser = Parser::new_from(lexer);
    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }
    let module = parser
        .parse_module()
        .map_err(|x| Error::new(&x.into_diagnostic(&handler).message()))?;
    let items = module.body;
    evaluator::eval_module(items, envs).await
}
