use cosmic_text::{Affinity, Cursor};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ByteCursor {
    pub cursor: Cursor,
    pub byte_character_start: usize,
}

impl ByteCursor {
    pub fn string_start() -> Self {
        Self {
            cursor: Cursor {
                line: 0,
                index: 0,
                affinity: Default::default(),
            },
            byte_character_start: 0,
        }
    }

    pub fn after_last_character(string: &str) -> Self {
        // The cosmic cursor must point past the last glyph, not at it: cosmic's highlight
        // comparison includes a glyph's right edge only for cursors at or past
        // (index = glyph.end, Affinity::Before).
        Self {
            cursor: char_byte_offset_to_cursor(string, string.len())
                .expect("the string length is always a valid cursor position"),
            byte_character_start: string.len(),
        }
    }

    pub fn from_cursor(cursor: Cursor, string: &str) -> Option<ByteCursor> {
        let mut res = Self::string_start();
        let is_valid_cursor = res.update_cursor(cursor, string);
        if is_valid_cursor {
            Some(res)
        } else {
            None
        }
    }

    /// Returns char index of the cursor in a given string
    pub fn char_index(&self, string: &str) -> Option<usize> {
        char_byte_offset_to_char_index(string, self.byte_character_start)
    }

    pub fn update_cursor(&mut self, cursor: Cursor, string: &str) -> bool {
        if cursor == self.cursor {
            return true;
        }
        if let Some(byte_offset) = byte_offset_cursor_to_byte_offset(string, cursor) {
            self.cursor = cursor;
            self.byte_character_start = byte_offset;
            true
        } else {
            false
        }
    }

    pub fn update_byte_offset(&mut self, byte_offset: usize, string: &str) -> bool {
        if self.byte_character_start == byte_offset {
            return true;
        }
        if let Some(cursor) = char_byte_offset_to_cursor(string, byte_offset) {
            self.cursor = cursor;
            self.byte_character_start = byte_offset;
            true
        } else {
            false
        }
    }

    pub fn prev_char_byte_offset(&self, string: &str) -> Option<usize> {
        previous_char_byte_offset(string, self.byte_character_start)
    }
}

pub fn char_byte_offset_to_cursor(full_text: &str, char_byte_offset: usize) -> Option<Cursor> {
    // Handle the special case where char_byte_offset equals the string length
    if char_byte_offset == full_text.len() {
        // Find the last line and its length
        let mut last_line_number = 0;
        let mut last_line_len = 0;

        for (line_number, line) in full_text.lines().enumerate() {
            last_line_number = line_number;
            last_line_len = line.len();
        }

        return Some(Cursor {
            line: last_line_number,
            index: last_line_len,
            affinity: Affinity::Before,
        });
    }

    // Original logic for other cases
    let mut cumulative = 0;
    let mut maybe_line = None;
    let mut maybe_char = None;
    // Iterator over lines
    for (line_number, line) in full_text.lines().enumerate() {
        let line_len = line.len();
        // Check if char_index is in the current line.
        if char_byte_offset <= cumulative + line_len {
            maybe_line = Some(line_number);
            maybe_char = Some(char_byte_offset.saturating_sub(cumulative));
            break;
        }
        // Add one for the newline character removed by .lines()
        cumulative += line_len + 1;
    }

    if let (Some(line), Some(index)) = (maybe_line, maybe_char) {
        Some(Cursor {
            line,
            index,
            affinity: Default::default(),
        })
    } else {
        None
    }
}

pub fn char_byte_offset_to_char_index(text: &str, char_byte_offset: usize) -> Option<usize> {
    if char_byte_offset > text.len() {
        return None;
    }

    // If the byte offset is at the end of the string, return the character count
    if char_byte_offset == text.len() {
        return Some(text.chars().count());
    }

    // Iterate over characters until we find a required byte offset
    for (char_index, (byte_offset, _)) in text.char_indices().enumerate() {
        if byte_offset == char_byte_offset {
            return Some(char_index);
        }
        if byte_offset > char_byte_offset {
            // The byte offset is not at a character boundary
            return None;
        }
    }

    None
}

fn previous_char_byte_offset(text: &str, current: usize) -> Option<usize> {
    // if we're already at the very start, there's no previous char
    if current == 0 {
        return None;
    }
    if current > text.len() {
        return None;
    }
    // take everything up to `current`, iterate its character indices,
    // and pick the last one
    text[..current]
        .char_indices()
        .last()
        .map(|(byte_idx, _ch)| byte_idx)
}

pub fn byte_offset_cursor_to_byte_offset(string: &str, cursor: Cursor) -> Option<usize> {
    let mut char_byte_offset = 0;

    // Iterate through lines until we reach cursor.line
    for (line_number, line) in string.lines().enumerate() {
        if line_number == cursor.line {
            // Ensure index is within bounds
            return if cursor.index <= line.len() {
                // Base offset up to this line + index
                char_byte_offset += cursor.index;

                Some(char_byte_offset)
            } else {
                // Cursor index is out of bounds for this line
                None
            };
        }

        // Add line length plus 1 for the newline character
        char_byte_offset += line.len() + 1;
    }

    // If cursor.line is beyond the available lines
    None
}
