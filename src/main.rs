use std::io::{self, Write};

use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    queue,
    terminal::{self, Clear, ClearType},
};
use errno::errno;

macro_rules! match_key {
    ( $code:pat , $modifier:pat ) => {
        Event::Key(KeyEvent {
            code: $code,
            modifiers: $modifier,
        })
    };
}

fn process_key() -> bool {
    match event::read() {
        Ok(match_key!(KeyCode::Char('q'), KeyModifiers::CONTROL)) => return false,
        _ => {}
    }

    true
}

fn refresh_screen() -> crossterm::Result<()> {
    let mut stdout = io::stdout();
    queue!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

    stdout.flush()?;

    Ok(())
}

fn die(message: &str) -> crossterm::Result<()> {
    let mut stdout = io::stdout();
    clear_screen(&mut stdout)?;

    terminal::disable_raw_mode()?;

    let errno = errno();
    eprintln!("{message}: {errno}");

    Ok(())
}

fn clear_screen<W: io::Write>(writer: &mut W) -> crossterm::Result<()> {
    queue!(writer, Clear(ClearType::All), MoveTo(0, 0))?;

    writer.flush()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;

    loop {
        if refresh_screen().is_err() {
            die("unable to refresh screen");
            break;
        }

        if !process_key() {
            break;
        }
    }

    terminal::disable_raw_mode()?;

    Ok(())
}
