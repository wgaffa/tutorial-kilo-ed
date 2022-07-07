#![cfg_attr(feature = "extend_one", feature(extend_one))]

use std::{
    borrow::Cow,
    cell::RefCell,
    fmt,
    fs,
    io::{self, Write},
    path::Path,
    rc::Rc,
    time::SystemTime,
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    queue,
    style::{Attribute, Print, SetAttribute},
    terminal::{Clear, ClearType},
};

use cursor::*;
use input::{CursorEvent, InputEvent};

pub mod cursor;
pub mod error;
pub mod input;
pub mod macros;
pub mod screen;

use cursor::Cursor;
use screen::Screen;

const TAB_STOP: usize = 8;
const SPACES: &str = "                                                                                                                                ";

type RowBufferRef = Rc<RefCell<Vec<Row>>>;
type ScreenRef = Rc<RefCell<Screen>>;

/// The position on screen or buffer. The tuple index represents the horizontal value
/// x or column while the vertical is y or rows for example.
#[derive(Debug, Clone, Copy, Default)]
pub struct Position(u16, u16);

#[derive(Debug, Clone, Default)]
struct Row {
    buffer: String,
}

fn expand_tabs(buffer: &str, tab_stop: usize) -> Cow<str> {
    let mut buf = String::with_capacity(buffer.len());
    for ch in buffer.chars() {
        if ch == '\t' {
            let spaces = if tab_stop > SPACES.len() {
                Cow::Owned(" ".repeat(tab_stop))
            } else {
                Cow::Borrowed(&SPACES[..tab_stop])
            };
            buf.push_str(&spaces);
        } else {
            buf.push(ch);
        }
    }

    Cow::Borrowed(buffer)
}

impl Row {
    fn new<T: Into<String>>(buffer: T) -> Self {
        Self {
            buffer: buffer.into(),
        }
    }

    fn render_buffer(&self) -> Cow<str> {
        for (i, ch) in self.buffer.chars().enumerate() {
            if ch == '\t' {
                let mut buf = String::with_capacity(self.buffer.len());
                buf.push_str(&self.buffer[..i]);

                #[cfg(feature = "extend_one")]
                buf.extend_one(expand_tabs(&self.buffer[i..], TAB_STOP));

                #[cfg(not(feature = "extend_one"))]
                buf.extend(std::iter::once(expand_tabs(&self.buffer[i..], TAB_STOP)));

                return Cow::Owned(buf);
            }
        }

        Cow::Borrowed(&self.buffer)
    }

    fn insert(&mut self, index: usize, ch: char) {
        self.buffer.insert(index, ch);
    }
}

#[derive(Debug, Clone)]
pub struct Editor {
    screen: ScreenRef,
    cursor: BoundedCursor,
    rows: RowBufferRef,
    filename: Option<String>,
    status_message: String,
    status_time: SystemTime,
}

impl Editor {
    pub fn new(cols: u16, rows: u16) -> Self {
        let mut me = Self {
            screen: Rc::new(RefCell::new(Screen::new(cols, rows))),
            cursor: BoundedCursor::default(),
            rows: Default::default(),
            filename: None,
            status_message: String::new(),
            status_time: SystemTime::now(),
        };

        me.cursor.set_screen(Rc::clone(&me.screen));
        me.cursor.set_buffer(Rc::clone(&me.rows));

        me
    }

    pub fn draw_rows<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let screen = self.screen.borrow();
        for i in 0..screen.rows() {
            let file_row = i + screen.row_offset();
            if file_row >= self.rows.borrow().len() as u16 {
                if self.rows.borrow().is_empty() && i == (screen.rows() / 3) {
                    let message = self.message();
                    let padding = self.padding(message.len() as u16);

                    write!(writer, "{}{}", padding, &message)?;
                } else {
                    write!(writer, "~")?;
                }
            } else {
                let len = self.rows.borrow()[file_row as usize]
                    .render_buffer()
                    .len()
                    .saturating_sub(screen.col_offset() as usize)
                    .min(screen.cols() as usize);

                if self.rows.borrow()[file_row as usize].render_buffer().len()
                    >= screen.col_offset() as usize
                {
                    write!(
                        writer,
                        "{}",
                        &self.rows.borrow()[file_row as usize].render_buffer()
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
        const NO_NAME: &str = "[No Name]";
        let filename = self
            .filename
            .as_ref()
            .map(|x| &x[..x.len().min(20)])
            .unwrap_or(NO_NAME);

        let rows = self.rows.borrow().len();
        let left = format!("{} - {} lines", filename, rows);
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
            .len()
            .min(self.screen.borrow().cols() as usize);
        if let Ok(duration) = self.status_time.elapsed() {
            if duration.as_secs() < 5 {
                queue!(writer, Print(&self.status_message[..message_len]))?;
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

    pub fn process_event(&mut self, event: InputEvent) {
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
            InputEvent::InsertChar(ch) => self.insert_char(ch),
            _ => {}
        }
    }

    pub fn open<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let content = fs::read_to_string(&path)?;
        self.rows = Rc::new(RefCell::new(content.lines().map(Row::new).collect()));
        self.filename = Some(path.as_ref().to_string_lossy().into());
        self.cursor.set_buffer(Rc::clone(&self.rows));
        Ok(())
    }

    pub fn from_str(&mut self, contents: &str) {
        self.rows = Rc::new(RefCell::new(contents.lines().map(Row::new).collect()));
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

    fn insert_char(&mut self, ch: char) {
        // We need to to get exclusive access to self.rows here,
        // but self.cursor movement also has to get a references to row and
        // this is why we need to make sure to drop the exclusive access before cursor tries
        // to access it.
        {
            let mut buffer = self.rows.borrow_mut();
            if self.cursor.y() as usize == buffer.len() {
                buffer.push(Row::new(""));
            }

            let row = &mut buffer[self.cursor.y() as usize];
            let index = char_index(self.cursor.x() as usize, &row.buffer);
            row.insert(index, ch);
        }

        // Cursor borrows the buffer as mutable here as well
        self.cursor.right();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version;

    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    #[test]
    fn draw_rows_should_be_empty_given_zero_size() {
        let editor = Editor::new(0, 0);
        let mut output = Vec::new();
        editor.draw_rows(&mut output).unwrap();

        assert!(output.is_empty());
    }

    #[quickcheck]
    fn message_should_less_than_cols_size(cols: u16, rows: u16) -> bool {
        let editor = Editor::new(cols, rows);

        let message = editor.message();

        message.len() <= cols as usize
    }

    #[quickcheck]
    fn padding_should_start_with_tiled_given_positive_cols(size: (u16, u16)) -> TestResult {
        let (rows, cols) = size;

        if cols == 0 {
            return TestResult::discard();
        }

        let editor = Editor::new(cols, rows);
        let message = editor.message();

        TestResult::from_bool(message.len() <= cols as usize)
    }

    #[quickcheck]
    fn draw_rows_prints_tildes_and_escape_sequences(cols: u16, rows: u16) -> bool {
        let editor = Editor::new(cols, rows);

        let mut output = Vec::new();
        editor.draw_rows(&mut output).unwrap();

        let top = "~\x1b[K\r\n".repeat(rows as usize / 3);

        let mut message = if rows > 0 {
            format!("Kilo editor -- version {}", version!())
        } else {
            String::new()
        };
        message.truncate(cols as usize);
        let padding = {
            if rows == 0 {
                String::new()
            } else {
                let pad = (cols.saturating_sub(message.len() as u16)) / 2;
                let mut s = String::new();
                if pad > 0 {
                    s.push('~');
                }

                s + &" ".repeat(pad.saturating_sub(1) as usize)
            }
        };
        message = padding + &message;
        if rows > 0 {
            message += "\x1b[K\r\n";
        }

        let remaining_rows = rows as usize - rows as usize / 3;
        let bottom = "~\x1b[K\r\n".repeat(remaining_rows.saturating_sub(1));

        let expected = top + &message + &bottom;
        let expected = expected.trim_end().as_bytes().to_vec();

        expected == output
    }
}
