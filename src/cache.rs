use crate::api::{self, CommitsApi, MetadataApi};
use crate::commit::CommitInfo;
use log::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;
use yew::services::fetch::FetchTask;

pub trait ApiAdaptor<K, R> {
    fn call(&mut self, key: &K, callback: Box<dyn FnOnce(R)>) -> Option<FetchTask>;
}

impl<K, R, F> ApiAdaptor<K, R> for F
where
    F: FnMut(&K, Box<dyn FnOnce(R)>) -> Option<FetchTask>,
{
    fn call(&mut self, key: &K, callback: Box<dyn FnOnce(R)>) -> Option<FetchTask> {
        self(key, callback)
    }
}

pub struct ApiCache<K, R, A>
where
    K: Hash + Eq,
    R: Clone,
    A: ApiAdaptor<K, R>,
{
    cache: Rc<RefCell<HashMap<K, RequestState<R>>>>,
    adaptor: A,
}

impl<K, R, A> ApiCache<K, R, A>
where
    K: Hash + Eq + Clone + Debug + 'static,
    R: Clone + 'static,
    A: ApiAdaptor<K, R>,
{
    pub fn new(adaptor: A) -> Self {
        Self {
            adaptor,
            cache: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn fetch<F>(&mut self, key: K, callback: F)
    where
        F: FnOnce(R) + 'static,
    {
        debug!("CACHE BEGIN");
        if let Some(cached) = self.cache.borrow_mut().get_mut(&key) {
            use RequestState::*;
            match cached {
                InFlight(_, callbacks) => {
                    callbacks.push(Box::new(callback));
                }
                Cached(result) => {
                    callback(result.clone());
                }
            }
            debug!("Request for key {:?} served from cache", key);
            return;
        }

        debug!("CACHE MISS, key = {:?}", key);

        let cache = Rc::clone(&self.cache);
        let key_cp = key.clone();
        let task = self.adaptor.call(
            &key,
            Box::new(move |resp| {
                if let Some(RequestState::InFlight(_, callbacks)) =
                    cache.borrow_mut().remove(&key_cp)
                {
                    for cb in callbacks {
                        cb(resp.clone());
                    }
                }
                cache
                    .borrow_mut()
                    .insert(key_cp, RequestState::Cached(resp));
            }),
        );
        if let Some(task) = task {
            self.cache
                .borrow_mut()
                .insert(key, RequestState::InFlight(task, vec![Box::new(callback)]));
        }
    }
}

pub enum RequestState<R> {
    InFlight(FetchTask, Vec<Box<dyn FnOnce(R) + 'static>>),
    Cached(R),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommitsApiKey {
    pub repo: String,
    pub from: Option<String>,
    pub page: u32,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MetadataApiKey {
    pub commit: String,
    pub file: String,
}

impl<A: ApiAdaptor<CommitsApiKey, Result<Vec<CommitInfo>, api::Error>>> CommitsApi
    for ApiCache<CommitsApiKey, Result<Vec<CommitInfo>, api::Error>, A>
{
    fn commits<F>(
        &mut self,
        repo: &str,
        from: Option<&str>,
        page: u32,
        count: u32,
        callback: F,
    ) -> Option<FetchTask>
    where
        F: FnOnce(Result<Vec<CommitInfo>, api::Error>) + 'static,
    {
        let key = CommitsApiKey {
            repo: repo.to_owned(),
            from: from.clone().map(|s| s.to_owned()),
            page,
            count,
        };
        self.fetch(key, callback);
        None
    }
}

impl<A: ApiAdaptor<MetadataApiKey, Result<String, api::Error>>> MetadataApi
    for ApiCache<MetadataApiKey, Result<String, api::Error>, A>
{
    fn commit_metadata<F>(&mut self, commit: &str, file: &str, callback: F) -> Option<FetchTask>
    where
        F: FnOnce(Result<String, api::Error>) + 'static,
    {
        let key = MetadataApiKey {
            commit: commit.to_owned(),
            file: file.to_owned(),
        };
        self.fetch(key, callback);
        None
    }
}
