pub mod chart;
pub mod container;
pub mod loupe;
pub mod table;

use crate::commit::CommitInfo;
use crate::dataset::CommitDataSet;
use crate::query::{self, Query};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct CommitViewData {
    pub commits: Vec<CommitInfo>,
    pub metadata: HashMap<String, f64>,
}

impl CommitViewData {
    pub fn from_dataset(ds: CommitDataSet, query: &Query) -> Result<Self, query::Error> {
        let mut meta_vals = HashMap::new();
        for c in &ds.commits {
            let meta_val = if let Some(json) = ds.metadata.get(&c.sha) {
                query.extract_value(json)?.unwrap_or_default()
            } else {
                f64::default()
            };
            meta_vals.insert(c.sha.clone(), meta_val);
        }
        Ok(Self {
            commits: ds.commits,
            metadata: meta_vals,
        })
    }
}
