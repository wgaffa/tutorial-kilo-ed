use crate::Position;

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
pub struct BoundedCursor {
    position: Position,
}

impl BoundedCursor {
    pub fn new(x: u16, y: u16) -> Self {
        Self {
            position: Position(x, y),
        }
    }

    pub fn x(&self) -> u16 {
        self.position.0
    }

    pub fn y(&self) -> u16 {
        self.position.1
    }

    pub fn up(&mut self) {
        self.position.1 = self.position.1.saturating_sub(1);
    }

    pub fn down(&mut self, row_bound: u16) {
        self.position.1 = self.position.1.saturating_add(1).min(row_bound);
    }

    pub fn left(&mut self) {
        self.position.0 = self.position.0.saturating_sub(1);
    }

    pub fn right(&mut self) {
        self.position.0 = self.position.0.saturating_add(1);
    }

    pub fn end(&mut self, col_bound: u16) {
        self.position.0 = col_bound;
    }

    pub fn begin(&mut self) {
        self.position.0 = 0;
    }
}
