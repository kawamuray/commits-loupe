use serde::{Deserialize, Serialize};

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
