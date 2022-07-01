use std::io::{self, Write};

use crossterm::{
    queue,
    terminal::{Clear, ClearType},
};

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

#[derive(Debug, Clone, Copy, Default)]
pub struct Position(u16, u16);

impl Position {
    pub fn new(x: u16, y: u16) -> Self {
        Self(x, y)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Editor {
    size: ScreenSize,
    cursor: Position,
}

impl Editor {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            size: ScreenSize::new(cols, rows),
            cursor: Position::default(),
        }
    }

    pub fn draw_rows<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        for i in 0..self.size.rows() {
            if i == (self.size.rows() / 3) {
                let message = self.message();
                let padding = self.padding(message.len() as u16);

                write!(writer, "{}", padding + &message)?;
            } else {
                write!(writer, "~")?;
            }

            queue!(writer, Clear(ClearType::UntilNewLine))?;
            if i < self.size.rows() - 1 {
                write!(writer, "\r\n")?;
            }
        }

        Ok(())
    }

    fn padding(&self, message_len: u16) -> String {
        let mut padding = String::new();
        let pad_size = (self.size.cols() - message_len) / 2;
        if pad_size > 0 {
            padding.push('~');
        }

        padding += &" ".repeat(pad_size.saturating_sub(1) as usize);
        padding
    }

    fn message(&self) -> String {
        let mut message = format!("Kilo editor -- version {}", version!());

        if message.len() > self.size.cols() as usize {
            message.truncate(self.size.cols() as usize);
        }

        message
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

        println!("({cols}, {rows}) output: {:?}", output);
        println!("({cols}, {rows}) expected: {:?}", expected);

        expected == output
    }
}
