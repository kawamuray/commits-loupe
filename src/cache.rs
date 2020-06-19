use crate::api::{self, Api};
use log::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;
use yew::services::fetch::FetchTask;

type Cache<K, R> = Rc<RefCell<HashMap<K, RequestState<Result<R, api::Error>>>>>;

pub struct ApiCache<K, R, A>
where
    K: Hash + Eq + Debug,
    R: Clone,
    A: Api<K, R>,
{
    cache: Cache<K, R>,
    api: A,
}

impl<K, R, A> ApiCache<K, R, A>
where
    K: Hash + Eq + Clone + Debug + 'static,
    R: Clone + 'static,
    A: Api<K, R>,
{
    pub fn new(api: A) -> Self {
        Self {
            api,
            cache: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn fetch<F>(&mut self, key: &K, callback: F) -> Result<(), anyhow::Error>
    where
        F: FnOnce(Result<R, api::Error>) + 'static,
    {
        if let Some(cached) = self.cache.borrow_mut().get_mut(key) {
            use RequestState::*;
            match cached {
                InFlight(_, callbacks) => {
                    callbacks.push(Box::new(callback));
                }
                Cached(result) => {
                    callback(result.clone());
                }
            }
            debug!("API cache hit: {:?}", key);
            return Ok(());
        }
        debug!("API cache miss: {:?}", key);

        let cache = Rc::clone(&self.cache);
        let key_cp = key.clone();
        let task = self.api.call(
            key,
            Box::new(move |resp: Result<R, api::Error>| {
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
        )?;
        if let Some(task) = task {
            self.cache.borrow_mut().insert(
                key.clone(),
                RequestState::InFlight(task, vec![Box::new(callback)]),
            );
        }
        Ok(())
    }
}

impl<K, R, A> Api<K, R> for ApiCache<K, R, A>
where
    K: Hash + Eq + Clone + Debug + 'static,
    R: Clone + 'static,
    A: Api<K, R>,
{
    fn call<F>(&mut self, req: &K, callback: F) -> Result<Option<FetchTask>, anyhow::Error>
    where
        F: FnOnce(Result<R, api::Error>) + 'static,
    {
        self.fetch(req, callback)?;
        Ok(None)
    }
}

pub enum RequestState<R> {
    InFlight(FetchTask, Vec<Box<dyn FnOnce(R) + 'static>>),
    Cached(R),
}
