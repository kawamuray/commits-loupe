use chrono::{DateTime, Local};
use std::time::SystemTime;

const SHORT_SHA_LEN: usize = 7;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitInfo {
    pub sha: String,
    pub author: UserInfo,
    pub author_date: SystemTime,
    pub committer: UserInfo,
    pub commit_date: SystemTime,
    pub message: String,
    pub view_url: String,
}

impl CommitInfo {
    pub fn sha_short(&self) -> &str {
        &self.sha[..SHORT_SHA_LEN]
    }

    pub fn message_headline(&self) -> &str {
        self.message.split('\n').next().unwrap()
    }

    pub fn author_date_str(&self) -> String {
        DateTime::<Local>::from(self.author_date)
            .format("%Y-%m-%d %H:%M")
            .to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserInfo {
    pub name: String,
    pub email: String,
}
