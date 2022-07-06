use std::{borrow::Cow, cell::RefCell, fs, path::Path, rc::Rc};

use error_stack::Result;

use crate::{
    cursor::{BoundedCursor, Cursor, HorizontalMovement},
    SPACES,
    TAB_STOP,
};

pub type RowBufferRef = Rc<RefCell<Vec<Row>>>;

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

#[derive(Debug, Clone, Default)]
pub struct Buffer {
    buffer: RowBufferRef,
    filename: Option<String>,
    cursor: BoundedCursor,
}

impl Buffer {
    pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<Buffer> {
        let content = fs::read_to_string(&path)?;

        let mut me = Self {
            buffer: Rc::new(RefCell::new(content.lines().map(Row::new).collect())),
            filename: Some(path.as_ref().to_string_lossy().into()),
            ..Default::default()
        };

        me.cursor.set_buffer(Rc::clone(&me.buffer));

        Ok(me)
    }

    pub fn filename(&self) -> Option<&String> {
        self.filename.as_ref()
    }

    pub fn buffer(&self) -> &RowBufferRef {
        &self.buffer
    }

    pub fn cursor(&self) -> &BoundedCursor {
        &self.cursor
    }

    pub fn cursor_mut(&mut self) -> &mut BoundedCursor {
        &mut self.cursor
    }

    pub fn insert_char(&mut self, ch: char) {
        // We need to to get exclusive access to self.rows here,
        // but self.cursor movement also has to get a references to row and
        // this is why we need to make sure to drop the exclusive access before cursor tries
        // to access it.
        {
            let mut buffer = self.buffer.borrow_mut();
            if self.cursor.y() as usize == buffer.len() {
                buffer.push(Row::new(""));
            }

            let row = &mut buffer[self.cursor.y() as usize];
            let index = crate::text::char_index(self.cursor.x() as usize, row.buffer());
            row.insert(index, ch);
        }

        // Cursor borrows the buffer as mutable here as well
        self.cursor.right();
    }
}
