use super::chart::{self, ChartComponent};
use super::CommitViewData;
use crate::api::commit_metadata::CommitMetadataApi;
use crate::api::github::GitHubApi;
use crate::chart::Chart;
use crate::component::table::{self, TableComponent};
use crate::dataset::CommitDataSet;
use crate::query::Query;
use crate::range::Range;
use log::*;
use std::marker::PhantomData;
use std::rc::Rc;
use yew::prelude::*;

/// A container component to contain single unit of view
pub struct ContainerComponent<C>
where
    C: Chart + 'static,
{
    props: Properties,
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

#[derive(Debug, Clone, Properties)]
pub struct Properties {
    pub repo: String,
    pub branch: Option<String>,
    pub data_path: String,
    pub file: String,
    pub query: String,
}

impl<C: Chart> Component for ContainerComponent<C> {
    type Message = Msg;
    type Properties = Properties;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let cb = link.callback(|resp| match resp {
            Ok(dataset) => Msg::DataReady(dataset),
            Err(e) => Msg::DataFetchError(anyhow::Error::new(e)),
        });

        let range = Range::new(props.branch.clone(), 50, 50);
        CommitDataSet::collect_range(
            GitHubApi::new(),
            CommitMetadataApi::new(props.data_path.clone()),
            &props.repo,
            &props.file,
            range,
            move |resp| cb.emit(resp),
        );

        Self {
            props,
            data: None,
            phantom: PhantomData,
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
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
            query: self.props.query.clone(),
        };
        let table_props = table::Properties {
            data: self.data.as_ref().map(Rc::clone),
            query: self.props.query.clone(),
        };

        html! {
            <div>
              <ChartComponent<C> with chart_props />
              <TableComponent with table_props />
            </div>
        }
    }
}
