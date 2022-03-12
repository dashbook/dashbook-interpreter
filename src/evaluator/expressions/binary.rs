use crate::value::*;
// use resast::prelude::*;
use js_sys::Error;
use swc_ecma_ast::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[inline]
pub(crate) fn eval_binary_expression(
    opr: BinaryOp,
    left: &Value,
    right: &Value,
) -> Result<Value, Error> {
    match opr {
        BinaryOp::Add => eval_plus_operator(left, right),
        BinaryOp::Sub => eval_minus_operator(left, right),
        BinaryOp::Mul => eval_multiplication_operator(left, right),
        BinaryOp::Div => eval_division_operator(left, right),
        BinaryOp::Mod => eval_mod_operator(left, right),
        BinaryOp::Exp => eval_power_operator(left, right),
        BinaryOp::Gt => eval_greater_than_operator(left, right),
        BinaryOp::GtEq => eval_greater_than_equal_operator(left, right),
        BinaryOp::Lt => eval_less_than_operator(left, right),
        BinaryOp::LtEq => eval_less_than_equal_operator(left, right),
        BinaryOp::EqEq => eval_equal_operator(left, right),
        BinaryOp::EqEqEq => eval_equal_equal_operator(left, right),
        BinaryOp::NotEq => eval_not_equal_operator(left, right),
        BinaryOp::NotEqEq => eval_not_equal_equal_operator(left, right),
        BinaryOp::LogicalAnd => eval_and_operator(left, right),
        BinaryOp::LogicalOr => eval_or_operator(left, right),
        BinaryOp::LShift => left_shift_operator(left, right),
        BinaryOp::RShift => right_shift_operator(left, right),
        BinaryOp::ZeroFillRShift => zero_right_shift_operator(left, right),
        BinaryOp::BitAnd => bitwise_and_operator(left, right),
        BinaryOp::BitOr => bitwise_or_operator(left, right),
        BinaryOp::BitXor => bitwise_xor_operator(left, right),
        BinaryOp::In => in_operator(left, right),
        BinaryOp::InstanceOf => instanceof_operator(left, right),
        BinaryOp::NullishCoalescing => nullish_coalescing_operator(left, right),
    }
}

#[inline]
pub(crate) fn eval_plus_operator(left: &Value, right: &Value) -> Result<Value, Error> {
    match (left, right) {
        (Value::Number(x), Value::Number(y)) => {
            Ok(Value::Number(JsNumber::from(x.value_of() + y.value_of())))
        }
        (Value::String(x), Value::Number(y)) => Ok(Value::Number(JsNumber::from(
            x.as_string()
                .ok_or(Error::new("Error: Failed to parse {:?} as number."))?
                .parse::<f64>()
                .unwrap_or(std::f64::NAN)
                + y.value_of(),
        ))),
        (Value::Number(y), Value::String(x)) => Ok(Value::Number(JsNumber::from(
            x.as_string()
                .ok_or(Error::new("Error: Failed to parse {:?} as number."))?
                .parse::<f64>()
                .unwrap_or(std::f64::NAN)
                + y.value_of(),
        ))),
        (Value::String(x), Value::String(y)) => Ok(Value::String(
            x.value_of().concat(&JsValue::from(y.value_of())),
        )),
        (x, y) => Err(Error::from(js_sys::TypeError::new(&format!(
            "TypeError: Plus operator not supported for {} and {}.",
            x, y
        )))),
    }
}

#[inline]
pub(crate) fn eval_minus_operator(left: &Value, right: &Value) -> Result<Value, Error> {
    match (left, right) {
        (Value::Number(x), Value::Number(y)) => {
            Ok(Value::Number(JsNumber::from(x.value_of() - y.value_of())))
        }
        (Value::String(x), Value::Number(y)) => Ok(Value::Number(JsNumber::from(
            x.as_string()
                .ok_or(Error::new("Error: Failed to parse {:?} as number."))?
                .parse::<f64>()
                .unwrap_or(std::f64::NAN)
                - y.value_of(),
        ))),
        (Value::Number(y), Value::String(x)) => Ok(Value::Number(JsNumber::from(
            x.as_string()
                .ok_or(Error::new("Error: Failed to parse {:?} as number."))?
                .parse::<f64>()
                .unwrap_or(std::f64::NAN)
                - y.value_of(),
        ))),
        (x, y) => Err(Error::from(js_sys::TypeError::new(&format!(
            "TypeError: Minus operator not supported for {} and {}.",
            x, y
        )))),
    }
}

#[inline]
pub(crate) fn eval_multiplication_operator(left: &Value, right: &Value) -> Result<Value, Error> {
    match (left, right) {
        (Value::Number(x), Value::Number(y)) => {
            Ok(Value::Number(JsNumber::from(x.value_of() * y.value_of())))
        }
        (Value::String(x), Value::Number(y)) => Ok(Value::Number(JsNumber::from(
            x.as_string()
                .ok_or(Error::new("Error: Failed to parse {:?} as number."))?
                .parse::<f64>()
                .unwrap_or(std::f64::NAN)
                * y.value_of(),
        ))),
        (Value::Number(y), Value::String(x)) => Ok(Value::Number(JsNumber::from(
            x.as_string()
                .ok_or(Error::new("Error: Failed to parse {:?} as number."))?
                .parse::<f64>()
                .unwrap_or(std::f64::NAN)
                * y.value_of(),
        ))),
        (x, y) => Err(Error::from(js_sys::TypeError::new(&format!(
            "TypeError: Multiplication operator not supported for {} and {}.",
            x, y
        )))),
    }
}

#[inline]
pub(crate) fn eval_division_operator(left: &Value, right: &Value) -> Result<Value, Error> {
    match (left, right) {
        (Value::Number(x), Value::Number(y)) => {
            Ok(Value::Number(JsNumber::from(x.value_of() / y.value_of())))
        }
        (Value::String(x), Value::Number(y)) => Ok(Value::Number(JsNumber::from(
            x.as_string()
                .ok_or(Error::new("Error: Failed to parse {:?} as number."))?
                .parse::<f64>()
                .unwrap_or(std::f64::NAN)
                / y.value_of(),
        ))),
        (Value::Number(y), Value::String(x)) => Ok(Value::Number(JsNumber::from(
            x.as_string()
                .ok_or(Error::new("Error: Failed to parse {:?} as number."))?
                .parse::<f64>()
                .unwrap_or(std::f64::NAN)
                / y.value_of(),
        ))),
        (x, y) => Err(Error::from(js_sys::TypeError::new(&format!(
            "TypeError: Division operator not supported for {} and {}.",
            x, y
        )))),
    }
}

#[inline]
pub(crate) fn eval_mod_operator(left: &Value, right: &Value) -> Result<Value, Error> {
    match (left, right) {
        (Value::Number(x), Value::Number(y)) => {
            Ok(Value::Number(JsNumber::from(x.value_of() % y.value_of())))
        }
        (Value::String(x), Value::Number(y)) => Ok(Value::Number(JsNumber::from(
            x.as_string()
                .ok_or(Error::new("Error: Failed to parse {:?} as number."))?
                .parse::<f64>()
                .unwrap_or(std::f64::NAN)
                % y.value_of(),
        ))),
        (Value::Number(y), Value::String(x)) => Ok(Value::Number(JsNumber::from(
            x.as_string()
                .ok_or(Error::new("Error: Failed to parse {:?} as number."))?
                .parse::<f64>()
                .unwrap_or(std::f64::NAN)
                % y.value_of(),
        ))),
        _ => Ok(Value::Number(JsNumber::from(std::f64::NAN))),
    }
}

#[inline]
pub(crate) fn eval_power_operator(left: &Value, right: &Value) -> Result<Value, Error> {
    match (left, right) {
        (Value::Number(x), Value::Number(y)) => Ok(Value::Number(JsNumber::from(
            x.value_of().powf(y.value_of()),
        ))),
        (x, y) => Err(Error::from(js_sys::TypeError::new(&format!(
            "TypeError: Power operator not supported for {} and {}.",
            x, y
        )))),
    }
}

#[inline]
pub(crate) fn eval_greater_than_operator(left: &Value, right: &Value) -> Result<Value, Error> {
    match (left, right) {
        (Value::Number(x), Value::Number(y)) => {
            Ok(Value::Bool(JsBool::from(x.value_of() > y.value_of())))
        }
        (x, y) => Err(Error::from(js_sys::TypeError::new(&format!(
            "TypeError: Greater than operator not supported for {} and {}.",
            x, y
        )))),
    }
}

#[inline]
pub(crate) fn eval_greater_than_equal_operator(
    left: &Value,
    right: &Value,
) -> Result<Value, Error> {
    match (left, right) {
        (Value::Number(x), Value::Number(y)) => {
            Ok(Value::Bool(JsBool::from(x.value_of() >= y.value_of())))
        }
        (x, y) => Err(Error::from(js_sys::TypeError::new(&format!(
            "TypeError: Greater than equal operator not supported for {} and {}.",
            x, y
        )))),
    }
}

#[inline]
pub(crate) fn eval_less_than_operator(left: &Value, right: &Value) -> Result<Value, Error> {
    match (left, right) {
        (Value::Number(x), Value::Number(y)) => {
            Ok(Value::Bool(JsBool::from(x.value_of() < y.value_of())))
        }
        (x, y) => Err(Error::from(js_sys::TypeError::new(&format!(
            "TypeError: Less than operator not supported for {} and {}.",
            x, y
        )))),
    }
}

#[inline]
pub(crate) fn eval_less_than_equal_operator(left: &Value, right: &Value) -> Result<Value, Error> {
    match (left, right) {
        (Value::Number(x), Value::Number(y)) => {
            Ok(Value::Bool(JsBool::from(x.value_of() <= y.value_of())))
        }
        (x, y) => Err(Error::from(js_sys::TypeError::new(&format!(
            "TypeError: Less than equal operator not supported for {} and {}.",
            x, y
        )))),
    }
}

#[inline]
pub(crate) fn eval_equal_operator(left: &Value, right: &Value) -> Result<Value, Error> {
    match (left, right) {
        (Value::Number(x), Value::Number(y)) => {
            Ok(Value::Bool(JsBool::from(x.value_of() == y.value_of())))
        }
        (Value::Bool(x), Value::Bool(y)) => {
            Ok(Value::Bool(JsBool::from(x.value_of() == y.value_of())))
        }
        (Value::String(x), Value::String(y)) => {
            Ok(Value::Bool(JsBool::from(x.value_of() == y.value_of())))
        }
        (Value::Undefined(_), Value::Undefined(_)) => Ok(Value::Bool(JsBool::from(true))),
        (Value::Null(_), Value::Null(_)) => Ok(Value::Bool(JsBool::from(true))),
        (Value::Null(_), Value::Undefined(_)) => Ok(Value::Bool(JsBool::from(true))),
        (Value::Undefined(_), Value::Null(_)) => Ok(Value::Bool(JsBool::from(true))),
        (Value::Bool(x), Value::Number(y)) => Ok(Value::Bool(JsBool::from(if x.value_of() {
            y.value_of() == 1.0
        } else {
            y.value_of() == 0.0
        }))),
        (Value::Number(y), Value::Bool(x)) => Ok(Value::Bool(JsBool::from(if x.value_of() {
            y.value_of() == 1.0
        } else {
            y.value_of() == 0.0
        }))),
        (Value::String(x), Value::Number(y)) => Ok(Value::Bool(JsBool::from(
            x.as_string()
                .ok_or(Error::new("Error: Failed to parse {:?} as number."))?
                .parse::<f64>()
                .map_err(|z| Error::new(&format!("{}", z)))?
                == y.value_of(),
        ))),
        (Value::Number(y), Value::String(x)) => Ok(Value::Bool(JsBool::from(
            x.as_string()
                .ok_or(Error::new("Error: Failed to parse {:?} as number."))?
                .parse::<f64>()
                .map_err(|z| Error::new(&format!("{}", z)))?
                == y.value_of(),
        ))),
        (Value::Object(x), Value::Object(y)) => {
            Ok(Value::Bool(JsBool::from(js_sys::Object::is(&x, &y))))
        }
        (Value::JsFunction(x), Value::JsFunction(y)) => {
            Ok(Value::Bool(JsBool::from(js_sys::Object::is(&x, &y))))
        }
        (Value::Object(x), Value::JsFunction(y)) => {
            Ok(Value::Bool(JsBool::from(js_sys::Object::is(&x, &y))))
        }
        (Value::JsFunction(x), Value::Object(y)) => {
            Ok(Value::Bool(JsBool::from(js_sys::Object::is(&x, &y))))
        }
        (x, y) => Err(Error::from(js_sys::TypeError::new(&format!(
            "TypeError: Equal operator not supported for {} and {}.",
            x, y
        )))),
    }
}

#[inline]
pub(crate) fn eval_not_equal_operator(left: &Value, right: &Value) -> Result<Value, Error> {
    match (left, right) {
        (Value::Number(x), Value::Number(y)) => {
            Ok(Value::Bool(JsBool::from(x.value_of() != y.value_of())))
        }
        (Value::Bool(x), Value::Bool(y)) => {
            Ok(Value::Bool(JsBool::from(x.value_of() != y.value_of())))
        }
        (Value::String(x), Value::String(y)) => {
            Ok(Value::Bool(JsBool::from(x.value_of() != y.value_of())))
        }
        (Value::Undefined(_), Value::Undefined(_)) => Ok(Value::Bool(JsBool::from(false))),
        (Value::Null(_), Value::Null(_)) => Ok(Value::Bool(JsBool::from(false))),
        (Value::Null(_), Value::Undefined(_)) => Ok(Value::Bool(JsBool::from(false))),
        (Value::Undefined(_), Value::Null(_)) => Ok(Value::Bool(JsBool::from(false))),
        (Value::Bool(x), Value::Number(y)) => Ok(Value::Bool(JsBool::from(if x.value_of() {
            y.value_of() != 1.0
        } else {
            y.value_of() != 0.0
        }))),
        (Value::Number(y), Value::Bool(x)) => Ok(Value::Bool(JsBool::from(if x.value_of() {
            y.value_of() != 1.0
        } else {
            y.value_of() != 0.0
        }))),
        (Value::String(x), Value::Number(y)) => Ok(Value::Bool(JsBool::from(
            x.as_string()
                .ok_or(Error::new("Error: Failed to parse {:?} as number."))?
                .parse::<f64>()
                .map_err(|z| Error::new(&format!("{}", z)))?
                != y.value_of(),
        ))),
        (Value::Number(y), Value::String(x)) => Ok(Value::Bool(JsBool::from(
            x.as_string()
                .ok_or(Error::new("Error: Failed to parse {:?} as number."))?
                .parse::<f64>()
                .map_err(|z| Error::new(&format!("{}", z)))?
                != y.value_of(),
        ))),
        (Value::Object(x), Value::Object(y)) => {
            Ok(Value::Bool(JsBool::from(!js_sys::Object::is(&x, &y))))
        }
        (Value::JsFunction(x), Value::JsFunction(y)) => {
            Ok(Value::Bool(JsBool::from(!js_sys::Object::is(&x, &y))))
        }
        (Value::Object(x), Value::JsFunction(y)) => {
            Ok(Value::Bool(JsBool::from(!js_sys::Object::is(&x, &y))))
        }
        (Value::JsFunction(x), Value::Object(y)) => {
            Ok(Value::Bool(JsBool::from(!js_sys::Object::is(&x, &y))))
        }
        (x, y) => Err(Error::from(js_sys::TypeError::new(&format!(
            "TypeError: Not equal operator not supported for {} and {}.",
            x, y
        )))),
    }
}

#[inline]
pub(crate) fn eval_equal_equal_operator(left: &Value, right: &Value) -> Result<Value, Error> {
    match (left, right) {
        (Value::Number(x), Value::Number(y)) => {
            Ok(Value::Bool(JsBool::from(x.value_of() == y.value_of())))
        }
        (Value::Bool(x), Value::Bool(y)) => {
            Ok(Value::Bool(JsBool::from(x.value_of() == y.value_of())))
        }
        (Value::String(x), Value::String(y)) => {
            Ok(Value::Bool(JsBool::from(x.value_of() == y.value_of())))
        }
        (Value::Undefined(_), Value::Undefined(_)) => Ok(Value::Bool(JsBool::from(true))),
        (Value::Null(_), Value::Null(_)) => Ok(Value::Bool(JsBool::from(true))),
        (Value::Object(x), Value::Object(y)) => {
            Ok(Value::Bool(JsBool::from(js_sys::Object::is(&x, &y))))
        }
        (Value::JsFunction(x), Value::JsFunction(y)) => {
            Ok(Value::Bool(JsBool::from(js_sys::Object::is(&x, &y))))
        }
        (Value::Object(x), Value::JsFunction(y)) => {
            Ok(Value::Bool(JsBool::from(js_sys::Object::is(&x, &y))))
        }
        (Value::JsFunction(x), Value::Object(y)) => {
            Ok(Value::Bool(JsBool::from(js_sys::Object::is(&x, &y))))
        }
        _ => Ok(Value::Bool(JsBool::from(false))),
    }
}

#[inline]
pub(crate) fn eval_not_equal_equal_operator(left: &Value, right: &Value) -> Result<Value, Error> {
    match (left, right) {
        (Value::Number(x), Value::Number(y)) => {
            Ok(Value::Bool(JsBool::from(x.value_of() != y.value_of())))
        }
        (Value::Bool(x), Value::Bool(y)) => {
            Ok(Value::Bool(JsBool::from(x.value_of() != y.value_of())))
        }
        (Value::String(x), Value::String(y)) => {
            Ok(Value::Bool(JsBool::from(x.value_of() != y.value_of())))
        }
        (Value::Undefined(_), Value::Undefined(_)) => Ok(Value::Bool(JsBool::from(false))),
        (Value::Null(_), Value::Null(_)) => Ok(Value::Bool(JsBool::from(false))),
        (Value::Object(x), Value::Object(y)) => {
            Ok(Value::Bool(JsBool::from(!js_sys::Object::is(&x, &y))))
        }
        (Value::JsFunction(x), Value::JsFunction(y)) => {
            Ok(Value::Bool(JsBool::from(!js_sys::Object::is(&x, &y))))
        }
        (Value::Object(x), Value::JsFunction(y)) => {
            Ok(Value::Bool(JsBool::from(!js_sys::Object::is(&x, &y))))
        }
        (Value::JsFunction(x), Value::Object(y)) => {
            Ok(Value::Bool(JsBool::from(!js_sys::Object::is(&x, &y))))
        }
        _ => Ok(Value::Bool(JsBool::from(true))),
    }
}

#[inline]
pub(crate) fn eval_and_operator(left: &Value, right: &Value) -> Result<Value, Error> {
    match (left, right) {
        (Value::Bool(x), Value::Bool(y)) => {
            Ok(Value::Bool(JsBool::from(x.value_of() && y.value_of())))
        }
        (x, y) => Err(Error::from(js_sys::TypeError::new(&format!(
            "TypeError: And operator not supported for {} and {}.",
            x, y
        )))),
    }
}

#[inline]
pub(crate) fn eval_or_operator(left: &Value, right: &Value) -> Result<Value, Error> {
    match (left, right) {
        (Value::Bool(x), Value::Bool(y)) => {
            Ok(Value::Bool(JsBool::from(x.value_of() || y.value_of())))
        }
        (Value::Bool(x), _) => {
            if x.value_of() == false {
                Ok(right.clone())
            } else {
                Ok(left.clone())
            }
        }
        (Value::Null(_), _) => Ok(right.clone()),
        (Value::Undefined(_), _) => Ok(right.clone()),
        (Value::Number(x), _) => {
            if js_sys::Number::is_nan(x) || x.value_of() == 0.0 {
                Ok(right.clone())
            } else {
                Ok(left.clone())
            }
        }
        (Value::String(x), _) => {
            if x.length() == 0 {
                Ok(right.clone())
            } else {
                Ok(left.clone())
            }
        }
        (x, y) => Err(Error::from(js_sys::TypeError::new(&format!(
            "TypeError: Or operator not supported for {} and {}.",
            x, y
        )))),
    }
}

#[inline]
pub(crate) fn left_shift_operator(left: &Value, right: &Value) -> Result<Value, Error> {
    match (left, right) {
        (Value::Number(x), Value::Number(y)) => Ok(Value::Number(JsNumber::from(
            ((x.value_of() as i64) << (y.value_of() as i64)) as f64,
        ))),
        (x, y) => Err(Error::from(js_sys::TypeError::new(&format!(
            "TypeError: Left shift operator not supported for {} and {}.",
            x, y
        )))),
    }
}

#[inline]
pub(crate) fn right_shift_operator(left: &Value, right: &Value) -> Result<Value, Error> {
    match (left, right) {
        (Value::Number(x), Value::Number(y)) => Ok(Value::Number(JsNumber::from(
            ((x.value_of() as i64) >> (y.value_of() as i64)) as f64,
        ))),
        (x, y) => Err(Error::from(js_sys::TypeError::new(&format!(
            "TypeError: Right shift operator not supported for {} and {}.",
            x, y
        )))),
    }
}

#[inline]
pub(crate) fn zero_right_shift_operator(left: &Value, right: &Value) -> Result<Value, Error> {
    match (left, right) {
        (Value::Number(x), Value::Number(y)) => Ok(Value::Number(JsNumber::from(
            ((x.value_of() as u64) >> (y.value_of() as i64)) as f64,
        ))),
        (x, y) => Err(Error::from(js_sys::TypeError::new(&format!(
            "TypeError: Unsigned right shift operator not supported for {} and {}.",
            x, y
        )))),
    }
}

#[inline]
pub(crate) fn bitwise_and_operator(left: &Value, right: &Value) -> Result<Value, Error> {
    match (left, right) {
        (Value::Number(x), Value::Number(y)) => Ok(Value::Number(JsNumber::from(
            ((x.value_of() as i64) & (y.value_of() as i64)) as f64,
        ))),
        (x, y) => Err(Error::from(js_sys::TypeError::new(&format!(
            "TypeError: Bitwise and operator not supported for {} and {}.",
            x, y
        )))),
    }
}

#[inline]
pub(crate) fn bitwise_or_operator(left: &Value, right: &Value) -> Result<Value, Error> {
    match (left, right) {
        (Value::Number(x), Value::Number(y)) => Ok(Value::Number(JsNumber::from(
            ((x.value_of() as i64) | (y.value_of() as i64)) as f64,
        ))),
        (x, y) => Err(Error::from(js_sys::TypeError::new(&format!(
            "TypeError: Bitwise or operator not supported for {} and {}.",
            x, y
        )))),
    }
}

#[inline]
pub(crate) fn bitwise_xor_operator(left: &Value, right: &Value) -> Result<Value, Error> {
    match (left, right) {
        (Value::Number(x), Value::Number(y)) => Ok(Value::Number(JsNumber::from(
            ((x.value_of() as i64) ^ (y.value_of() as i64)) as f64,
        ))),
        (x, y) => Err(Error::from(js_sys::TypeError::new(&format!(
            "TypeError: Bitwise xor operator not supported for {} and {}.",
            x, y
        )))),
    }
}

#[inline]
pub(crate) fn in_operator(left: &Value, right: &Value) -> Result<Value, Error> {
    match (left, right) {
        (Value::Object(x), y) => Ok(Value::Bool(JsBool::from(
            js_sys::Reflect::has(&JsValue::from(x), &JsValue::from(y))
                .map_err(|x| Error::from(x))?,
        ))),
        (x, y) => Err(Error::from(js_sys::TypeError::new(&format!(
            "TypeError: In operator not supported for {} and {}.",
            x, y
        )))),
    }
}

#[inline]
pub(crate) fn instanceof_operator(left: &Value, right: &Value) -> Result<Value, Error> {
    match (left, right) {
        (x, Value::Bool(_)) => Ok(Value::Bool(JsBool::from(
            x.as_ref().is_instance_of::<js_sys::Boolean>(),
        ))),
        (x, Value::Number(_)) => Ok(Value::Bool(JsBool::from(
            x.as_ref().is_instance_of::<js_sys::Number>(),
        ))),
        (x, Value::String(_)) => Ok(Value::Bool(JsBool::from(
            x.as_ref().is_instance_of::<js_sys::JsString>(),
        ))),
        (x, Value::Object(_)) => Ok(Value::Bool(JsBool::from(
            x.as_ref().is_instance_of::<js_sys::Object>(),
        ))),
        (x, Value::JsFunction(_)) => Ok(Value::Bool(JsBool::from(
            x.as_ref().is_instance_of::<js_sys::Function>(),
        ))),
        (x, y) => Err(Error::from(js_sys::TypeError::new(&format!(
            "TypeError: Instanceof operator not supported for {} and {}.",
            x, y
        )))),
    }
}

#[inline]
pub(crate) fn nullish_coalescing_operator(left: &Value, right: &Value) -> Result<Value, Error> {
    match (left, right) {
        (Value::Null(_), y) => Ok(y.clone()),
        (Value::Undefined(_), y) => Ok(y.clone()),
        (x, _) => Ok(x.clone()),
    }
}
