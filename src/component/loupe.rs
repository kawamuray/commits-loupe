use super::container::{self, ContainerComponent};
use crate::chart::Chart;
use crate::config::Config;
use crate::range::Range;
use std::marker::PhantomData;
use yew::prelude::*;

/// The main component
pub struct LoupeComponent<C>
where
    C: Chart + 'static,
{
    link: ComponentLink<Self>,
    props: Properties,
    range: Range,
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
                data_path: cfg.data_url.clone(),
                range: self.range.clone(),
                file: data.file.clone(),
                value_title: data.title.clone(),
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
    type Message = Msg;
    type Properties = Properties;

    fn create(mut props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let range = Range::new(props.config.branch.take(), 50, 50);

        Self {
            link,
            props,
            range,
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
