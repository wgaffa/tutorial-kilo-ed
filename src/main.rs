use std::{env, io};

use crossterm::{
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use error_stack::{IntoReport, ResultExt};

use kilo_edit::{error::ApplicationError, Editor};

fn main() -> error_stack::Result<(), ApplicationError> {
    startup()
        .report()
        .change_context(ApplicationError)
        .attach_printable("Failed to initialize screen")?;

    let mut editor = setup_editor()
        .report()
        .change_context(ApplicationError)
        .attach_printable("Failed to initialize editor")?;

    let args = env::args().collect::<Vec<_>>();
    if args.len() >= 2 {
        editor
            .open(&args[1])
            .report()
            .change_context(ApplicationError)
            .attach_printable_lazy(|| format!("Unable to open the file: {}", args[1]))?;
    }

    loop {
        if let Err(e) = editor.refresh(&mut io::stdout()) {
            cleanup()
                .report()
                .change_context(ApplicationError)
                .attach_printable("Failed to do terminal cleanup")?;
            return Err(error_stack::report!(ApplicationError)
                .attach_printable(format!("Unable to refresh screen: {}", e)));
        }

        if !editor.process_key() {
            break;
        }
    }

    cleanup()
        .report()
        .change_context(ApplicationError)
        .attach_printable("Failed to do terminal cleanup")?;

    Ok(())
}

fn cleanup() -> crossterm::Result<()> {
    let err1 = execute!(io::stdout(), LeaveAlternateScreen);
    let err2 = terminal::disable_raw_mode();

    err1.and(err2)
}

fn startup() -> crossterm::Result<()> {
    terminal::enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)
}

fn setup_editor() -> crossterm::Result<Editor> {
    let (cols, rows) = terminal::size()?;

    let mut editor = Editor::new(cols, rows - 2);
    editor.set_status_message("HELP: Ctrl+Q = quit");

    Ok(editor)
}
