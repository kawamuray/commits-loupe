pub mod chartjs;

use web_sys::Element;

pub type ChartData = (String, f64);

pub struct Config {
    pub title: String,
}

pub trait Chart {
    fn create(target: Element, config: &Config, data: &Vec<ChartData>) -> Self;
}
