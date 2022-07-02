#[derive(Debug, Clone, Copy, Default)]
struct ScreenSize(u16, u16);

#[derive(Debug, Clone, Copy, Default)]
pub struct Screen {
    size: ScreenSize,
}

impl Screen {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self { size: ScreenSize(cols, rows) }
    }

    pub fn cols(&self) -> u16 {
        self.size.0
    }

    pub fn rows(&self) -> u16 {
        self.size.1
    }
}
