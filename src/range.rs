use crate::commit::CommitInfo;

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
        let sample_index = self.count / self.samples;
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
}
