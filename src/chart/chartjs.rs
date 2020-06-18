use crate::chart;
use crate::component::CommitViewData;
use js_sys::{Array, Map, Object, Reflect};
use log::*;
use wasm_bindgen::prelude::*;
use web_sys::Element;

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
    _on_click: Closure<dyn Fn(JsValue, Array)>,
}

impl chart::Chart for ChartJs {
    fn create(target: Element, config: &chart::Config, data: &CommitViewData) -> Self {
        let commits = &data.commits;
        let mut labels = Vec::with_capacity(commits.len());
        let mut link_urls = Vec::with_capacity(commits.len());
        let mut datapoints = Vec::with_capacity(commits.len());
        for commit in commits.iter().rev() {
            link_urls.push(commit.view_url.clone());
            labels.push(commit.sha_short());
            let value = data.metadata.get(&commit.sha).expect("missing meta value");
            datapoints.push(*value);
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
                                        &JsValue::from_str("data"),
                                        &datapoints
                                            .into_iter()
                                            .map(JsValue::from_f64)
                                            .collect::<Array>(),
                                    ),
                            )
                            .unwrap(),
                        )),
                    ),
            )
            .unwrap(),
        );

        let on_click = Closure::wrap(Box::new(move |_: JsValue, elems: Array| {
            let index = Reflect::get(&elems.get(0), &JsValue::from_str("_index"))
                .expect("no elems[0]._index")
                .as_f64()
                .unwrap() as usize;
            let url = &link_urls[index];
            debug!("ChartJs onClick: index={}, link={}", index, url);
            yew::utils::window()
                .open_with_url_and_target(url, "_blank")
                .expect("set window.location.href");
        }) as Box<dyn Fn(JsValue, Array)>);
        options.set(
            &JsValue::from_str("options"),
            &Object::from_entries(
                &Map::new()
                    .set(
                        &JsValue::from_str("scales"),
                        &Object::from_entries(
                            &Map::new().set(
                                &JsValue::from_str("yAxes"),
                                &JsValue::from(Array::of1(&JsValue::from(
                                    Map::new().set(
                                        &JsValue::from_str("ticks"),
                                        Map::new()
                                            .set(
                                                &JsValue::from_str("beginAtZero"),
                                                &JsValue::from_bool(true),
                                            )
                                            .as_ref(),
                                    ),
                                ))),
                            ),
                        )
                        .unwrap(),
                    )
                    .set(&JsValue::from_str("onClick"), on_click.as_ref()),
            )
            .unwrap(),
        );
        info!("chart options = {:?}", options);

        let chart = Chart::new(target, Object::from_entries(&options).unwrap());

        ChartJs {
            chart,
            _on_click: on_click,
        }
    }
}

impl Drop for ChartJs {
    fn drop(&mut self) {
        self.chart.destroy();
    }
}
