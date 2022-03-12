use crate::value::*;
use js_sys::Error;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::slice::{Iter, IterMut};
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct Environments {
    pub stack: Vec<HashMap<String, Rc<RefCell<Value>>>>,
}

impl Environments {
    pub fn new() -> Environments {
        let mut envs = Environments {
            stack: vec![HashMap::new()],
        };
        let global = js_sys::global();
        let global_proxy = create_global_proxy(&global);
        let global_this = Rc::new(RefCell::new(Value::from(JsValue::from(global_proxy))));
        envs.insert("globalThis", global_this.clone()).unwrap();
        envs.insert("self", global_this.clone()).unwrap();
        envs.insert("window", global_this.clone()).unwrap();
        envs.insert("this", global_this).unwrap();

        let document_proxy = create_document_proxy(&global).unwrap();
        let document = Rc::new(RefCell::new(Value::from(JsValue::from(document_proxy))));
        envs.insert("document", document).unwrap();
        envs
    }

    pub fn empty() -> Environments {
        Environments { stack: Vec::new() }
    }
}

#[inline]
fn create_global_proxy(global: &js_sys::Object) -> js_sys::Proxy {
    let handler = js_sys::Object::new();
    let set =
        js_sys::Function::new_with_args("target, propKey, value, receiver", "return undefined;");
    let get = js_sys::Function::new_with_args(
        "target, propKey, receiver",
        "if (propKey === \"console\"
            || propKey === \"undefined\" 
            || propKey === \"fetch\" 
            || propKey === \"Number\"
            || propKey === \"Boolean\"
            || propKey === \"Object\"
            || propKey === \"Function\"
            || propKey === \"String\"
            || propKey === \"Generator\"
            || propKey === \"Iterator\"
            || propKey === \"Symbol\"
            || propKey === \"Math\"
            || propKey === \"Date\"
            || propKey === \"RegExp\"
            || propKey === \"Array\"
            || propKey === \"Int8Array\"
            || propKey === \"Uint8Array\"
            || propKey === \"Uint8ClampedArray\"
            || propKey === \"Int16Array\"
            || propKey === \"Uint16Array\"
            || propKey === \"Int32Array\"
            || propKey === \"Uint32Array\"
            || propKey === \"Float32Array\"
            || propKey === \"Float64Array\"
            || propKey === \"BigInt64Array\"
            || propKey === \"BigUint64Array\"
            || propKey === \"Map\"
            || propKey === \"Set\"
            || propKey === \"WeakMap\"
            || propKey === \"WeakSet\"
            || propKey === \"ArrayBuffer\"
            || propKey === \"SharedArrayBuffer\"
            || propKey === \"Error\"
            || propKey === \"EvalError\"
            || propKey === \"RangeError\"
            || propKey === \"ReferenceError\"
            || propKey === \"SyntaxError\"
            || propKey === \"TypeError\"
            || propKey === \"DataView\"
            || propKey === \"Promise\"
            || propKey === \"JSON\"
            || propKey === \"parseFloat\"
            || propKey === \"parseInt\"
            || propKey === \"NaN\"
            || propKey === \"isNaN\"
            || propKey === \"Infinity\"
            || propKey === \"isFinite\"
            || propKey === \"ImageData\"
            || propKey === \"URL\"
            || propKey === \"createImageBitmap\"
            || propKey === \"setTimeout\"
            || propKey === \"clearTimeout\"
            || propKey === \"setInterval\"
            || propKey === \"clearInterval\"
            || propKey === \"TextEncoder\"
            || propKey === \"TextDecoder\"
        ) {return Reflect.get(...arguments);} return undefined;",
    );
    let apply =
        js_sys::Function::new_with_args("target, thisArgument, argumentsList", "return undefined;");
    let construct =
        js_sys::Function::new_with_args("target, argumentsList, newTarget", "return undefined;");
    let define_prop =
        js_sys::Function::new_with_args("target, propKey, propDesc", "return undefined;");
    let delete_prop = js_sys::Function::new_with_args("target, propKey", "return undefined;");
    js_sys::Reflect::set(
        &JsValue::from(&handler),
        &JsValue::from_str("set"),
        &JsValue::from(set),
    )
    .unwrap();
    js_sys::Reflect::set(
        &JsValue::from(&handler),
        &JsValue::from_str("get"),
        &JsValue::from(get),
    )
    .unwrap();
    js_sys::Reflect::set(
        &JsValue::from(&handler),
        &JsValue::from_str("apply"),
        &JsValue::from(apply),
    )
    .unwrap();
    js_sys::Reflect::set(
        &JsValue::from(&handler),
        &JsValue::from_str("construct"),
        &JsValue::from(construct),
    )
    .unwrap();
    js_sys::Reflect::set(
        &JsValue::from(&handler),
        &JsValue::from_str("defineProperty"),
        &JsValue::from(define_prop),
    )
    .unwrap();
    js_sys::Reflect::set(
        &JsValue::from(&handler),
        &JsValue::from_str("deleteProperty"),
        &JsValue::from(delete_prop),
    )
    .unwrap();
    js_sys::Proxy::new(global, &handler)
}

#[inline]
fn create_document_proxy(global: &js_sys::Object) -> Result<js_sys::Proxy, Error> {
    let document =
        js_sys::Reflect::get(global, &JsValue::from_str("document")).map_err(|x| Error::from(x))?;
    let handler = js_sys::Object::new();
    let set =
        js_sys::Function::new_with_args("target, propKey, value, receiver", "return undefined;");
    let get = js_sys::Function::new_with_args(
        "target, propKey, receiver",
        "if (propKey === \"createElement\") {return Reflect.get(...arguments).bind(document);} return undefined;",
    );
    let apply =
        js_sys::Function::new_with_args("target, thisArgument, argumentsList", "return undefined;");
    let construct =
        js_sys::Function::new_with_args("target, argumentsList, newTarget", "return undefined;");
    let define_prop =
        js_sys::Function::new_with_args("target, propKey, propDesc", "return undefined;");
    let delete_prop = js_sys::Function::new_with_args("target, propKey", "return undefined;");
    js_sys::Reflect::set(
        &JsValue::from(&handler),
        &JsValue::from_str("set"),
        &JsValue::from(set),
    )
    .unwrap();
    js_sys::Reflect::set(
        &JsValue::from(&handler),
        &JsValue::from_str("get"),
        &JsValue::from(get),
    )
    .unwrap();
    js_sys::Reflect::set(
        &JsValue::from(&handler),
        &JsValue::from_str("apply"),
        &JsValue::from(apply),
    )
    .unwrap();
    js_sys::Reflect::set(
        &JsValue::from(&handler),
        &JsValue::from_str("construct"),
        &JsValue::from(construct),
    )
    .unwrap();
    js_sys::Reflect::set(
        &JsValue::from(&handler),
        &JsValue::from_str("defineProperty"),
        &JsValue::from(define_prop),
    )
    .unwrap();
    js_sys::Reflect::set(
        &JsValue::from(&handler),
        &JsValue::from_str("deleteProperty"),
        &JsValue::from(delete_prop),
    )
    .unwrap();
    Ok(js_sys::Proxy::new(&document, &handler))
}

impl Environments {
    pub fn insert(
        &mut self,
        key: &str,
        obj: Rc<RefCell<Value>>,
    ) -> Result<Rc<RefCell<Value>>, Error> {
        self.stack
            .last_mut()
            .ok_or(Error::new(&format!("ERROR: No Environment available.")))
            .map(|x| {
                x.insert(String::from(key), obj.clone());
                obj
            })
    }

    pub fn get(&self, key: &str) -> Result<Rc<RefCell<Value>>, Error> {
        Environments::get_i(self.stack.iter(), key).or({
            let global_this = Environments::get_i(self.stack.iter(), "globalThis")?;
            let temp = js_sys::Reflect::get(global_this.borrow().as_ref(), &JsValue::from_str(key))
                .map_err(|x| Error::from(x));
            temp.and_then(|x| Ok(Rc::new(RefCell::new(Value::from(x)))))
        })
    }

    fn get_i<'a>(
        mut iter: Iter<'a, HashMap<String, Rc<RefCell<Value>>>>,
        key: &str,
    ) -> Result<Rc<RefCell<Value>>, Error> {
        iter.nth_back(0)
            .ok_or(Error::new(&format!("ERROR: Variable {:?} not found.", key)))?
            .get(&String::from(key))
            .map(|x| Rc::clone(&x))
            .ok_or(Error::new(&format!("ERROR: Variable {:?} not found.", key)))
            .or(Environments::get_i(iter, key))
    }

    pub fn set<'a>(
        &'a mut self,
        key: &str,
        obj: Rc<RefCell<Value>>,
    ) -> Result<Rc<RefCell<Value>>, Error> {
        Environments::set_i(self.stack.iter_mut(), key, obj.clone())
            .or::<Error>(self.insert(key, obj))
    }

    fn set_i<'a>(
        mut iter_mut: IterMut<'a, HashMap<String, Rc<RefCell<Value>>>>,
        key: &str,
        obj: Rc<RefCell<Value>>,
    ) -> Result<Rc<RefCell<Value>>, Error> {
        let hm = iter_mut.nth_back(0).ok_or(JsValue::from_str(&format!(
            "ERROR: Variable {:?} not found.",
            key
        )))?;
        if hm.contains_key(&String::from(key)) {
            match &*(obj.clone()).borrow() {
                Value::Object(_) => hm
                    .insert(String::from(key), obj)
                    .ok_or(Error::new(&format!("ERROR: Variable {:?} not found.", key))),
                _ => hm
                    .get(&String::from(key))
                    .ok_or(Error::new(&format!("ERROR: Variable {:?} not found.", key)))
                    .map(|x| {
                        *x.borrow_mut() = obj.borrow().clone();
                        obj
                    }),
            }
        } else {
            Environments::set_i(iter_mut, key, obj)
        }
    }

    pub fn closure(&self) -> ClosedEnvironment {
        let mut env = ClosedEnvironment::new();
        self.stack
            .iter()
            .for_each(|x| x.iter().for_each(|(k, v)| env.insert(k, v.clone())));
        env
    }

    pub fn from_closed_env(env: ClosedEnvironment) -> Environments {
        Environments {
            stack: vec![env.bindings],
        }
    }

    pub fn push_env(&mut self) {
        self.stack.push(HashMap::new())
    }

    pub fn pop_env(&mut self) {
        self.stack.pop();
    }
}

#[derive(Debug, Clone)]
pub struct ClosedEnvironment {
    bindings: HashMap<String, Rc<RefCell<Value>>>,
}

impl ClosedEnvironment {
    pub fn new() -> ClosedEnvironment {
        ClosedEnvironment {
            bindings: HashMap::new(),
        }
    }
    pub fn insert(&mut self, key: &str, obj: Rc<RefCell<Value>>) {
        self.bindings.insert(String::from(key), obj);
    }
    pub fn get(&self, key: &str) -> Result<Rc<RefCell<Value>>, JsValue> {
        self.bindings
            .get(key)
            .map(|x| Rc::clone(x))
            .ok_or(JsValue::from_str(&format!(
                "ERROR: {:} not found in function environment.",
                key
            )))
            .or({
                let global_this =
                    self.bindings
                        .get("globalThis")
                        .ok_or(JsValue::from_str(&format!(
                            "ERROR: globalThis not found in function environment."
                        )))?;
                let temp =
                    js_sys::Reflect::get(global_this.borrow().as_ref(), &JsValue::from_str(key));
                temp.and_then(|x| Ok(Rc::new(RefCell::new(Value::from(x)))))
            })
    }

    pub fn bindings(&self) -> &HashMap<String, Rc<RefCell<Value>>> {
        &self.bindings
    }
}
