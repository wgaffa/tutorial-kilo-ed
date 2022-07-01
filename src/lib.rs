use std::io::{self, Write};

use crossterm::{
    queue,
    terminal::{Clear, ClearType},
};

pub mod macros;

pub struct ScreenSize {
    cols: u16,
    rows: u16,
}

pub struct Editor {
    size: ScreenSize,
}

impl Editor {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            size: ScreenSize { cols, rows },
        }
    }

    pub fn draw_rows<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        for i in 0..self.size.rows {

            if i == (self.size.rows / 3) {
                let mut message = format!("Kilo editor -- version {}", version!());
                if message.len() > self.size.cols as usize {
                    message.truncate(self.size.cols as usize);
                }

                let padding = (self.size.cols - message.len() as u16) / 2;
                if padding > 0 {
                    write!(writer, "~")?;
                }

                write!(writer, "{}", " ".repeat(padding.saturating_sub(1) as usize))?;

                write!(writer, "{}", message)?;
            } else {
                write!(writer, "~")?;
            }

            queue!(writer, Clear(ClearType::UntilNewLine))?;
            if i < self.size.rows - 1 {
                write!(writer, "\r\n")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::version;
    use super::*;

    use quickcheck_macros::quickcheck;

    #[test]
    fn draw_rows_should_be_empty_given_zero_size() {
        let editor = Editor::new(0, 0);
        let mut output = Vec::new();
        editor.draw_rows(&mut output).unwrap();

        assert!(output.is_empty());
    }

    #[quickcheck]
    fn draw_rows_prints_tildes_and_escape_sequences(cols: u16, rows: u16) -> bool {
        let editor = Editor::new(cols, rows);

        let mut output = Vec::new();
        editor.draw_rows(&mut output).unwrap();

        let top = "~\x1b[K\r\n".repeat(rows as usize / 3);

        let mut message = if rows > 0 {format!("Kilo editor -- version {}", version!())} else {String::new()};
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
