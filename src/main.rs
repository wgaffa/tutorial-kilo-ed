use std::{env, io};

use crossterm::{
    cursor::MoveTo,
    execute,
    queue,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use errno::errno;

use kilo_edit::Editor;

fn die(message: &str) -> ! {
    let mut stdout = io::stdout();
    let _ = clear_screen(&mut stdout);

    let _ = terminal::disable_raw_mode();

    let errno = errno();
    eprintln!("{message}: {errno}");

    std::process::exit(1);
}

fn clear_screen<W: io::Write>(writer: &mut W) -> crossterm::Result<()> {
    queue!(writer, Clear(ClearType::All), MoveTo(0, 0))?;

    writer.flush()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    let mut editor = setup_editor()?;
    let args = env::args().collect::<Vec<_>>();
    if args.len() >= 2 {
        editor.open(&args[1])?;
    }

    loop {
        if editor.refresh(&mut io::stdout()).is_err() {
            die("unable to refresh screen");
        }

        if !editor.process_key() {
            break;
        }
    }

    execute!(io::stdout(), LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}

fn setup_editor() -> crossterm::Result<Editor> {
    let (cols, rows) = terminal::size()?;

    Ok(Editor::new(cols, rows))
}
