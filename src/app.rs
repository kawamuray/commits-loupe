use crate::jslib;
use anyhow::Error;
use jmespatch;
use js_sys::{Array, Map, Object};
use log::*;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::JsValue;
use yew::format::{Json, Nothing};
use yew::prelude::*;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};

pub struct App {
    link: ComponentLink<Self>,
    props: ConfigProperties,
    data: Vec<CommitData>,
    perf_data: HashMap<String, f64>,
    ft1: Option<FetchTask>,
    fetch_tasks: Vec<FetchTask>,
    chart_obj: Option<jslib::Chart>,
}

pub enum Msg {
    FetchReady(Vec<CommitData>),
    PerfDataReady { commit: String, data: String },
    Nope,
}

#[derive(Debug, Clone, Properties)]
pub struct ConfigProperties {
    pub repo: String,
    pub branch: Option<String>,
    pub file: String,
    pub query: String,
}

impl Component for App {
    type Message = Msg;
    type Properties = ConfigProperties;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(
            move |response: Response<Json<Result<Vec<CommitData>, Error>>>| {
                let (meta, Json(data)) = response.into_parts();
                println!("META: {:?}, {:?}", meta, data);
                if meta.status.is_success() {
                    Msg::FetchReady(data.unwrap())
                } else {
                    Msg::Nope // FIXME: Handle this error accordingly.
                }
            },
        );
        let mut url = format!("https://api.github.com/repos/{}/commits", props.repo);
        if let Some(branch) = &props.branch {
            url.push_str(&format!("?sha={}", branch));
        }
        let request = Request::get(&url).body(Nothing).unwrap();
        let ft1 = FetchService::new().fetch(request, callback).unwrap();

        App {
            link,
            props,
            data: vec![],
            perf_data: HashMap::new(),
            ft1: Some(ft1),
            fetch_tasks: Vec::new(),
            chart_obj: None,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FetchReady(data) => {
                self.data = data;

                for c in &self.data {
                    let sha = c.sha.clone();
                    let callback =
                        self.link
                            .callback(move |response: Response<Result<String, Error>>| {
                                let (meta, data) = response.into_parts();
                                println!("META: {:?}, {:?}", meta, data);
                                if meta.status.is_success() {
                                    Msg::PerfDataReady {
                                        commit: sha.clone(),
                                        data: data.unwrap(),
                                    }
                                } else {
                                    Msg::Nope // FIXME: Handle this error accordingly.
                                }
                            });
                    let request = Request::get(format!(
                        "https://kawamuray.github.io/decaton/commit-data/{}/{}",
                        c.sha, self.props.file
                    ))
                    .body(Nothing)
                    .unwrap();
                    let ft = FetchService::new().fetch(request, callback).unwrap();
                    self.fetch_tasks.push(ft);
                }
            }
            Msg::PerfDataReady { commit, data } => {
                let expr = jmespatch::compile(&self.props.query).unwrap();
                info!("data = {}", data);
                let data = jmespatch::Variable::from_json(&data).unwrap();
                let v = expr.search(data).unwrap().as_number().unwrap();

                self.perf_data.insert(commit, v);

                if let Some(chart_obj) = self.chart_obj.take() {
                    chart_obj.destroy();
                }
                let mut labels = Vec::with_capacity(self.data.len());
                let mut data = Vec::with_capacity(self.data.len());
                for commit in self.data.iter().rev() {
                    labels.push(commit.sha.clone());
                    data.push(*self.perf_data.get(&commit.sha).unwrap_or(&0.0));
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
                                                &JsValue::from_str("Throughput"),
                                            )
                                            .set(
                                                &JsValue::from_str("data"),
                                                &data
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

                let chart = jslib::Chart::new(
                    yew::utils::document()
                        .get_element_by_id("commit-data")
                        .unwrap(),
                    Object::from_entries(&options).unwrap(),
                );
                self.chart_obj.replace(chart);
            }
            Msg::Nope => {}
        }
        true
    }

    fn view(&self) -> Html {
        info!("rendered!");
        html! {
            <div>
                <canvas id="commit-data" width="200" height="200"></canvas>
                <table>
                  <tr>
                    <th>{ "Commit" }</th>
                    <th>{ "Author" }</th>
                    <th>{ "Timestamp" }</th>
                    <th>{ "Subject" }</th>
                    <th>{ "Throughput" }</th>
                  </tr>
                  { for self.data.iter().map(|c| self.view_commit_table_entry(c)) }
                </table>
            </div>
        }
    }
}

impl App {
    fn view_commit_table_entry(&self, commit: &CommitData) -> Html {
        let throughput = *self.perf_data.get(&commit.sha).unwrap_or(&0.0);
        html! {
          <tr>
            <th><a href=commit.html_url.clone()>{ &commit.sha }</a></th>
            <th>{ &commit.commit.author.name }</th>
            <th>{ &commit.commit.author.date }</th>
            <th>{ &commit.commit.message }</th>
            <th>{ throughput }</th>
          </tr>
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommitData {
    pub sha: String,
    pub commit: Commit,
    pub html_url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Commit {
    pub author: UserInfo,
    pub committer: UserInfo,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserInfo {
    pub name: String,
    pub email: String,
    pub date: String,
}
