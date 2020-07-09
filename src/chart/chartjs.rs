use crate::chart;
use crate::component::CommitViewData;
use js_sys::{Array, Object, Reflect};
use log::*;
use number_prefix::NumberPrefix;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::Element;

const BORDER_COLOR: &str = "rgba(1,169,244,0.5)";
const BACKGROUND_COLOR: &str = "rgba(1,169,244,0.2)";

#[wasm_bindgen(module = "chart.js")]
extern "C" {
    pub type Chart;

    #[wasm_bindgen(constructor)]
    pub fn new(ctx: Element, options: Object) -> Chart;

    #[wasm_bindgen(method)]
    pub fn destroy(this: &Chart);

}

pub struct ChartJs {
    chart: Chart,
    _closures: Vec<Box<dyn Drop>>,
}

impl ChartJs {
    fn format_value(val: f64) -> String {
        match NumberPrefix::decimal(val) {
            NumberPrefix::Standalone(n) => n.to_string(),
            NumberPrefix::Prefixed(prefix, n) => format!("{}{}", n, prefix),
        }
    }
}

impl chart::Chart for ChartJs {
    fn create(target: Element, config: &chart::Config, data: &CommitViewData) -> Self {
        let commits = Rc::new(
            data.commits
                .iter()
                .map(Clone::clone)
                .rev()
                .collect::<Vec<_>>(),
        );
        let mut labels = Vec::with_capacity(commits.len());
        let mut datapoints = Vec::with_capacity(commits.len());
        for commit in commits.iter() {
            labels.push(commit.sha_short());
            let value = data.metadata.get(&commit.sha);
            datapoints.push(value.copied());
        }

        let mut closures: Vec<Box<dyn Drop>> = Vec::new();

        let coms = Rc::clone(&commits);
        let on_click = closure!(
            closures,
            dyn Fn(JsValue, Array),
            move |_: JsValue, elems: Array| {
                let index = Reflect::get(&elems.get(0), &JsValue::from_str("_index"))
                    .expect("no elems[0]._index")
                    .as_f64()
                    .unwrap() as usize;
                let url = &coms[index].view_url;
                debug!("ChartJs onClick: index={}, link={}", index, url);
                yew::utils::window()
                    .open_with_url_and_target(url, "_blank")
                    .expect("window.open");
            }
        );

        let coms = Rc::clone(&commits);
        let title_cb = closure!(
            closures,
            dyn Fn(Array, JsValue) -> JsValue,
            move |tl_item: Array, _data: JsValue| {
                let t = Reflect::get(&tl_item.get(0), &JsValue::from_str("xLabel"))
                    .unwrap()
                    .as_string()
                    .unwrap();
                let index = Reflect::get(&tl_item.get(0), &JsValue::from_str("index"))
                    .expect("no elems[0]._index")
                    .as_f64()
                    .unwrap() as usize;
                let date = &coms[index].author_date_str();
                let message = &coms[index].message_headline();
                JsValue::from_str(&format!("{}\n{} {}", date, t, message))
            }
        );
        let yaxis_cb = closure!(
            closures,
            dyn Fn(JsValue, JsValue, JsValue) -> JsValue,
            move |value: JsValue, _, _| {
                let val = Self::format_value(value.as_f64().unwrap());
                JsValue::from_str(&val)
            }
        );

        let chart_config = js_obj! {
            type => js_ref!("line"),
            data => js_obj! {
                labels => &labels.into_iter().map(JsValue::from).collect::<Array>(),
                datasets => js_arr![js_obj! {
                    backgroundColor => js_ref!(BACKGROUND_COLOR),
                    borderColor => js_ref!(BORDER_COLOR),
                    label => js_ref!(&config.title),
                    data => &datapoints.into_iter().map(JsValue::from).collect::<Array>(),
                }.as_ref()].as_ref(),
            }.as_ref(),
            options => js_obj! {
                tooltips => js_obj! {
                    callbacks => js_obj! {
                        title => title_cb.as_ref(),
                    }.as_ref(),
                }.as_ref(),
                scales => js_obj! {
                    yAxes => js_arr![js_obj! {
                        ticks => js_obj! {
                            beginAtZero => js_ref!(true),
                            callback => yaxis_cb.as_ref()
                        }.as_ref(),
                    }.as_ref()].as_ref(),
                }.as_ref(),
                onClick => on_click.as_ref(),
            }.as_ref(),
        };

        let chart = Chart::new(target, chart_config);
        ChartJs {
            chart,
            _closures: closures,
        }
    }
}

impl Drop for ChartJs {
    fn drop(&mut self) {
        self.chart.destroy();
    }
}
