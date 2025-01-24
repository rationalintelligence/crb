use ratatui::Frame;

pub struct AppState {
    crabs: usize,
}

impl AppState {
    pub fn new() -> Self {
        Self { crabs: 0 }
    }

    pub fn plus_crab(&mut self) {
        self.crabs += 1;
    }

    pub fn render(&self, frame: &mut Frame<'_>) {
        let mut text = String::from("Hello, CRaBs! ");
        for _ in 0..self.crabs {
            text.push('🦀');
        }
        frame.render_widget(text, frame.area());
    }
}
