use super::CommitViewData;
use crate::chart::{self, Chart};
use log::*;
use std::rc::Rc;
use web_sys::Element;
use yew::html::NodeRef;
use yew::prelude::*;

/// A chart component which shows line chart of commits data
pub struct ChartComponent<C: Chart + 'static> {
    props: Properties,
    canvas_ref: NodeRef,
    chart: Option<C>,
}

#[derive(Debug, Clone, Properties)]
pub struct Properties {
    pub data: Option<Rc<CommitViewData>>,
    pub value_title: String,
}

impl<C: Chart> ChartComponent<C> {
    fn refresh_chart(&mut self) {
        let data = if let Some(data) = self.props.data.as_ref() {
            data
        } else {
            debug!("Dataset is not ready");
            return;
        };

        if let Some(chart) = self.chart.take() {
            debug!("Destroying currently displaying chart");
            drop(chart);
        }

        debug!("Creating new chart with {} datapoints", data.commits.len());
        let target = self
            .canvas_ref
            .cast::<Element>()
            .expect("canvas_ref: !Element");
        let chart = C::create(
            target,
            &chart::Config {
                title: self.props.value_title.clone(),
            },
            data,
        );
        self.chart.replace(chart);
    }
}

impl<C: Chart> Component for ChartComponent<C> {
    type Message = ();
    type Properties = Properties;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {
            props,
            canvas_ref: NodeRef::default(),
            chart: None,
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
            <canvas ref=self.canvas_ref.clone() width="400" height="200"></canvas>
        }
    }

    fn rendered(&mut self, _first_render: bool) {
        self.refresh_chart();
    }
}
