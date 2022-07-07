use unicode_width::UnicodeWidthChar;

/// This trait is to determine the width of a character so that we can move the cursor
/// properly. Some Unicode characters are more than one cursor wide.
pub trait ConsoleWidthChar {
    fn render_width(&self) -> usize;
}

impl ConsoleWidthChar for char {
    fn render_width(&self) -> usize {
        if *self == '\t' {
            1
        } else {
            self.width().unwrap_or(1)
        }
    }
}

pub trait ConsoleWidthStr {
    fn column_width(&self) -> usize;
}

impl ConsoleWidthStr for String {
    fn column_width(&self) -> usize {
        self.chars().fold(0, |width, ch| match ch.width() {
            Some(0) | None => width + 1,
            Some(w) => width + w,
        })
    }
}

impl ConsoleWidthStr for str {
    fn column_width(&self) -> usize {
        self.chars().fold(0, |width, ch| match ch.width() {
            Some(0) | None => width + 1,
            Some(w) => width + w,
        })
    }
}

pub fn char_index(cursor: usize, buffer: &str) -> usize {
    buffer
        .chars()
        .scan(0, |st, ch| {
            if cursor > *st {
                *st += ch.render_width() as usize;

                Some(ch.len_utf8())
            } else {
                None
            }
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    use test_case::test_case;
    use quickcheck_macros::quickcheck;

    #[test_case("⛄⛄", 4 => 6; "Two two-width, three byte")]
    #[test_case("⛄", 2 => 3; "Single two-width, three byte")]
    #[test_case("❄❄", 2 => 6; "Two one-width, three byte")]
    #[test_case("❄", 1 => 3; "Single one-width, three byte")]
    fn char_index_should_return_byte_index_given_unicode_char(input: &str, cursor: usize) -> usize {
        char_index(cursor, input)
    }

    #[quickcheck]
    fn char_index_at_column_width_should_be_equal_to_string_byte_length(input: String) -> bool {
        char_index(input.column_width(), &input) == input.len()
    }
}
