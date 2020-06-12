use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;
use yew::services::fetch::FetchTask;

pub trait ApiAdaptor<K, R> {
    fn call(&self, key: &K, callback: Box<dyn FnOnce(R)>) -> FetchTask;
}

impl<K, R, F> ApiAdaptor<K, R> for F
where
    F: Fn(&K, Box<dyn FnOnce(R)>) -> FetchTask,
{
    fn call(&self, key: &K, callback: Box<dyn FnOnce(R)>) -> FetchTask {
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
    K: Hash + Eq + Clone + 'static,
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
            return;
        }

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
        self.cache
            .borrow_mut()
            .insert(key, RequestState::InFlight(task, vec![Box::new(callback)]));
    }
}

pub enum RequestState<R> {
    InFlight(FetchTask, Vec<Box<dyn FnOnce(R) + 'static>>),
    Cached(R),
}
