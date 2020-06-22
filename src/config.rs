use js_sys::{Array, Reflect};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use thiserror::Error;
use wasm_bindgen::prelude::*;

#[derive(Debug, Error)]
pub enum Error {
    #[error("no valid property value for: {0}")]
    NoPropertyValue(String),
    #[error("invalid property value for {}: {:?}", key, value)]
    InvalidPropertyValue { key: String, value: JsValue },
    #[error("js error: {:?}", 0)]
    JsError(JsValue),
}

impl From<JsValue> for Error {
    fn from(v: JsValue) -> Self {
        Error::JsError(v)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub on: String,
    pub repo: String,
    pub branch: Option<String>,
    pub data_url: String,
    pub components: Components,
    pub data: Vec<Data>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Components {
    pub show_table: bool,
    pub show_range: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Data {
    pub title: String,
    pub file: String,
    pub query: String,
}
