/// This trait is to determine the width of a character so that we can move the cursor
/// properly. Some Unicode characters are more than one cursor wide.
pub trait ConsoleWidth {
    fn render_width(&self) -> usize;
}

impl ConsoleWidth for char {
    fn render_width(&self) -> usize {
        if *self == '\t' {
            1
        } else {
            unicode_width::UnicodeWidthChar::width(*self).unwrap_or(1)
        }
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

    #[test_case("⛄⛄", 4 => 6; "Two two-width, three byte")]
    #[test_case("⛄", 2 => 3; "Single two-width, three byte")]
    #[test_case("❄❄", 2 => 6; "Two one-width, three byte")]
    #[test_case("❄", 1 => 3; "Single one-width, three byte")]
    fn char_index_should_return_byte_index_given_unicode_char(input: &str, cursor: usize) -> usize {
        char_index(cursor, input)
    }
}
