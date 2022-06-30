use std::io::{self, Read};

use crossterm::terminal;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;

    for byte in io::stdin().bytes() {
        match byte {
            Ok(b'q') => break,
            Ok(c) if c.is_ascii_control() => print!("{c}\r\n"),
            Ok(c) => print!("{c} ('{}')\r\n", c as char),
            _ => {},
        }
    }

    terminal::disable_raw_mode()?;

    Ok(())
}
