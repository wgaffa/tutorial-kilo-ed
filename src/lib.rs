use std::io::{self, Write};

use crossterm::{
    queue,
    terminal::{Clear, ClearType},
};

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
            write!(writer, "~")?;

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
    use super::*;

    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn draw_rows_prints_tildes_and_escape_sequences(cols: u16, rows: u16) -> bool {
        let editor = Editor::new(cols, rows);

        let mut output = Vec::new();
        editor.draw_rows(&mut output).unwrap();

        let expected = "~\x1b[K\r\n".repeat(rows as usize).trim_end().as_bytes().to_vec();

        expected == output
    }
}
