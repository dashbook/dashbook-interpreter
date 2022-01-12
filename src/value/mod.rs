use crate::environment::ClosedEnvironment;
use js_sys::Error;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use swc_ecma_ast::Stmt;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub type JsNumber = js_sys::Number;
pub type JsBool = js_sys::Boolean;
pub type JsString = js_sys::JsString;
pub type JsObject = js_sys::Object;
pub type JsFunction = js_sys::Function;
pub type JsSymbol = js_sys::Symbol;
pub type JsBigInt = js_sys::BigInt;

#[derive(Debug, Clone)]
pub enum Value {
    Number(JsNumber),
    Bool(JsBool),
    String(JsString),
    Null(JsValue),
    Undefined(JsValue),
    Function(Function),
    JsFunction(JsFunction),
    Object(JsObject),
    Symbol(JsSymbol),
    BigInt(JsBigInt),
}

pub type RcValue = Rc<RefCell<Value>>;

impl Value {
    pub fn output(&self) -> Result<JsValue, Error> {
        match self {
            Value::Number(x) => Ok(JsValue::from(x.to_string(10)?)),
            Value::Bool(x) => Ok(JsValue::from(x.to_string())),
            Value::Null(_) => Ok(JsValue::from_str("null")),
            Value::Undefined(_) => Ok(JsValue::from_str("undefined")),
            Value::String(x) => Ok(JsValue::from(x)),
            Value::Object(x) => {
                if x.is_instance_of::<web_sys::HtmlElement>() {
                    Ok(JsValue::from(x))
                } else {
                    Ok(JsValue::from(JsString::from(
                        js_sys::JSON::stringify(&JsValue::from(x))
                            .or_else(|_| {
                                let replacer = js_sys::Function::new_no_args(
                                    &"const seen = new WeakSet();
                    return (key, value) => {
                      if (typeof value === \"object\" && value !== null) {
                        if (seen.has(value)) {
                          return;
                        }
                        seen.add(value);
                      }
                      return value;
                    };",
                                );
                                js_sys::JSON::stringify_with_replacer(
                                    &JsValue::from(x),
                                    &replacer.call0(&JsValue::null())?,
                                )
                            })
                            .map(|y| String::from(y))
                            .unwrap_or_else(|x| String::from(Error::from(x).message())),
                    )))
                }
            }
            Value::Function(x) => Ok(JsValue::from(JsString::from(x.jsfunction.clone()))),
            Value::JsFunction(x) => Ok(JsValue::from(x.to_string())),
            Value::Symbol(x) => Ok(JsValue::from(x.to_string())),
            Value::BigInt(x) => Ok(JsValue::from(x.to_string(10)?)),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(x) => write!(f, "{}", x.value_of()),
            Value::Bool(x) => write!(f, "{}", x.value_of()),
            Value::Null(_) => write!(f, "null"),
            Value::Undefined(_) => write!(f, "undefined"),
            Value::String(x) => write!(f, "{}", String::from(x)),
            Value::Object(x) => {
                if x.is_instance_of::<web_sys::HtmlElement>() {
                    write!(
                        f,
                        "{}",
                        &web_sys::HtmlElement::from(JsValue::from(x)).outer_html()
                    )
                } else {
                    write!(
                        f,
                        "{}",
                        js_sys::JSON::stringify(&JsValue::from(x))
                            .or_else(|_| {
                                let replacer = js_sys::Function::new_no_args(
                                    &"const seen = new WeakSet();
                    return (key, value) => {
                      if (typeof value === \"object\" && value !== null) {
                        if (seen.has(value)) {
                          return;
                        }
                        seen.add(value);
                      }
                      return value;
                    };",
                                );
                                js_sys::JSON::stringify_with_replacer(
                                    &JsValue::from(x),
                                    &replacer.call0(&JsValue::null())?,
                                )
                            })
                            .map(|y| String::from(y))
                            .unwrap_or_else(|x| String::from(Error::from(x).message()))
                    )
                }
            }
            Value::Function(x) => write!(f, "{:?}", x),
            Value::JsFunction(x) => write!(f, "{}", String::from(x.to_string())),
            Value::Symbol(x) => write!(f, "{:?}", String::from(x.to_string())),
            Value::BigInt(x) => write!(f, "{}", x.value_of(10)),
        }
    }
}

impl From<JsValue> for Value {
    fn from(input: JsValue) -> Self {
        if input.is_undefined() {
            Value::Undefined(input)
        } else if let Some(bool) = input.as_bool() {
            Value::Bool(JsBool::from(bool))
        } else if let Some(number) = input.as_f64() {
            Value::Number(JsNumber::from(number))
        } else if let Some(string) = input.as_string() {
            Value::String(JsString::from(string))
        } else if input.is_object() {
            Value::Object(JsObject::from(input))
        } else if input.is_function() {
            Value::JsFunction(JsFunction::from(input))
        } else if input.is_symbol() {
            Value::Symbol(JsSymbol::from(input))
        } else if input.is_bigint() {
            Value::BigInt(JsBigInt::from(input))
        } else {
            Value::Null(JsValue::null())
        }
    }
}

impl From<&Value> for JsValue {
    fn from(input: &Value) -> JsValue {
        match input {
            Value::Number(x) => JsValue::from(x),
            Value::Bool(x) => JsValue::from(x),
            Value::Null(_) => JsValue::null(),
            Value::Undefined(_) => JsValue::undefined(),
            Value::String(x) => JsValue::from(x),
            Value::Object(x) => JsValue::from(x),
            Value::Function(x) => JsValue::from(&x.jsfunction),
            Value::JsFunction(x) => JsValue::from(x),
            Value::Symbol(x) => JsValue::from(x),
            Value::BigInt(x) => JsValue::from(x),
        }
    }
}

impl AsRef<JsValue> for Value {
    fn as_ref(&self) -> &JsValue {
        match self {
            Value::Number(x) => x.as_ref(),
            Value::Bool(x) => x.as_ref(),
            Value::Null(x) => x.as_ref(),
            Value::Undefined(x) => x.as_ref(),
            Value::String(x) => x.as_ref(),
            Value::Object(x) => x.as_ref(),
            Value::Function(x) => x.jsfunction.as_ref(),
            Value::JsFunction(x) => x.as_ref(),
            Value::Symbol(x) => x.as_ref(),
            Value::BigInt(x) => x.as_ref(),
        }
    }
}

impl From<Value> for RcValue {
    fn from(val: Value) -> RcValue {
        Rc::new(RefCell::new(val))
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub args: Vec<String>,
    pub body: Vec<Stmt>,
    pub env: ClosedEnvironment,
    pub async_: bool,
    jsfunction: JsValue,
    pub prototype: Option<JsObject>,
}

impl Function {
    pub fn new(
        args: Vec<String>,
        body: Vec<Stmt>,
        env: ClosedEnvironment,
        async_: bool,
        jsfunction: JsValue,
        prototype: Option<JsObject>,
    ) -> Function {
        Function {
            args,
            body,
            env,
            async_,
            jsfunction,
            prototype,
        }
    }
}
