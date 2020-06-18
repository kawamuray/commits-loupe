use crate::api::{Error, MetadataApi};
use log::*;
use yew::callback::Callback;
use yew::format::Nothing;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};

pub struct CommitMetadataApi {
    service: FetchService,
    data_path: String,
}

impl CommitMetadataApi {
    pub fn new(data_path: String) -> Self {
        Self {
            service: FetchService::new(),
            data_path,
        }
    }

    fn build_url(&self, commit: &str, file: &str) -> String {
        // TODO: think what to do for this
        format!(
            "https://kawamuray.github.io/decaton/{}/{}/{}",
            self.data_path, commit, file
        )
        // format!("/decaton/{}/{}/{}", self.data_path, commit, file)
    }
}

impl MetadataApi for CommitMetadataApi {
    fn commit_metadata<F>(&mut self, commit: &str, file: &str, callback: F) -> Option<FetchTask>
    where
        F: FnOnce(Result<String, Error>) + 'static,
    {
        let url = self.build_url(commit, file);
        let request = Request::get(&url)
            .body(Nothing)
            .expect("build request error");
        Some(
            self.service
                .fetch(
                    request,
                    Callback::once(move |resp: Response<Result<String, anyhow::Error>>| {
                        let (meta, data) = resp.into_parts();
                        debug!(
                            "Received response for commit metadata: meta={:?}, data={:?}",
                            meta, data
                        );
                        if meta.status.is_success() {
                            match data {
                                Ok(d) => callback(Ok(d)),
                                Err(e) => callback(Err(Error::Fetch(e.to_string()))),
                            }
                        } else {
                            error!("Failed to get commit metadata: {:?}", meta.status);
                            callback(Err(Error::Http(meta.status)));
                        }
                    }),
                )
                .unwrap(),
        )
    }
}
