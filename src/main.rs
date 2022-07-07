use std::{env, io};

use async_std::channel::{self, TryRecvError};
use crossterm::{
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use error_stack::{IntoReport, ResultExt};

use kilo_edit::{
    buffer::Buffer,
    error::ApplicationError,
    input::{InputError, InputEvent, InputSystem},
    Editor,
};

fn main() -> error_stack::Result<(), ApplicationError> {
    startup()
        .report()
        .change_context(ApplicationError)
        .attach_printable("Failed to initialize screen")?;

    let mut editor = setup_editor()
        .report()
        .change_context(ApplicationError)
        .attach_printable("Failed to initialize editor")?;

    let (tx, rx) = channel::bounded(5);

    let args = env::args().collect::<Vec<_>>();
    if args.len() >= 2 {
        let buf = Buffer::open(&args[1])
            .change_context(ApplicationError)
            .attach_printable_lazy(|| format!("Unable to open the file: {}", args[1]))?;
        editor.set_buffer(buf);
    }

    let input = InputSystem::new(tx);

    loop {
        if let Err(e) = editor.refresh(&mut io::stdout()) {
            cleanup()
                .report()
                .change_context(ApplicationError)
                .attach_printable("Failed to do terminal cleanup")?;

            return Err(error_stack::report!(ApplicationError)
                .attach_printable(format!("Unable to refresh screen: {}", e)));
        }

        let res = input.process_key();
        if let Err(err) = res {
            match *err.current_context() {
                InputError::ReadFailure => {
                    eprintln!("Something went wrong when trying to read your input. Quitting.");
                    return Err(err)
                        .change_context(ApplicationError)
                        .attach_printable("Terminal could not read from input");
                }
                InputError::SendError(event) => {
                    // If our receiver is dead we can't get any quit events so we need to preemptively quit as well
                    if rx.is_closed() {
                        eprintln!(
                            "Our channel to the input system is closed unexpectedly. Quitting"
                        );

                        return Err(err).change_context(ApplicationError).attach_printable(
                            format!(
                                "Receiver is closed and Sender could not send the event \
                                 '{event:?}'"
                            ),
                        );
                    }
                }
            }
        }

        match rx.try_recv() {
            Ok(InputEvent::Quit) => break,
            Ok(event) => {
                if let Err(rep) = editor.process_event(event) {
                    let _ = cleanup();
                    eprintln!("An error occurred when processing the event, Quitting");
                    return Err(rep).change_context(ApplicationError);
                }
            }
            Err(TryRecvError::Closed) => {
                eprintln!("InputSystem closed unexpectedly, Quitting");
                break;
            }
            _ => {}
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
