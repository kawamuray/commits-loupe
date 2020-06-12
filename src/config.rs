use js_sys::{Array, Reflect};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub on: String,
    pub repo: String,
    pub branch: Option<String>,
    pub data_url: String,
    pub components: Components,
    pub data: Vec<Data>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Components {
    pub show_table: bool,
    pub show_range: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Data {
    pub title: String,
    pub file: String,
    pub query: String,
}

impl TryFrom<&JsValue> for Config {
    type Error = Error;

    fn try_from(value: &JsValue) -> Result<Self, Self::Error> {
        Ok(Config {
            on: obj_get_type(&value, "on", JsValue::as_string)?,
            repo: obj_get_type(&value, "repo", JsValue::as_string)?,
            branch: obj_get_option_type(&value, "branch", JsValue::as_string)?,
            data_url: obj_get_type(&value, "data_url", JsValue::as_string)?,
            components: Components::try_from(&obj_get(&value, "components")?)?,
            data: Array::from(&obj_get(&value, "data")?)
                .iter()
                .map(|v| Data::try_from(&v))
                .collect::<Result<Vec<_>, Error>>()?,
        })
    }
}

impl TryFrom<&JsValue> for Components {
    type Error = Error;

    fn try_from(value: &JsValue) -> Result<Self, Self::Error> {
        Ok(Components {
            show_table: obj_get_type(&value, "show_table", JsValue::as_bool)?,
            show_range: obj_get_type(&value, "show_range", JsValue::as_bool)?,
        })
    }
}

impl TryFrom<&JsValue> for Data {
    type Error = Error;

    fn try_from(value: &JsValue) -> Result<Self, Self::Error> {
        Ok(Data {
            title: obj_get_type(&value, "title", JsValue::as_string)?,
            file: obj_get_type(&value, "file", JsValue::as_string)?,
            query: obj_get_type(&value, "query", JsValue::as_string)?,
        })
    }
}

fn obj_get_option(o: &JsValue, key: &str) -> Result<Option<JsValue>, Error> {
    let v = Reflect::get(o, &JsValue::from_str(key))?;
    if v.is_null() || v.is_undefined() {
        return Ok(None);
    }
    Ok(Some(v))
}

fn obj_get(o: &JsValue, key: &str) -> Result<JsValue, Error> {
    if let Some(v) = obj_get_option(o, key)? {
        return Ok(v);
    }
    Err(Error::NoPropertyValue(key.to_owned()))
}

fn obj_get_option_type<T, F: FnOnce(&JsValue) -> Option<T>>(
    o: &JsValue,
    key: &str,
    convert_fn: F,
) -> Result<Option<T>, Error> {
    if let Some(v) = obj_get_option(o, key)? {
        if let Some(value) = convert_fn(&v) {
            return Ok(Some(value));
        }
        return Err(Error::InvalidPropertyValue {
            key: key.to_owned(),
            value: v,
        });
    }
    Ok(None)
}

fn obj_get_type<T, F: FnOnce(&JsValue) -> Option<T>>(
    o: &JsValue,
    key: &str,
    convert_fn: F,
) -> Result<T, Error> {
    if let Some(v) = obj_get_option_type(o, key, convert_fn)? {
        return Ok(v);
    }
    Err(Error::NoPropertyValue(key.to_owned()))
}
