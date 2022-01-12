use js_sys::Error;
use wasm_bindgen::prelude::*;

pub fn get_iterator(obj: &JsValue) -> Result<js_sys::Iterator, JsValue> {
    Ok(js_sys::Iterator::from(
        js_sys::Function::from(
            js_sys::Reflect::get(obj, &js_sys::Symbol::iterator()).or(Err(Error::new(
                &format!("Error: The object in the for ... of loop is not iterable."),
            )))?,
        )
        .call0(obj)?,
    ))
}
