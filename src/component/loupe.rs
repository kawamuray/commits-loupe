use super::container::{self, ContainerComponent};
use crate::chart::Chart;
use crate::config::Config;
use std::marker::PhantomData;
use yew::prelude::*;

/// The main component
pub struct LoupeComponent<C>
where
    C: Chart + 'static,
{
    props: Properties,
    phantom: PhantomData<C>,
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
                branch: cfg.branch.clone(),
                data_path: cfg.data_url.clone(),
                file: data.file.clone(),
                query: data.query.clone(),
            };
            htmls.push(html! {
                <ContainerComponent<C> with props />
            });
        }
        htmls
    }
}

impl<C: Chart> Component for LoupeComponent<C> {
    type Message = ();
    type Properties = Properties;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {
            props,
            phantom: PhantomData,
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        html! {
            <div>
              { for self.view_containers() }
            </div>
        }
    }
}
