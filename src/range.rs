use crate::commit::CommitInfo;
use log::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range {
    pub from: Option<String>,
    pub count: u32,
    pub samples: u32,
}

impl Range {
    pub fn new(from: Option<String>, count: u32, samples: u32) -> Self {
        Self {
            from,
            count,
            samples,
        }
    }

    pub fn pages_for_batch(&self, batch_size: u32) -> u32 {
        (batch_size / self.count) + 1
    }

    pub fn sample(&self, candidates: Vec<CommitInfo>) -> Vec<CommitInfo> {
        let sample_index = (self.count / self.samples).max(1);
        debug!(
            "Sampleing {} from {} with sample_index={}",
            self.samples, self.count, sample_index
        );
        let mut samples = Vec::with_capacity(self.samples as usize);
        for (i, commit) in candidates.into_iter().enumerate() {
            if samples.len() == self.samples as usize {
                break;
            }
            if i % sample_index as usize == 0 {
                samples.push(commit);
            }
        }
        samples
    }

    pub fn zoom(&mut self, factor: f64) {
        self.count = ((self.count as f64 * factor) as u32).max(self.samples);
    }
}
