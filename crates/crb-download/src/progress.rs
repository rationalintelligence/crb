#[derive(Debug)]
pub enum Progress<T> {
    Working(Option<u8>),
    Done(T),
}

#[derive(Debug)]
pub struct ProgressCalc {
    total: Option<u64>,
    processed: u64,
}

impl ProgressCalc {
    pub fn new(total: Option<u64>) -> Self {
        Self {
            total,
            processed: 0,
        }
    }

    pub fn change_total(&mut self, total: Option<u64>) {
        self.total = total;
    }

    pub fn inc(&mut self, value: u64) {
        self.processed += value;
    }

    pub fn set(&mut self, value: u64) {
        self.processed = value;
    }

    pub fn progress(&self) -> Option<u8> {
        self.total
            .as_ref()
            .map(|total| self.processed * 100 / total)
            .map(|value| value.clamp(0, 100))
            .and_then(|pct| pct.try_into().ok())
    }
}
