#![recursion_limit = "512"]

mod app;
mod jslib;

use js_sys::Reflect;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn init_page(config: &JsValue) -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());
    yew::initialize();
    let element = Reflect::get(config, &JsValue::from_str("element")).unwrap();
    let elem = yew::utils::document()
        .query_selector(&element.as_string().unwrap())
        .unwrap()
        .unwrap();
    let repo = Reflect::get(config, &JsValue::from_str("repo")).unwrap();
    let branch = Reflect::get(config, &JsValue::from_str("branch")).unwrap();
    let file = Reflect::get(config, &JsValue::from_str("file")).unwrap();
    let query = Reflect::get(config, &JsValue::from_str("query")).unwrap();
    let props = app::ConfigProperties {
        repo: repo.as_string().unwrap(),
        branch: Some(branch.as_string().unwrap()),
        file: file.as_string().unwrap(),
        query: query.as_string().unwrap(),
    };
    yew::App::<app::App>::new().mount_with_props(elem, props);
    yew::run_loop();
    Ok(())
}
