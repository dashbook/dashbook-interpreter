use js_sys::Error;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/js/includes.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name=importjs)]
    pub async fn import_js(input: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name=importDanfoJS)]
    pub async fn import_danfojs() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name=importTFJS_VIS)]
    pub async fn import_tfjs_vis() -> Result<JsValue, JsValue>;
}

pub async fn import(input: &str) -> Result<JsValue, Error> {
    if input == "@tensorflow/tfjs-vis" {
        import_tfjs_vis().await.map_err(|x| Error::from(x))
    } else if input == "danfojs" {
        import_danfojs().await.map_err(|x| Error::from(x))
    } else if input.starts_with("https://cdn.skypack.dev")
        || input.starts_with("https://cdn.jsdelivr.net")
    {
        import_js(input).await.map_err(|x| Error::from(x))
    } else {
        let mut temp = String::from("https://cdn.skypack.dev/");
        temp.push_str(input);
        import_js(&temp).await.map_err(|x| Error::from(x))
    }
}
