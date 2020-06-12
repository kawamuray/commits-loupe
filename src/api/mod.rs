pub mod commit_metadata;
pub mod github;

use crate::commit::CommitInfo;
use anyhow;
use http::status::StatusCode;
use thiserror::Error;
use yew::services::fetch::FetchTask;

#[derive(Debug, Error)]
pub enum Error {
    #[error("fetch error: {0}")]
    Fetch(anyhow::Error),
    #[error("http error: {0}")]
    Http(StatusCode),
}

pub trait CommitsApi {
    fn commits<F>(
        &mut self,
        repo: &str,
        from: Option<&str>,
        page: u32,
        count: u32,
        callback: F,
    ) -> FetchTask
    where
        F: FnOnce(Result<Vec<CommitInfo>, Error>) + 'static;
}

pub trait MetadataApi {
    fn commit_metadata<F>(&mut self, commit: &str, file: &str, callback: F) -> FetchTask
    where
        F: FnOnce(Result<String, Error>) + 'static;
}
