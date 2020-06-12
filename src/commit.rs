use std::time::SystemTime;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserInfo {
    pub name: String,
    pub email: String,
}
