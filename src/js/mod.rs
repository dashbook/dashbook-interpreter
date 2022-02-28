use js_sys::Error;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(
    inline_js = "export function importjs(str){return import(/* webpackIgnore: true */str)}"
)]
extern "C" {
    #[wasm_bindgen(catch, js_name=importjs)]
    pub async fn import_js(input: &str) -> Result<JsValue, JsValue>;
}

pub async fn import(input: &str) -> Result<JsValue, Error> {
    if input.starts_with("https://cdn.skypack.dev") || input.starts_with("https://cdn.jsdelivr.net")
    {
        import_js(input).await.map_err(|x| Error::from(x))
    } else {
        let mut temp = String::from("https://cdn.skypack.dev/");
        temp.push_str(input);
        import_js(&temp).await.map_err(|x| Error::from(x))
    }
}
