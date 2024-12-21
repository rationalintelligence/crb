#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Metadata {
    pub layer: Layer,
}

impl Metadata {
    pub fn new(layer: Layer) -> Self {
        Self { layer }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Layer(u64);

#[derive(Default)]
pub struct Sequencer {
    id: u64,
}

impl Sequencer {
    pub fn next(&mut self) -> Layer {
        self.id += 1;
        Layer(self.id)
    }
}
