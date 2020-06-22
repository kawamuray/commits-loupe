use crate::chart;
use crate::component::CommitViewData;
use js_sys::{Array, Map, Object, Reflect};
use log::*;
use number_prefix::NumberPrefix;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::Element;

const BORDER_COLOR: &'static str = "rgba(1,169,244,0.5)";
const BACKGROUND_COLOR: &'static str = "rgba(1,169,244,0.2)";

#[wasm_bindgen(module = "chart.js")]
extern "C" {
    pub type Chart;

    #[wasm_bindgen(constructor)]
    pub fn new(ctx: Element, options: Object) -> Chart;

    #[wasm_bindgen(method)]
    pub fn destroy(this: &Chart);

}

/// Create an instance of JS `Closure<T>`, store it into vec, and return the reference to it.
macro_rules! closure {
    ($store:expr, $type:ty, $fn:expr) => {{
        let closure = Closure::wrap(Box::new($fn) as Box<$type>);
        $store.push(Box::new(closure) as Box<dyn Drop>);
        unsafe { &**($store.last().unwrap() as *const Box<dyn Drop> as *const Box<Closure<$type>>) }
    }};
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
        let commits = Rc::new(data.commits.iter().map(Clone::clone).collect::<Vec<_>>());
        let mut labels = Vec::with_capacity(commits.len());
        let mut datapoints = Vec::with_capacity(commits.len());
        for commit in commits.iter().rev() {
            labels.push(commit.sha_short());
            let value = data.metadata.get(&commit.sha);
            datapoints.push(value.map(|v| *v));
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

        let config = Config {
            r#type: "line",
            data: DataConfig {
                labels,
                datasets: vec![DatasetConfig {
                    backgroundColor: BACKGROUND_COLOR,
                    borderColor: BORDER_COLOR,
                    label: &config.title,
                    data: datapoints,
                }],
            },
            options: OptionsConfig {
                tooltips: TooltipsConfig {
                    callbacks: TooltipCallbacks { title: title_cb },
                },
                scales: ScalesConfig {
                    yAxes: vec![AxisConfig {
                        ticks: TicksConfig {
                            beginAtZero: true,
                            callback: yaxis_cb,
                        },
                    }],
                },
                onClick: on_click,
            },
        };

        let chart = Chart::new(target, serde_wasm_bindgen::to_value(&config));
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

#[derive(Serialize, Deserialize)]
struct Config<'a> {
    r#type: &'a str,
    data: DataConfig<'a>,
    options: OptionsConfig<'a>,
}

#[derive(Serialize, Deserialize)]
struct DataConfig<'a> {
    labels: Vec<&'a str>,
    datasets: Vec<DatasetConfig<'a>>,
}

#[derive(Serialize, Deserialize)]
struct DatasetConfig<'a> {
    backgroundColor: &'a str,
    borderColor: &'a str,
    label: &'a str,
    data: Vec<Option<f64>>,
}

#[derive(Serialize, Deserialize)]
struct OptionsConfig<'a> {
    tooltips: TooltipsConfig<'a>,
    scales: ScalesConfig<'a>,
    onClick: &'a JsValue,
}

#[derive(Serialize, Deserialize)]
struct TooltipsConfig<'a> {
    callbacks: TooltipCallbacks<'a>,
}

#[derive(Serialize, Deserialize)]
struct TooltipCallbacks<'a> {
    title: &'a JsValue,
}

#[derive(Serialize, Deserialize)]
struct ScalesConfig<'a> {
    yAxes: Vec<AxisConfig<'a>>,
}

#[derive(Serialize, Deserialize)]
struct AxisConfig<'a> {
    ticks: TicksConfig<'a>,
}

#[derive(Serialize, Deserialize)]
struct TicksConfig<'a> {
    beginAtZero: bool,
    callback: &'a JsValue,
}
