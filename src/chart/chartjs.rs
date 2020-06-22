use crate::chart;
use crate::component::CommitViewData;
use js_sys::{Array, Map, Object, Reflect};
use log::*;
use number_prefix::NumberPrefix;
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
            datapoints.push(value);
        }

        let options = Map::new();
        options.set(&JsValue::from_str("type"), &JsValue::from_str("line"));
        options.set(
            &JsValue::from_str("data"),
            &Object::from_entries(
                &Map::new()
                    .set(
                        &JsValue::from_str("labels"),
                        &labels
                            .into_iter()
                            .map(|v| JsValue::from_str(&v))
                            .collect::<Array>(),
                    )
                    .set(
                        &JsValue::from_str("datasets"),
                        &JsValue::from(Array::of1(
                            &Object::from_entries(
                                &Map::new()
                                    .set(
                                        &JsValue::from_str("label"),
                                        &JsValue::from_str(&config.title),
                                    )
                                    .set(
                                        &JsValue::from_str("backgroundColor"),
                                        &JsValue::from_str(BACKGROUND_COLOR),
                                    )
                                    .set(
                                        &JsValue::from_str("borderColor"),
                                        &JsValue::from_str(BORDER_COLOR),
                                    )
                                    .set(
                                        &JsValue::from_str("data"),
                                        &datapoints
                                            .into_iter()
                                            .map(|v| {
                                                if let Some(val) = v {
                                                    JsValue::from_f64(*val)
                                                } else {
                                                    JsValue::NULL
                                                }
                                            })
                                            .collect::<Array>(),
                                    ),
                            )
                            .unwrap(),
                        )),
                    ),
            )
            .unwrap(),
        );

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
        options.set(
            &JsValue::from_str("options"),
            &Object::from_entries(
                &Map::new()
                    .set(
                        &JsValue::from_str("tooltips"),
                        &Object::from_entries(
                            &Map::new().set(
                                &JsValue::from_str("callbacks"),
                                &Object::from_entries(
                                    &Map::new().set(&JsValue::from_str("title"), title_cb.as_ref()),
                                )
                                .unwrap(),
                            ),
                        )
                        .unwrap(),
                    )
                    .set(
                        &JsValue::from_str("scales"),
                        &Object::from_entries(
                            &Map::new().set(
                                &JsValue::from_str("yAxes"),
                                &JsValue::from(Array::of1(
                                    &Object::from_entries(
                                        &Map::new().set(
                                            &JsValue::from_str("ticks"),
                                            &Object::from_entries(
                                                &Map::new()
                                                    .set(
                                                        &JsValue::from_str("beginAtZero"),
                                                        &JsValue::from_bool(true),
                                                    )
                                                    .set(
                                                        &JsValue::from_str("callback"),
                                                        yaxis_cb.as_ref(),
                                                    ),
                                            )
                                            .unwrap(),
                                        ),
                                    )
                                    .unwrap(),
                                )),
                            ),
                        )
                        .unwrap(),
                    )
                    .set(&JsValue::from_str("onClick"), on_click.as_ref()),
            )
            .unwrap(),
        );

        let chart = Chart::new(target, Object::from_entries(&options).unwrap());

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
