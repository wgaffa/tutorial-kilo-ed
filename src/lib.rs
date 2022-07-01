use std::io::{self, Write};

pub struct Editor;

impl Editor {
    pub fn draw_rows<W: Write>(writer: &mut W) -> io::Result<()> {
        for _ in 0..24 {
            write!(writer, "~\r\n")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_rows_prints_tildes() {
        let mut output = Vec::new();
        Editor::draw_rows(&mut output).unwrap();

        let expected = "~\r\n".repeat(24).as_bytes().to_vec();

        assert_eq!(expected, output);
    }
}
