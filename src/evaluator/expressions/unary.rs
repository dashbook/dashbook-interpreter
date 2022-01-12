use crate::value::*;
use js_sys::Error;
use swc_ecma_ast::*;
use wasm_bindgen::prelude::*;

pub fn eval_unary_expression(opr: UnaryOp, argument: &Value) -> Result<Value, Error> {
    match opr {
        UnaryOp::Bang => eval_not_operator(argument),
        UnaryOp::Minus => eval_minus_operator(argument),
        UnaryOp::Plus => eval_plus_operator(argument),
        UnaryOp::TypeOf => eval_typeof_operator(argument),
        UnaryOp::Void => eval_void_operator(argument),
        _ => Err(Error::new(&format!(
            "ERROR: Unary operator {:?} is not supported.",
            opr
        ))),
    }
}

pub fn eval_not_operator(argument: &Value) -> Result<Value, Error> {
    match argument {
        Value::Bool(x) => Ok(Value::Bool(JsBool::from(!x.value_of()))),
        Value::Number(x) => Ok(Value::Bool(JsBool::from(JsValue::from(x)))),
        Value::Null(_) => Ok(Value::Bool(JsBool::from(true))),
        Value::Undefined(_) => Ok(Value::Bool(JsBool::from(true))),
        Value::String(x) => Ok(Value::Bool(JsBool::from(JsValue::from(x)))),
        Value::Object(x) => Ok(Value::Bool(JsBool::from(JsValue::from(x)))),
        _ => Err(Error::from(js_sys::TypeError::new(&format!(
            "TypeError: Not operator not defined for {:}.",
            argument
        )))),
    }
}

pub fn eval_minus_operator(argument: &Value) -> Result<Value, Error> {
    match argument {
        Value::Number(x) => Ok(Value::Number(JsNumber::from(-x.value_of()))),
        _ => Err(Error::from(js_sys::TypeError::new(&format!(
            "ERROR: Minus operator not defined for {:}.",
            argument
        )))),
    }
}

pub fn eval_plus_operator(argument: &Value) -> Result<Value, Error> {
    match argument {
        Value::Bool(x) => Ok(Value::Number(JsNumber::from(JsValue::from(x)))),
        Value::Number(x) => Ok(Value::Number(x.clone())),
        Value::Null(_) => Ok(Value::Number(JsNumber::from(JsValue::null()))),
        Value::Undefined(_) => Ok(Value::Number(JsNumber::from(JsValue::undefined()))),
        Value::String(x) => Ok(Value::Number(JsNumber::from(JsValue::from(x)))),
        _ => Err(Error::from(js_sys::TypeError::new(&format!(
            "ERROR: Plus operator not defined for {:}.",
            argument
        )))),
    }
}

pub fn eval_typeof_operator(argument: &Value) -> Result<Value, Error> {
    match argument {
        Value::Bool(_) => Ok(Value::String(JsString::from("boolean"))),
        Value::Number(_) => Ok(Value::String(JsString::from("number"))),
        Value::Null(_) => Ok(Value::String(JsString::from("object"))),
        Value::Undefined(_) => Ok(Value::String(JsString::from("undefined"))),
        Value::String(_) => Ok(Value::String(JsString::from("string"))),
        Value::Object(_) => Ok(Value::String(JsString::from("object"))),
        Value::Function(_) => Ok(Value::String(JsString::from("function"))),
        Value::JsFunction(_) => Ok(Value::String(JsString::from("function"))),
        Value::Symbol(_) => Ok(Value::String(JsString::from("symbol"))),
        Value::BigInt(_) => Ok(Value::String(JsString::from("bigint"))),
    }
}
pub fn eval_void_operator(argument: &Value) -> Result<Value, Error> {
    match argument {
        Value::Bool(x) => {
            x.value_of();
        }
        Value::Number(x) => {
            x.value_of();
        }
        Value::Null(_) => (),
        Value::Undefined(_) => (),
        Value::String(x) => {
            x.value_of();
        }
        Value::Object(x) => {
            x.value_of();
        }
        _ => (),
    }
    Ok(Value::Undefined(JsValue::undefined()))
}
