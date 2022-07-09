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
            self.width().unwrap_or(0)
        }
    }
}

pub trait ConsoleWidthStr {
    fn column_width(&self) -> usize;
}

impl ConsoleWidthStr for String {
    fn column_width(&self) -> usize {
        self.chars().fold(0, |width, ch| match ch.width() {
            None => width + 1,
            Some(w) => width + w,
        })
    }
}

impl ConsoleWidthStr for str {
    fn column_width(&self) -> usize {
        self.chars().fold(0, |width, ch| match ch.width() {
            None => width + 1,
            Some(w) => width + w,
        })
    }
}

pub fn nth_position_width(buffer: &str, position: usize) -> usize {
    buffer
        .chars()
        .map(|ch| ch.render_width())
        .scan(0, |st, width| {
            if position >= *st {
                *st += width;
                Some(width)
            } else {
                None
            }
        })
        .last()
        .unwrap_or(1)
}

pub fn buffer_width(buffer: &str) -> usize {
    buffer.chars().fold(0, |width, ch| match ch.width() {
        None => width + 1,
        Some(w) => width + if w > 0 { w } else { 1 },
    })
}

pub fn char_index(cursor: usize, buffer: &str) -> usize {
    buffer
        .chars()
        .scan(0, |st, ch| {
            if cursor > *st {
                *st += match ch.render_width() {
                    0 => 1,
                    w => w,
                };

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

    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;
    use test_case::test_case;

    #[test_case("⛄⛄", 4 => 6; "Two two-width, three byte")]
    #[test_case("⛄", 2 => 3; "Single two-width, three byte")]
    #[test_case("❄❄", 2 => 6; "Two one-width, three byte")]
    #[test_case("❄", 1 => 3; "Single one-width, three byte")]
    #[test_case("\u{200c}🦀", 2 => 7; "Crab with zero-width joiner")]
    #[test_case("👩‍🔬", 1 => 4; "Woman scientist")]
    fn char_index_should_return_byte_index_given_unicode_char(input: &str, cursor: usize) -> usize {
        char_index(cursor, input)
    }

    #[quickcheck]
    fn char_index_at_buffer_width_should_be_equal_to_string_byte_length(input: String) -> bool {
        char_index(buffer_width(&input), &input) == input.len()
    }

    #[quickcheck]
    fn buffer_width_should_be_atleast_one_for_non_empty_strings(input: String) -> TestResult {
        if input.is_empty() {
            return TestResult::discard()
        }

        TestResult::from_bool(buffer_width(&input) > 0)
    }
}
