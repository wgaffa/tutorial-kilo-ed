use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal,
};

macro_rules! match_key {
    ( $code:pat , $modifier:pat ) => {
        Event::Key(KeyEvent {
            code: $code,
            modifiers: $modifier,
        })
    }
}

fn process_key() -> bool {
    match event::read() {
        Ok(match_key!(KeyCode::Char('q'), KeyModifiers::CONTROL)) => return false,
        _ => {}
    }

    true
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;

    loop {
        if !process_key() {
            break;
        }
    }

    terminal::disable_raw_mode()?;

    Ok(())
}
