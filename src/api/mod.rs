pub mod github;
pub mod static_metadata;

use http::status::StatusCode;
use std::fmt::Debug;
use thiserror::Error;
use yew::services::fetch::FetchTask;

#[derive(Debug, Error, Clone)]
pub enum Error {
    #[error("fetch error: {0}")]
    Fetch(String),
    #[error("http error: {0}")]
    Http(StatusCode),
}

pub trait Api<Req: Debug, Res> {
    fn call<F>(&mut self, req: &Req, callback: F) -> Result<Option<FetchTask>, anyhow::Error>
    where
        F: FnOnce(Result<Res, Error>) + 'static;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommitListRequest {
    pub repo: String,
    pub from: Option<String>,
    pub page: u32,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommitMetadataRequest {
    pub commit: String,
    pub file: String,
}
