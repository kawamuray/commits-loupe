use super::*;
use log::*;
use yew::callback::Callback;
use yew::format::Nothing;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};

pub struct StaticMetadataApi {
    service: FetchService,
    data_path: String,
}

impl StaticMetadataApi {
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

impl Api<CommitMetadataRequest, String> for StaticMetadataApi {
    fn call<F>(
        &mut self,
        req: &CommitMetadataRequest,
        callback: F,
    ) -> Result<Option<FetchTask>, anyhow::Error>
    where
        F: FnOnce(Result<String, Error>) + 'static,
    {
        let url = self.build_url(req.commit.as_ref(), req.file.as_ref());
        let request = Request::get(&url)
            .body(Nothing)
            .expect("build request error");
        Ok(Some(self.service.fetch(
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
        )?))
    }
}
