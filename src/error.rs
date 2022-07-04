use std::{fmt, error::Error};

#[derive(Debug, Clone, Copy)]
pub struct ApplicationError;

impl Error for ApplicationError {}

impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Error while running editor")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TerminalError;

impl Error for TerminalError {}

impl fmt::Display for TerminalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("A command to terminal failed")
    }
}
