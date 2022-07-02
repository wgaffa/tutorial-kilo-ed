use crate::cursor::Cursor;

#[derive(Debug, Clone, Copy, Default)]
struct ScreenSize(u16, u16);

#[derive(Debug, Clone, Copy, Default)]
pub struct Screen {
    size: ScreenSize,
    offset: Offset,
}

impl Screen {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self { size: ScreenSize(cols, rows), offset: Offset(0, 0) }
    }

    pub fn cols(&self) -> u16 {
        self.size.0
    }

    pub fn rows(&self) -> u16 {
        self.size.1
    }

    pub fn col_offset(&self) -> u16 {
        self.offset.0
    }

    pub fn row_offset(&self) -> u16 {
        self.offset.1
    }

    pub fn scroll<T: Cursor>(&mut self, cursor: &T) {
        if cursor.y() < self.row_offset() {
            self.offset.1 = cursor.y();
        }

        if cursor.y() >= self.row_offset() + self.rows() {
            self.offset.1 = cursor.y() - self.rows() + 1;
        }

        if cursor.x() < self.col_offset() {
            self.offset.0 = cursor.x();
        }

        if cursor.x() >= self.col_offset() + self.cols() {
            self.offset.0 = cursor.x() - self.cols() + 1;
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Offset(u16, u16);
