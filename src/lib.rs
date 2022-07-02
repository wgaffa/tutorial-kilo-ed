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

#[derive(Debug, Clone, Copy, Default)]
pub struct ScreenSize(u16, u16);

impl ScreenSize {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self(cols, rows)
    }

    pub fn cols(&self) -> u16 {
        self.0
    }

    pub fn rows(&self) -> u16 {
        self.1
    }
}

#[derive(Debug, Clone, Default)]
pub struct Editor {
    size: ScreenSize,
    cursor: cursor::Position,
    rows: Vec<String>,
    row_offset: u16,
    col_offset: u16,
}

impl Editor {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            size: ScreenSize::new(cols, rows),
            cursor: cursor::Position::default().with_bounds(cols, rows),
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
                        &self.rows[file_row as usize][(self.col_offset as usize)..self.col_offset as usize + len]
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
            MoveTo(self.cursor.x() - self.col_offset, self.cursor.y() - self.row_offset),
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
        match key {
            CursorMovement::Left => self.cursor.left(),
            CursorMovement::Right => self.cursor.right(),
            CursorMovement::Up => self.cursor.up(),
            CursorMovement::Down => self.cursor.down(),
            CursorMovement::ScreenTop => {
                for _ in 0..self.size.rows() {
                    self.cursor.up()
                }
            }
            CursorMovement::ScreenBottom => {
                for _ in 0..self.size.rows() {
                    self.cursor.down()
                }
            }
            CursorMovement::ScreenEnd => self.cursor.far_left(),
            CursorMovement::ScreenBegin => self.cursor.far_right(),
        }
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
        self.cursor = self
            .cursor
            .with_bounds(self.size.cols(), self.rows.len() as u16);
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
