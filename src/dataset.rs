use crate::api::{self, Api, CommitListRequest, CommitMetadataRequest};
use crate::commit::CommitInfo;
use crate::range::Range;
use log::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;
use yew::services::fetch::FetchTask;

const COMMITS_PAGE_SIZE: u32 = 50;

#[derive(Debug)]
pub struct CommitDataSet {
    pub commits: Vec<CommitInfo>,
    pub metadata: HashMap<String, String>,
}

impl CommitDataSet {
    pub fn collect_range<A, M, C>(
        commits_api: Rc<RefCell<A>>,
        meta_api: Rc<RefCell<M>>,
        repo: &str,
        file: &str,
        range: Range,
        callback: C,
    ) where
        A: Api<CommitListRequest, Vec<CommitInfo>>,
        M: Api<CommitMetadataRequest, String> + 'static,
        C: FnOnce(Result<Self, api::Error>) + 'static,
    {
        let pages = range.pages_for_batch(COMMITS_PAGE_SIZE);
        let from = range.from.clone();
        let file = file.to_owned();

        let commits_sg = Rc::new(SyncGroup::new((1..=pages).collect(), move |resp| {
            let mut commits = Vec::new();

            // First flatten all commits into linear vector
            let mut kvs: Vec<_> = resp.into_iter().collect();
            kvs.sort_by_key(|(i, _)| *i);
            for (_, batch) in kvs {
                match batch {
                    Ok(batch) => {
                        commits.extend(batch);
                    }
                    Err(e) => {
                        // If any of API call for commits API fails,
                        // fail it entirely.
                        callback(Err(e));
                        return;
                    }
                }
            }

            // Select sampled element from it
            let commits = range.sample(commits);
            let commit_ids: Vec<_> = commits.iter().map(|c| c.sha.clone()).collect();

            // For each commit issue metadata fetch
            let meta_sg = Rc::new(SyncGroup::new(
                commits.iter().map(|c| c.sha.clone()).collect(),
                move |resp| {
                    let mut metadata = HashMap::new();
                    for (sha, data) in resp.into_iter() {
                        match data {
                            Ok(data) => {
                                metadata.insert(sha, data);
                            }
                            Err(e) => info!("Could not obtain metadata for commit {}: {}", sha, e),
                        }
                    }
                    callback(Ok(Self { commits, metadata }));
                },
            ));
            for c in commit_ids {
                let sg = Rc::clone(&meta_sg);
                let sha = c.clone();
                let ret = meta_api.borrow_mut().call(
                    &CommitMetadataRequest {
                        commit: c.clone(),
                        file: file.to_owned(),
                    },
                    move |resp| {
                        sg.recv(sha, resp);
                    },
                );
                match ret {
                    Ok(task) => {
                        if let Some(task) = task {
                            meta_sg.in_flight(c, task);
                        }
                    }
                    Err(e) => error!("Failed to call API for commit metadata: {:?}", e),
                }
            }
            meta_sg.try_complete();
        }));

        for i in 1..=pages {
            let sg = Rc::clone(&commits_sg);
            let ret = commits_api.borrow_mut().call(
                &CommitListRequest {
                    repo: repo.to_owned(),
                    from: from.as_ref().map(|s| s.to_owned()),
                    page: i,
                    count: COMMITS_PAGE_SIZE,
                },
                move |resp| {
                    sg.recv(i, resp);
                },
            );
            match ret {
                Ok(task) => {
                    if let Some(task) = task {
                        commits_sg.in_flight(i, task);
                    }
                }
                Err(e) => error!("Failed to call API for commits listing: {:?}", e),
            }
        }
        commits_sg.try_complete();
    }
}

enum CollectState<T> {
    Vacant,
    FetchInFlight(FetchTask),
    Present(T),
}

struct SyncGroup<K, V, C>
where
    K: Eq + Hash,
    C: FnOnce(HashMap<K, V>),
{
    states: RefCell<HashMap<K, CollectState<V>>>,
    callback: C,
}

impl<K, V, C> SyncGroup<K, V, C>
where
    K: Eq + Hash,
    C: FnOnce(HashMap<K, V>),
{
    pub fn new(keys: Vec<K>, callback: C) -> Self {
        let mut states = HashMap::with_capacity(keys.len());
        for k in keys {
            states.insert(k, CollectState::Vacant);
        }

        Self {
            states: RefCell::new(states),
            callback,
        }
    }

    fn is_all_ready(&self) -> bool {
        self.states.borrow().values().all(|state| {
            if let CollectState::Present(_) = state {
                true
            } else {
                false
            }
        })
    }

    fn update_state(&self, key: K, state: CollectState<V>) {
        self.states.borrow_mut().insert(key, state);
    }

    pub fn try_complete(self: Rc<Self>) {
        if let Ok(this) = Rc::try_unwrap(self) {
            if !this.is_all_ready() {
                panic!("there are incomplete keys in sync group despite I am the last reference");
            }
            (this.callback)(
                this.states
                    .into_inner()
                    .into_iter()
                    .map(|(k, v)| {
                        if let CollectState::Present(inner) = v {
                            (k, inner)
                        } else {
                            panic!("BUG: this never happens by the above guard");
                        }
                    })
                    .collect(),
            );
        }
    }

    pub fn in_flight(&self, key: K, task: FetchTask) {
        self.update_state(key, CollectState::FetchInFlight(task));
    }

    pub fn recv(self: Rc<Self>, key: K, value: V) {
        self.update_state(key, CollectState::Present(value));
        self.try_complete();
    }
}
