use crate::chart;

use js_sys::{Array, Map, Object};
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

pub struct ChartJs(Chart);

impl chart::Chart for ChartJs {
    fn create(target: Element, config: &chart::Config, data: &Vec<chart::ChartData>) -> Self {
        let mut labels = Vec::with_capacity(data.len());
        let mut datapoints = Vec::with_capacity(data.len());
        for (label, value) in data.iter().rev() {
            labels.push(label);
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
                                            .map(|v| JsValue::from_f64(v))
                                            .collect::<Array>(),
                                    ),
                            )
                            .unwrap(),
                        )),
                    ),
            )
            .unwrap(),
        );
        options.set(
            &JsValue::from_str("options"),
            &Object::from_entries(
                &Map::new().set(
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
                ),
            )
            .unwrap(),
        );
        info!("chart options = {:?}", options);

        let chart = Chart::new(target, Object::from_entries(&options).unwrap());

        ChartJs(chart)
    }
}

impl Drop for ChartJs {
    fn drop(&mut self) {
        self.0.destroy();
    }
}
