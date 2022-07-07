#![cfg_attr(feature = "extend_one", feature(extend_one))]

use std::{
    cell::RefCell,
    fmt,
    io::{self, Write},
    rc::Rc,
    time::SystemTime,
};

use buffer::BufferState;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    queue,
    style::{Attribute, Print, SetAttribute},
    terminal::{Clear, ClearType},
};
use error_stack::Result;
use text::{ConsoleWidthStr, char_index};

use crate::{
    buffer::{Buffer, RowBufferRef},
    cursor::*,
    input::{CursorEvent, InputEvent},
    screen::Screen,
};

pub mod buffer;
pub mod cursor;
pub mod error;
pub mod input;
pub mod macros;
pub mod screen;
pub mod text;

const TAB_STOP: usize = 8;
const SPACES: &str = "                                                                                                                                ";
const NO_NAME: &str = "[No Name]";

type ScreenRef = Rc<RefCell<Screen>>;

/// The position on screen or buffer. The tuple index represents the horizontal value
/// x or column while the vertical is y or rows for example.
#[derive(Debug, Clone, Copy, Default)]
pub struct Position(u16, u16);

#[derive(Debug)]
pub enum EditorEventError {
    SaveBuffer,
}

impl std::error::Error for EditorEventError {}

impl fmt::Display for EditorEventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SaveBuffer => f.write_str("Unable to save buffer"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Editor {
    screen: ScreenRef,
    buffer: Buffer,
    status_message: String,
    status_time: SystemTime,
    cursor: BoundedCursor,
}

impl Editor {
    pub fn new(cols: u16, rows: u16) -> Self {
        let mut me = Self {
            screen: Rc::new(RefCell::new(Screen::new(cols, rows))),
            buffer: Default::default(),
            status_message: String::new(),
            status_time: SystemTime::now(),
            cursor: Default::default(),
        };

        me.cursor.set_buffer(Rc::clone(me.buffer.buffer()));
        me.cursor.set_screen(Rc::clone(&me.screen));

        me
    }

    pub fn draw_rows<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let screen = self.screen.borrow();
        let buf = self.buffer.buffer().borrow();
        for i in 0..screen.rows() {
            let file_row = i + screen.row_offset();
            if file_row >= buf.len() as u16 {
                if buf.is_empty() && i == (screen.rows() / 3) {
                    let message = self.message();
                    let padding = self.padding(message.len() as u16);

                    write!(writer, "{}{}", padding, &message)?;
                } else {
                    write!(writer, "~")?;
                }
            } else {
                let len = buf[file_row as usize]
                    .render_buffer()
                    .len()
                    .saturating_sub(screen.col_offset() as usize)
                    .min(screen.cols() as usize);

                if buf[file_row as usize].render_buffer().len() >= screen.col_offset() as usize {
                    write!(
                        writer,
                        "{}",
                        &buf[file_row as usize].render_buffer()
                            [(screen.col_offset() as usize)..screen.col_offset() as usize + len]
                    )?;
                }
            }

            queue!(writer, Clear(ClearType::UntilNewLine))?;
            write!(writer, "\r\n")?;
        }

        Ok(())
    }

    fn draw_status_bar<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let buf = self.buffer.buffer().borrow();
        let filename = self
            .buffer
            .filename_str()
            .map(|x| &x[..x.len().min(20)])
            .unwrap_or(NO_NAME);

        let rows = buf.len();
        let modified = match self.buffer.state() {
            BufferState::Modified => "(modified)",
            _ => "",
        };
        let left = format!("{filename} - {rows} lines {modified}");
        let right = format!("{}/{}", self.cursor.y() + 1, rows);

        let fill_length =
            (self.screen.borrow().cols() as usize).saturating_sub(right.len() + left.len());
        let modeline = if fill_length < SPACES.len() {
            format!("{left:<}{}{right:>}", &SPACES[..fill_length])
        } else {
            format!("{left:<}{}{right:>}", " ".repeat(fill_length))
        };

        queue!(
            writer,
            SetAttribute(Attribute::Reverse),
            Print(modeline),
            SetAttribute(Attribute::Reset),
            Print("\r\n"),
        )?;

        Ok(())
    }

    fn draw_message_bar<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        queue!(writer, Clear(ClearType::UntilNewLine))?;
        let message_len = self
            .status_message
            .column_width()
            .min(self.screen.borrow().cols() as usize);

        let index = char_index(message_len, &self.status_message);
        if let Ok(duration) = self.status_time.elapsed() {
            if duration.as_secs() < 5 {
                queue!(writer, Print(&self.status_message[..index]))?;
            } else {
                queue!(writer, Print(""))?;
            }
        }

        Ok(())
    }

    pub fn refresh<W: Write>(&mut self, writer: &mut W) -> crossterm::Result<()> {
        // Update the render cursor to match cursor position
        let render_x = self.cursor.render() as u16;

        self.screen.borrow_mut().scroll(render_x, self.cursor.y());
        queue!(writer, MoveTo(0, 0), Hide)?;

        self.draw_rows(writer)?;
        self.draw_status_bar(writer)?;
        self.draw_message_bar(writer)?;
        queue!(
            writer,
            MoveTo(
                render_x - self.screen.borrow().col_offset(),
                self.cursor.y() - self.screen.borrow().row_offset()
            ),
            Show
        )?;

        writer.flush()?;

        Ok(())
    }

    pub fn set_status_message<T: Into<String>>(&mut self, message: T) {
        self.status_message = message.into();
        self.status_time = SystemTime::now();
    }

    pub fn set_buffer(&mut self, buf: Buffer) {
        self.buffer = buf;
        self.cursor.set_buffer(Rc::clone(self.buffer.buffer()));
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn process_event(&mut self, event: InputEvent) -> Result<(), EditorEventError> {
        macro_rules! cursor {
            ( $ev:tt ) => {
                InputEvent::CursorEvent(CursorEvent::$ev)
            };
        }

        match event {
            cursor!(MoveLeft) => self.cursor.left(),
            cursor!(MoveRight) => self.cursor.right(),
            cursor!(MoveUp) => self.cursor.up(),
            cursor!(MoveDown) => self.cursor.down(),
            cursor!(MoveTop) => self.cursor.top(),
            cursor!(MoveBottom) => self.cursor.bottom(),
            cursor!(MoveBegin) => self.cursor.begin(),
            cursor!(MoveEnd) => self.cursor.end(),
            InputEvent::InsertChar(ch) => {
                self.buffer.insert_char(ch, &self.cursor);
                self.cursor.right()
            }
            InputEvent::SaveBuffer => {
                if let Err(_err) = self.buffer.save() {
                    self.set_status_message(format!(
                        "Can't save file {}",
                        self.buffer.filename_str().unwrap_or(crate::NO_NAME)
                    ));
                }
                self.set_status_message(format!(
                    "Saved {}",
                    self.buffer.filename_str().unwrap_or("??")
                ))
            }
            InputEvent::DeletePreviousChar => {
                self.buffer.delete_char(&mut self.cursor);
            }
            InputEvent::DeleteNextChar => {
                self.cursor.right();
                self.buffer.delete_char(&mut self.cursor);
            }
            _ => {}
        }

        Ok(())
    }

    fn padding(&self, message_len: u16) -> Padding {
        let pad_size = (self.screen.borrow().cols() - message_len) / 2;
        Padding::new('~', pad_size as usize)
    }

    fn message(&self) -> &str {
        let message = concat!("Kilo editor -- version ", version!());

        if message.len() > self.screen.borrow().cols() as usize {
            &message[..self.screen.borrow().cols() as usize]
        } else {
            message
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Padding {
    leading: char,
    size: usize,
}

impl Padding {
    pub fn new(leading: char, size: usize) -> Self {
        Self { leading, size }
    }
}

impl fmt::Display for Padding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.size > 0 {
            write!(f, "{:<size$}", self.leading, size = self.size)
        } else {
            Ok(())
        }
    }
}
