pub mod chartjs;

use crate::component::CommitViewData;
use web_sys::Element;

pub struct Config {
    pub title: String,
}

pub trait Chart {
    fn create(target: Element, config: &Config, data: &CommitViewData) -> Self;
}
