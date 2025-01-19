use anyhow::Error;
use std::collections::VecDeque;

const DEFAULT_LIMIT: usize = 8;

// TODO: Consider removing
pub struct Failures {
    limit: usize,
    errors: VecDeque<Error>,
}

impl Failures {
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            errors: VecDeque::with_capacity(limit),
        }
    }
}

impl Default for Failures {
    fn default() -> Self {
        Self::new(DEFAULT_LIMIT)
    }
}

impl Failures {
    pub fn put(&mut self, res: Result<(), Error>) {
        if self.errors.len() >= self.limit {
            self.errors.pop_front();
        }
        if let Err(err) = res {
            self.errors.push_back(err);
        }
    }
}
