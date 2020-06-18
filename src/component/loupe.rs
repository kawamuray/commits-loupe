use super::container::{self, ContainerComponent};
use crate::api::commit_metadata::CommitMetadataApi;
use crate::api::github::GitHubApi;
use crate::api::{self, CommitsApi, MetadataApi};
use crate::cache::{self, ApiCache, CommitsApiKey, MetadataApiKey};
use crate::chart::Chart;
use crate::commit::CommitInfo;
use crate::config::Config;
use crate::range::Range;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use yew::prelude::*;
use yew::services::fetch::FetchTask;

/// The main component
pub struct LoupeComponent<C>
where
    C: Chart + 'static,
{
    link: ComponentLink<Self>,
    props: Properties,
    range: Range,
    apis: container::Apis<
        ApiCache<CommitsApiKey, Result<Vec<CommitInfo>, api::Error>, CommitsApiAdapator>,
        ApiCache<MetadataApiKey, Result<String, api::Error>, MetadataApiAdapator>,
    >,
    phantom: PhantomData<C>,
}

#[derive(Debug)]
pub enum Msg {
    ZoomIn,
    ZoomOut,
}

#[derive(Debug, Clone, Properties)]
pub struct Properties {
    pub config: Config,
}

impl<C: Chart> LoupeComponent<C> {
    fn view_containers(&self) -> Vec<Html> {
        let cfg = &self.props.config;

        let mut htmls = Vec::with_capacity(cfg.data.len());
        for data in &cfg.data {
            let props = container::Properties {
                repo: cfg.repo.clone(),
                range: self.range.clone(),
                file: data.file.clone(),
                value_title: data.title.clone(),
                query: data.query.clone(),
                apis: self.apis.clone(),
            };
            htmls.push(html! {
                <ContainerComponent<C, _, _> with props />
            });
        }
        htmls
    }
}

impl<C: Chart> Component for LoupeComponent<C> {
    type Message = Msg;
    type Properties = Properties;

    fn create(mut props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let range = Range::new(props.config.branch.take(), 50, 50);

        let mut _gh_api = GitHubApi::new();
        let gh_api = ApiCache::new(CommitsApiAdapator(_gh_api));
        let _meta_api = CommitMetadataApi::new(props.config.data_url.clone());
        let meta_api = ApiCache::new(MetadataApiAdapator(_meta_api));
        let apis = container::Apis {
            commits: Rc::new(RefCell::new(gh_api)),
            metadata: Rc::new(RefCell::new(meta_api)),
        };

        Self {
            link,
            props,
            range,
            apis,
            phantom: PhantomData,
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        use Msg::*;
        match msg {
            ZoomIn => {
                self.range.zoom(0.5);
                true
            }
            ZoomOut => {
                self.range.zoom(2.0);
                true
            }
        }
    }

    fn view(&self) -> Html {
        html! {
            <div class="loupe-root">
              <div class="loupe-ctl-container">
                <button type="button" class="loupe-button loupe-ctl-zoom-in"
                        onclick=self.link.callback(|_| Msg::ZoomIn)>{ "+ Zoom In" }</button>
                <button type="button" class="loupe-button loupe-ctl-zoom-out"
                        onclick=self.link.callback(|_| Msg::ZoomOut)>{ "- Zoom Out" }</button>
              </div>
              <div class="loupe-panels">
                { for self.view_containers() }
              </div>
            </div>
        }
    }
}

pub struct CommitsApiAdapator(GitHubApi);

impl cache::ApiAdaptor<CommitsApiKey, Result<Vec<CommitInfo>, api::Error>> for CommitsApiAdapator {
    fn call(
        &mut self,
        key: &CommitsApiKey,
        callback: Box<dyn FnOnce(Result<Vec<CommitInfo>, api::Error>)>,
    ) -> Option<FetchTask> {
        self.0.commits(
            &key.repo,
            key.from.as_ref().map(|s| s.as_ref()),
            key.page,
            key.count,
            callback,
        )
    }
}

pub struct MetadataApiAdapator(CommitMetadataApi);

impl cache::ApiAdaptor<MetadataApiKey, Result<String, api::Error>> for MetadataApiAdapator {
    fn call(
        &mut self,
        key: &MetadataApiKey,
        callback: Box<dyn FnOnce(Result<String, api::Error>)>,
    ) -> Option<FetchTask> {
        self.0.commit_metadata(&key.commit, &key.file, callback)
    }
}
