#[non_exhaustive]
pub enum CursorMovement {
    Up,
    Down,
    Left,
    Right,
    ScreenTop,
    ScreenBottom,
    ScreenEnd,
    ScreenBegin,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Position {
    x: u16,
    y: u16,
    bounds: (u16, u16),
}

impl Position {
    pub fn new(x: u16, y: u16) -> Self {
        Self {
            x,
            y,
            bounds: Default::default(),
        }
    }

    pub fn with_bounds(self, cols: u16, rows: u16) -> Self {
        Self {
            bounds: (cols.saturating_sub(1), rows.saturating_sub(1)),
            ..self
        }
    }

    pub fn x(&self) -> u16 {
        self.x
    }

    pub fn y(&self) -> u16 {
        self.y
    }

    pub fn up(&mut self) {
        self.y = self.y.saturating_sub(1);
    }

    pub fn down(&mut self) {
        self.y = (self.y + 1).clamp(0, self.bounds.1);
    }

    pub fn left(&mut self) {
        self.x = self.x.saturating_sub(1);
    }

    pub fn right(&mut self) {
        self.x = (self.x + 1).clamp(0, self.bounds.0);
    }

    pub fn far_right(&mut self) {
        self.x = self.bounds.0;
    }

    pub fn far_left(&mut self) {
        self.x = 0;
    }
}
