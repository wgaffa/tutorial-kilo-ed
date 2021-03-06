use crate::{text::{ConsoleWidthChar, ConsoleWidthStr, nth_position_width}, Position};

pub trait Cursor {
    fn x(&self) -> u16;
    fn y(&self) -> u16;
}

pub trait CursorMovement:
    VerticalMovement + HorizontalMovement + LineMovement + PageMovement
{
}

pub trait HorizontalMovement {
    fn left(&mut self);
    fn right(&mut self);
}

pub trait VerticalMovement {
    fn up(&mut self);
    fn down(&mut self);
}

pub trait LineMovement {
    fn begin(&mut self);
    fn end(&mut self);
}

pub trait PageMovement {
    fn top(&mut self);
    fn bottom(&mut self);
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StaticCursor(u16, u16);

impl Cursor for StaticCursor {
    fn x(&self) -> u16 {
        self.0
    }

    fn y(&self) -> u16 {
        self.1
    }
}

#[derive(Debug, Clone, Default)]
pub struct BoundedCursor {
    position: Position,
    buffer: crate::RowBufferRef,
    screen: crate::ScreenRef,
}

impl BoundedCursor {
    pub(crate) fn set_buffer(&mut self, buffer: crate::RowBufferRef) {
        self.buffer = buffer;
    }

    pub(crate) fn set_screen(&mut self, screen: crate::ScreenRef) {
        self.screen = screen;
    }

    pub(crate) fn render(&self) -> usize {
        self.buffer
            .borrow()
            .get(self.position.1 as usize)
            .map(|row| render_cursor(row.buffer(), self.x() as usize, crate::TAB_STOP))
            .unwrap_or(0)
    }

    pub fn position_mut(&mut self) -> &mut Position {
        &mut self.position
    }
}

impl Cursor for BoundedCursor {
    fn x(&self) -> u16 {
        self.position.0
    }

    fn y(&self) -> u16 {
        self.position.1
    }
}

impl CursorMovement for BoundedCursor {}

impl HorizontalMovement for BoundedCursor {
    fn left(&mut self) {
        let buf = self.buffer.borrow();
        let line = buf.get(self.position.1 as usize);

        let prev_width = line
            .map(|row| nth_position_width(row.buffer(), self.position.0.saturating_sub(1) as usize) as u16)
            .unwrap_or(1);

        let (value, overflowed) = self.position.0.overflowing_sub(prev_width);

        // Check if we went past the beginning of the line and where we are in the y axis
        // to determine how we wrap around to the previous line
        match (overflowed, self.position.1) {
            (true, 1..) => {
                self.position.1 -= 1;

                // We can't use the old line variable here as we are switching lines
                // and need to get the new line from the buffer
                let column_width = buf
                    .get(self.position.1 as usize)
                    .map(|row| row.buffer().column_width())
                    .unwrap_or(0);

                self.position.0 = column_width as u16;
            } // We are not at the top, wrap
            (true, 0) => self.position.0 = 0, // We are at top and wrapped, keep at 0
            (false, _) => self.position.0 = value, // No wrapping needed, set the value
        }
    }

    fn right(&mut self) {
        let buf = self.buffer.borrow();
        let line = buf.get(self.position.1 as usize);

        let next_width = line
            .map(|row| nth_position_width(row.buffer(), self.position.0 as usize) as u16)
            .unwrap_or(1);

        let value = self.position.0.saturating_add(next_width);
        let column_width = line.map(|row| row.buffer().column_width()).unwrap_or(0) as u16;

        match (value > column_width, self.position.1) {
            (true, y) if y >= buf.len() as u16 => self.position.0 = column_width,
            (true, y) => {
                self.position.0 = 0;
                self.position.1 = y + 1;
            }
            (false, _) => self.position.0 = value,
        }
    }
}

impl VerticalMovement for BoundedCursor {
    fn up(&mut self) {
        self.position.1 = self.position.1.saturating_sub(1);

        let buffer = self.buffer.borrow();
        self.position.0 = self.position.0.min(
            buffer
                .get(self.position.1 as usize)
                .map(|row| row.buffer().column_width())
                .unwrap_or(1) as u16,
        );
    }

    fn down(&mut self) {
        let last_line = self.buffer.borrow().len() as u16;
        self.position.1 = self.position.1.saturating_add(1).min(last_line);

        let buffer = self.buffer.borrow();
        self.position.0 = self.position.0.min(
            buffer
                .get(self.position.1 as usize)
                .map(|row| row.buffer().column_width())
                .unwrap_or(1) as u16,
        );
    }
}

impl LineMovement for BoundedCursor {
    fn begin(&mut self) {
        self.position.0 = 0;
    }

    fn end(&mut self) {
        let last_column = self
            .buffer
            .borrow()
            .get(self.position.1 as usize)
            .map(|row| row.buffer().column_width())
            .unwrap_or(0) as u16;
        self.position.0 = last_column;
    }
}

impl PageMovement for BoundedCursor {
    fn top(&mut self) {
        let screen = self.screen.borrow();
        self.position.1 = screen.row_offset();
        self.position.1 = self.position.1.saturating_sub(screen.rows());
    }

    fn bottom(&mut self) {
        let screen = self.screen.borrow();
        let lines = self.buffer.borrow().len() as u16;
        let y = lines.min(screen.row_offset() + screen.rows() - 1);

        self.position.1 = y;

        self.position.1 = self.position.1.saturating_add(screen.rows()).min(lines);
    }
}

fn render_cursor(buffer: &str, cursor: usize, tabstop: usize) -> usize {
    buffer
        .chars()
        .scan(0, |st, ch| {
            if cursor > *st {
                let (width, render_width) = if ch == '\t' {
                    let tabstop = (tabstop - 1) - (*st % tabstop) + 1;
                    (1, tabstop)
                } else {
                    let width = ch.render_width();
                    (width, width)
                };

                *st += width;

                Some(render_width)
            } else {
                None
            }
        })
        .sum()
}
