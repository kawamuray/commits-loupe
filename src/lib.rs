#![recursion_limit = "512"]

#[macro_use]
mod js_macros;
mod api;
mod cache;
mod chart;
mod commit;
mod component;
mod config;
mod dataset;
mod query;
mod range;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn create(config: JsValue) -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());
    yew::initialize();
    let config: config::Config = serde_wasm_bindgen::from_value(config)?;
    let elem = yew::utils::document()
        .query_selector(&config.on)
        .unwrap()
        .unwrap();
    let props = component::loupe::Properties { config };
    yew::App::<component::loupe::LoupeComponent<chart::chartjs::ChartJs>>::new()
        .mount_with_props(elem, props);
    Ok(())
}
