use std::{
    fmt,
    fs,
    io::{self, Write},
    path::Path,
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event,
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    queue,
    terminal::{Clear, ClearType},
};
use cursor::CursorMovement;

pub mod cursor;
pub mod macros;
pub mod screen;

use screen::Screen;

/// The position on screen or buffer. The tuple index represents the horizontal value
/// x or column while the vertical is y or rows for example.
#[derive(Debug, Clone, Copy, Default)]
pub struct Position(u16, u16);

#[derive(Debug, Clone, Default)]
pub struct Editor {
    size: Screen,
    cursor: cursor::BoundedCursor,
    rows: Vec<String>,
    row_offset: u16,
    col_offset: u16,
}

impl Editor {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            size: Screen::new(cols, rows),
            cursor: cursor::BoundedCursor::default(),
            rows: Default::default(),
            row_offset: 0,
            col_offset: 0,
        }
    }

    pub fn draw_rows<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        for i in 0..self.size.rows() {
            let file_row = i + self.row_offset;
            if file_row >= self.rows.len() as u16 {
                if self.rows.is_empty() && i == (self.size.rows() / 3) {
                    let message = self.message();
                    let padding = self.padding(message.len() as u16);

                    write!(writer, "{}{}", padding, &message)?;
                } else {
                    write!(writer, "~")?;
                }
            } else {
                let len = self.rows[file_row as usize]
                    .len()
                    .saturating_sub(self.col_offset as usize)
                    .min(self.size.cols() as usize);

                if self.rows[file_row as usize].len() >= self.col_offset as usize {
                    write!(
                        writer,
                        "{}",
                        &self.rows[file_row as usize]
                            [(self.col_offset as usize)..self.col_offset as usize + len]
                    )?;
                }
            }

            queue!(writer, Clear(ClearType::UntilNewLine))?;
            if i < self.size.rows() - 1 {
                write!(writer, "\r\n")?;
            }
        }

        Ok(())
    }

    pub fn refresh<W: Write>(&mut self, writer: &mut W) -> crossterm::Result<()> {
        self.scroll();
        queue!(writer, MoveTo(0, 0), Hide)?;

        self.draw_rows(writer)?;
        queue!(
            writer,
            MoveTo(
                self.cursor.x() - self.col_offset,
                self.cursor.y() - self.row_offset
            ),
            Show
        )?;

        writer.flush()?;

        Ok(())
    }

    fn scroll(&mut self) {
        if self.cursor.y() < self.row_offset {
            self.row_offset = self.cursor.y();
        }

        if self.cursor.y() >= self.row_offset + self.size.rows() {
            self.row_offset = self.cursor.y() - self.size.rows() + 1;
        }

        if self.cursor.x() < self.col_offset {
            self.col_offset = self.cursor.x();
        }

        if self.cursor.x() >= self.col_offset + self.size.cols() {
            self.col_offset = self.cursor.x() - self.size.cols() + 1;
        }
    }

    pub fn move_cursor(&mut self, key: CursorMovement) {
        let column_bound = if self.cursor.y() >= self.size.rows() {
            0
        } else {
            self.rows[self.cursor.y() as usize].len() as u16
        };

        let rows = self.rows.len() as u16;

        match key {
            CursorMovement::Left => {
                if self.cursor.y() > 0 && self.cursor.x() == 0 {
                    self.cursor.up();
                    self.cursor
                        .end(self.rows[self.cursor.y() as usize].len() as u16)
                } else {
                    self.cursor.left()
                }
            }
            CursorMovement::Right => {
                if self.cursor.y() < rows
                    && self.cursor.x() == self.rows[self.cursor.y() as usize].len() as u16
                {
                    self.cursor.down(rows);
                    self.cursor.begin();
                } else {
                    self.cursor.right(column_bound)
                }
            }
            CursorMovement::Up => self.cursor.up(),
            CursorMovement::Down => self.cursor.down(rows),
            CursorMovement::ScreenTop => {
                for _ in 0..self.size.rows() {
                    self.cursor.up()
                }
            }
            CursorMovement::ScreenBottom => {
                for _ in 0..self.size.rows() {
                    self.cursor.down(rows)
                }
            }
            CursorMovement::ScreenEnd => self.cursor.end(self.size.cols()),
            CursorMovement::ScreenBegin => self.cursor.begin(),
        }

        let (row_bound, col_bound) = {
            let r = self.cursor.y();
            let c = self.cursor.x().min(
                self.rows
                    .get(self.cursor.y() as usize)
                    .map(|x| x.len() as u16)
                    .unwrap_or(0),
            );

            (r, c)
        };

        self.cursor.snap(row_bound, col_bound);
    }

    pub fn process_key(&mut self) -> bool {
        match event::read() {
            Ok(match_key!(KeyCode::Char('q'), KeyModifiers::CONTROL)) => return false,
            Ok(key!(ch)) if matches!(ch, 'a' | 'w' | 'd' | 's') => self.move_cursor(
                Self::map_key(KeyCode::Char(ch)).expect("Could not handle incorrect keycode"),
            ),
            Ok(Event::Key(KeyEvent { code, .. }))
                if matches!(
                    code,
                    KeyCode::Left
                        | KeyCode::Right
                        | KeyCode::Up
                        | KeyCode::Down
                        | KeyCode::PageDown
                        | KeyCode::PageUp
                        | KeyCode::Home
                        | KeyCode::End
                ) =>
            {
                self.move_cursor(Self::map_key(code).expect("Could not handle incorrect keycode"));
            }
            _ => {}
        }

        true
    }

    pub fn open<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let content = fs::read_to_string(path)?;
        self.rows = content.lines().map(String::from).collect();
        Ok(())
    }

    fn map_key(key: KeyCode) -> Option<CursorMovement> {
        match key {
            KeyCode::Left => Some(CursorMovement::Left),
            KeyCode::Right => Some(CursorMovement::Right),
            KeyCode::Up => Some(CursorMovement::Up),
            KeyCode::Down => Some(CursorMovement::Down),
            KeyCode::Char('a') => Some(CursorMovement::Left),
            KeyCode::Char('d') => Some(CursorMovement::Right),
            KeyCode::Char('w') => Some(CursorMovement::Up),
            KeyCode::Char('s') => Some(CursorMovement::Down),
            KeyCode::PageUp => Some(CursorMovement::ScreenTop),
            KeyCode::PageDown => Some(CursorMovement::ScreenBottom),
            KeyCode::End => Some(CursorMovement::ScreenBegin),
            KeyCode::Home => Some(CursorMovement::ScreenEnd),
            _ => None,
        }
    }

    fn padding(&self, message_len: u16) -> Padding {
        let pad_size = (self.size.cols() - message_len) / 2;
        Padding::new('~', pad_size as usize)
    }

    fn message(&self) -> &str {
        let message = concat!("Kilo editor -- version ", version!());

        if message.len() > self.size.cols() as usize {
            &message[..self.size.cols() as usize]
        } else {
            message
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Padding {
    leading: char,
    size: usize,
}

impl Padding {
    pub fn new(leading: char, size: usize) -> Self {
        Self { leading, size }
    }
}

impl fmt::Display for Padding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.size > 0 {
            write!(f, "{:<size$}", self.leading, size = self.size)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version;

    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    #[test]
    fn draw_rows_should_be_empty_given_zero_size() {
        let editor = Editor::new(0, 0);
        let mut output = Vec::new();
        editor.draw_rows(&mut output).unwrap();

        assert!(output.is_empty());
    }

    #[quickcheck]
    fn message_should_less_than_cols_size(cols: u16, rows: u16) -> bool {
        let editor = Editor::new(cols, rows);

        let message = editor.message();

        message.len() <= cols as usize
    }

    #[quickcheck]
    fn padding_should_start_with_tiled_given_positive_cols(size: (u16, u16)) -> TestResult {
        let (rows, cols) = size;

        if cols == 0 {
            return TestResult::discard();
        }

        let editor = Editor::new(cols, rows);
        let message = editor.message();

        TestResult::from_bool(message.len() <= cols as usize)
    }

    #[quickcheck]
    fn draw_rows_prints_tildes_and_escape_sequences(cols: u16, rows: u16) -> bool {
        let editor = Editor::new(cols, rows);

        let mut output = Vec::new();
        editor.draw_rows(&mut output).unwrap();

        let top = "~\x1b[K\r\n".repeat(rows as usize / 3);

        let mut message = if rows > 0 {
            format!("Kilo editor -- version {}", version!())
        } else {
            String::new()
        };
        message.truncate(cols as usize);
        let padding = {
            if rows == 0 {
                String::new()
            } else {
                let pad = (cols.saturating_sub(message.len() as u16)) / 2;
                let mut s = String::new();
                if pad > 0 {
                    s.push('~');
                }

                s + &" ".repeat(pad.saturating_sub(1) as usize)
            }
        };
        message = padding + &message;
        if rows > 0 {
            message += "\x1b[K\r\n";
        }

        let remaining_rows = rows as usize - rows as usize / 3;
        let bottom = "~\x1b[K\r\n".repeat(remaining_rows.saturating_sub(1));

        let expected = top + &message + &bottom;
        let expected = expected.trim_end().as_bytes().to_vec();

        expected == output
    }
}
