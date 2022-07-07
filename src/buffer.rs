use std::{borrow::Cow, cell::RefCell, fmt, fs, path::Path, rc::Rc};

use error_stack::{IntoReport, Result, ResultExt};

use crate::{
    cursor::{BoundedCursor, Cursor},
    SPACES,
    TAB_STOP,
};

pub type RowBufferRef = Rc<RefCell<Vec<Row>>>;

fn expand_tabs(buffer: &str, tab_stop: usize) -> String {
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

    buf
}

#[derive(Debug, Clone, Default)]
pub struct Row {
    buffer: String,
}

impl Row {
    pub fn new<T: Into<String>>(buffer: T) -> Self {
        Self {
            buffer: buffer.into(),
        }
    }

    pub fn render_buffer(&self) -> Cow<str> {
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

    pub fn insert(&mut self, index: usize, ch: char) {
        self.buffer.insert(index, ch);
    }

    pub fn buffer(&self) -> &str {
        &self.buffer
    }
}

impl<T: Into<String>> From<T> for Row {
    fn from(other: T) -> Self {
        Self::new(other)
    }
}

#[derive(Debug)]
pub enum BufferError {
    FailedToOpen(String),
    FailedToSave(String),
    NoFilename,
}

impl std::error::Error for BufferError {}

impl fmt::Display for BufferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FailedToOpen(path) => write!(f, "Unable to open file {path}"),
            Self::FailedToSave(path) => write!(f, "Unable to save file {path}"),
            Self::NoFilename => f.write_str("No filename was given")
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Buffer {
    buffer: RowBufferRef,
    filename: Option<String>,
    cursor: BoundedCursor,
}

impl Buffer {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, BufferError> {
        let content =
            fs::read_to_string(&path)
                .report()
                .change_context(BufferError::FailedToOpen(
                    path.as_ref().to_string_lossy().to_string(),
                ))?;

        let mut me = Self {
            buffer: Rc::new(RefCell::new(content.lines().map(Row::new).collect())),
            filename: Some(path.as_ref().to_string_lossy().into()),
            ..Default::default()
        };

        me.cursor.set_buffer(Rc::clone(&me.buffer));

        Ok(me)
    }

    pub fn save(&self) -> Result<(), BufferError> {
        if let Some(filename) = &self.filename {
            let contents = self
                .buffer
                .borrow()
                .iter()
                .map(|row| row.buffer())
                .collect::<String>();

            fs::write(filename, &contents)
                .report()
                .change_context_lazy(|| BufferError::FailedToSave(filename.clone()))
        } else {
            error_stack::bail!(BufferError::NoFilename)
        }
    }

    pub fn filename(&self) -> Option<&String> {
        self.filename.as_ref()
    }

    pub fn buffer(&self) -> &RowBufferRef {
        &self.buffer
    }

    pub fn save_cursor(&mut self, cursor: BoundedCursor) {
        self.cursor = cursor;
    }

    pub fn take_cursor(&mut self) -> BoundedCursor {
        std::mem::take(&mut self.cursor)
    }

    pub fn insert_char<T: Cursor>(&mut self, ch: char, cursor: &T) {
        let mut buffer = self.buffer.borrow_mut();
        if cursor.y() as usize == buffer.len() {
            buffer.push(Row::new(""));
        }

        let row = &mut buffer[cursor.y() as usize];
        let index = crate::text::char_index(cursor.x() as usize, row.buffer());
        row.insert(index, ch);
    }
}
