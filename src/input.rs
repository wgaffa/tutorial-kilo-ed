use std::{error::Error, fmt};

use async_std::channel::Sender;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use error_stack::{IntoReport, Result, ResultExt};

use crate::match_key;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorEvent {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveBegin,
    MoveEnd,
    MoveTop,
    MoveBottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent {
    CursorEvent(CursorEvent),
    InsertChar(char),
    Quit,
    SaveBuffer,
    DeletePreviousChar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputError {
    ReadFailure,
    SendError(InputEvent),
}

impl Error for InputError {}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputError::ReadFailure => f.write_str("Failure when reading from input device"),
            InputError::SendError(event) => write!(f, "Could not send the event '{event:?}'"),
        }
    }
}

pub struct InputSystem {
    sender: Sender<InputEvent>,
}

impl InputSystem {
    pub fn new(sender: Sender<InputEvent>) -> Self {
        Self { sender }
    }

    pub fn process_key(&self) -> Result<(), InputError> {
        let key = event::read()
            .report()
            .change_context(InputError::ReadFailure)?;

        let event = match key {
            match_key!(KeyCode::Char('q'), KeyModifiers::CONTROL) => Some(InputEvent::Quit),
            match_key!(KeyCode::Left) => Some(InputEvent::CursorEvent(CursorEvent::MoveLeft)),
            match_key!(KeyCode::Right) => Some(InputEvent::CursorEvent(CursorEvent::MoveRight)),
            match_key!(KeyCode::Up) => Some(InputEvent::CursorEvent(CursorEvent::MoveUp)),
            match_key!(KeyCode::Down) => Some(InputEvent::CursorEvent(CursorEvent::MoveDown)),
            match_key!(KeyCode::PageUp) => Some(InputEvent::CursorEvent(CursorEvent::MoveTop)),
            match_key!(KeyCode::PageDown) => Some(InputEvent::CursorEvent(CursorEvent::MoveBottom)),
            match_key!(KeyCode::Home) => Some(InputEvent::CursorEvent(CursorEvent::MoveBegin)),
            match_key!(KeyCode::End) => Some(InputEvent::CursorEvent(CursorEvent::MoveEnd)),
            match_key!(KeyCode::Char('s'), KeyModifiers::CONTROL) => Some(InputEvent::SaveBuffer),
            match_key!(KeyCode::Char(ch), KeyModifiers::NONE) => Some(InputEvent::InsertChar(ch)),
            match_key!(KeyCode::Delete) | match_key!(KeyCode::Backspace) => Some(InputEvent::DeletePreviousChar),
            _ => None,
        };

        if let Some(event) = event {
            self.sender
                .try_send(event)
                .report()
                .change_context(InputError::SendError(event))?;
        }

        Ok(())
    }
}
