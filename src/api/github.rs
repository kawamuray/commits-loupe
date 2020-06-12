use crate::api::{CommitsApi, Error};
use crate::commit::CommitInfo;
use anyhow;
use log::*;
use schema::*;
use url::Url;
use yew::callback::Callback;
use yew::format::{Json, Nothing};
use yew::services::fetch::{FetchService, FetchTask, Request, Response};

const ENDPOINT: &str = "https://api.github.com";

pub struct GitHubApi {
    service: FetchService,
}

impl GitHubApi {
    pub fn new() -> Self {
        Self {
            service: FetchService::new(),
        }
    }

    fn build_commits_url(repo: &str, from: Option<&str>, page: u32, count: u32) -> Url {
        let page = page.to_string();
        let count = count.to_string();
        let mut params = vec![("page", page.as_ref()), ("per_page", count.as_ref())];
        if let Some(sha) = from {
            params.push(("sha", sha));
        }
        Url::parse_with_params(&format!("{}/repos/{}/commits", ENDPOINT, repo), &params)
            .expect("error building commits API url")
    }
}

impl CommitsApi for GitHubApi {
    fn commits<F>(
        &mut self,
        repo: &str,
        from: Option<&str>,
        page: u32,
        count: u32,
        callback: F,
    ) -> FetchTask
    where
        F: FnOnce(Result<Vec<CommitInfo>, Error>) + 'static,
    {
        let url = Self::build_commits_url(repo, from, page, count);
        let request = Request::get(url.as_str())
            .body(Nothing)
            .expect("build request error");
        self.service
            .fetch(
                request,
                Callback::once(
                    move |resp: Response<Json<Result<Vec<CommitData>, anyhow::Error>>>| {
                        let (meta, Json(data)) = resp.into_parts();
                        debug!(
                            "Received response for commit list: meta={:?}, data={:?}",
                            meta, data
                        );
                        if meta.status.is_success() {
                            match data {
                                Ok(d) => callback(Ok(d.into_iter().map(Into::into).collect())),
                                Err(e) => callback(Err(Error::Fetch(e))),
                            }
                        } else {
                            error!("Failed to get commits list: {:?}", meta.status);
                            callback(Err(Error::Http(meta.status)));
                        }
                    },
                ),
            )
            .unwrap()
    }
}

pub(super) mod schema {
    use crate::commit;
    use chrono::DateTime;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct CommitData {
        pub sha: String,
        pub commit: Commit,
        pub html_url: String,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Commit {
        pub author: UserInfo,
        pub committer: UserInfo,
        pub message: String,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct UserInfo {
        pub name: String,
        pub email: String,
        pub date: String,
    }

    impl From<CommitData> for commit::CommitInfo {
        fn from(data: CommitData) -> Self {
            let author = commit::UserInfo {
                name: data.commit.author.name,
                email: data.commit.author.email,
            };
            let committer = commit::UserInfo {
                name: data.commit.committer.name,
                email: data.commit.committer.email,
            };

            Self {
                sha: data.sha,
                author,
                author_date: DateTime::parse_from_rfc3339(&data.commit.author.date)
                    .expect("parse author.date")
                    .into(),
                committer,
                commit_date: DateTime::parse_from_rfc3339(&data.commit.author.date)
                    .expect("parse committer.date")
                    .into(),
                message: data.commit.message,
                view_url: data.html_url,
            }
        }
    }
}
