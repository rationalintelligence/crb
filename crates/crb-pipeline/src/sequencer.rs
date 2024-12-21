#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SeqId(u64);

#[derive(Default)]
pub struct Sequencer {
    id: u64,
}

impl Sequencer {
    pub fn next(&mut self) -> SeqId {
        self.id += 1;
        SeqId(self.id)
    }
}
