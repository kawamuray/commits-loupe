use super::chart::{self, ChartComponent};
use super::CommitViewData;
use crate::api::commit_metadata::CommitMetadataApi;
use crate::api::github::GitHubApi;
use crate::api::{CommitsApi, MetadataApi};
use crate::cache::{ApiCache, CommitsApiKey};
use crate::chart::Chart;
use crate::component::table::{self, TableComponent};
use crate::dataset::CommitDataSet;
use crate::query::Query;
use crate::range::Range;
use log::*;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use yew::prelude::*;

/// A container component to contain single unit of view
pub struct ContainerComponent<C, A, M>
where
    C: Chart + 'static,
    A: CommitsApi + 'static,
    M: MetadataApi + 'static,
{
    link: ComponentLink<Self>,
    props: Properties<A, M>,
    data: Option<Rc<CommitViewData>>,
    phantom: PhantomData<C>,
}

/// Message types for `ContainerComponent`
pub enum Msg {
    /// Received target commits data
    DataReady(CommitDataSet),
    /// Error fetching data
    DataFetchError(anyhow::Error),
}

#[derive(Debug, Properties)]
pub struct Apis<A: CommitsApi, M: MetadataApi> {
    pub commits: Rc<RefCell<A>>,
    pub metadata: Rc<RefCell<M>>,
}

impl<A: CommitsApi, M: MetadataApi> Clone for Apis<A, M> {
    fn clone(&self) -> Self {
        Self {
            commits: Rc::clone(&self.commits),
            metadata: Rc::clone(&self.metadata),
        }
    }
}

#[derive(Debug, Properties)]
pub struct Properties<A: CommitsApi, M: MetadataApi> {
    pub repo: String,
    pub range: Range,
    pub file: String,
    pub value_title: String,
    pub query: String,
    pub apis: Apis<A, M>,
}

impl<A: CommitsApi, M: MetadataApi> Clone for Properties<A, M> {
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            range: self.range.clone(),
            file: self.file.clone(),
            value_title: self.value_title.clone(),
            query: self.query.clone(),
            apis: self.apis.clone(),
        }
    }
}

impl<C: Chart, A: CommitsApi, M: MetadataApi> ContainerComponent<C, A, M> {
    fn fetch_view_data(&self) {
        let cb = self.link.callback(|resp| match resp {
            Ok(dataset) => Msg::DataReady(dataset),
            Err(e) => Msg::DataFetchError(anyhow::Error::new(e)),
        });

        CommitDataSet::collect_range(
            Rc::clone(&self.props.apis.commits),
            Rc::clone(&self.props.apis.metadata),
            &self.props.repo,
            &self.props.file,
            self.props.range.clone(),
            move |resp| cb.emit(resp),
        );
    }
}

impl<C: Chart, A: CommitsApi + 'static, M: MetadataApi + 'static> Component
    for ContainerComponent<C, A, M>
{
    type Message = Msg;
    type Properties = Properties<A, M>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let this = Self {
            link,
            props,
            data: None,
            phantom: PhantomData,
        };
        this.fetch_view_data();
        this
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        self.fetch_view_data();
        // fetch_view_data() will trigger re-rendering.
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::DataReady(dataset) => {
                let query = match Query::new(&self.props.query) {
                    Ok(q) => q,
                    Err(e) => {
                        error!(
                            "cannot instantiate jmespath query '{}': {}",
                            self.props.query, e
                        );
                        return false;
                    }
                };

                match CommitViewData::from_dataset(dataset, &query) {
                    Ok(view_data) => {
                        self.data.replace(Rc::new(view_data));
                    }
                    Err(e) => error!("Could not make view data from fetched metadata: {}", e),
                }

                true
            }
            Msg::DataFetchError(e) => {
                error!("Error in fetching data: {}", e);
                false
            }
        }
    }

    fn view(&self) -> Html {
        let chart_props = chart::Properties {
            data: self.data.as_ref().map(Rc::clone),
            value_title: self.props.value_title.clone(),
        };
        let table_props = table::Properties {
            value_title: self.props.value_title.clone(),
            data: self.data.as_ref().map(Rc::clone),
        };

        html! {
            <div class="loupe-container">
              <ChartComponent<C> with chart_props />
              <TableComponent with table_props />
            </div>
        }
    }
}
