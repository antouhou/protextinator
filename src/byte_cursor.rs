use cosmic_text::{Buffer, Cursor, FontSystem, LayoutCursor};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ByteCursor {
    pub cursor: Cursor,
    pub full_byte_offset: usize,
}

impl ByteCursor {
    pub fn string_start() -> Self {
        Self {
            cursor: Cursor {
                line: 0,
                index: 0,
                affinity: Default::default(),
            },
            full_byte_offset: 0,
        }
    }
    
    pub fn string_end(string: &str) -> Self {
        if string.is_empty() {
            Self::string_start()
        } else {
            let last_byte_offset = string
                .char_indices()
                .last()
                .map(|(byte_idx, _ch)| byte_idx)
                .expect("string is not empty, so there must be at least one character");
            Self {
                cursor: char_byte_offset_to_cursor(string, last_byte_offset).expect("the byte offset must be a valid cursor at this point"),
                full_byte_offset: last_byte_offset,
            }
        }
    }
    
    pub fn from_cursor(cursor: Cursor, string: &str) -> Option<ByteCursor> {
        let mut  res = Self::string_start();
        let is_valid_cursor = res.update_cursor(cursor, string);
        if is_valid_cursor {
            Some(res)
        } else {
            None
        }
    }
    
    pub fn from_byte_offset(byte_offset: usize, string: &str) -> Option<ByteCursor> {
        let mut  res = Self::string_start();
        let is_valid_byte_offset = res.update_byte_offset(byte_offset, string);
        if is_valid_byte_offset {
            Some(res)
        } else {
            None
        }
    }
    
    pub fn layout_cursor(&self, buffer: &mut Buffer, font_system: &mut FontSystem) -> Option<LayoutCursor> {
        buffer.layout_cursor(font_system, self.cursor)
    }

    /// Returns char index of the cursor in a given string
    pub fn char_index(&self, string: &str) -> Option<usize> {
        char_byte_offset_to_char_index(string, self.full_byte_offset)
    }

    pub fn update_cursor(&mut self, cursor: Cursor, string: &str) -> bool {
        if cursor == self.cursor {
            return true;
        }
        if let Some(byte_offset) = byte_offset_cursor_to_byte_offset(string, cursor) {
            self.cursor = cursor;
            self.full_byte_offset = byte_offset;
            true
        } else {
            false
        }
    }

    pub fn update_byte_offset(&mut self, byte_offset: usize, string: &str) -> bool {
        if self.full_byte_offset == byte_offset {
            return true;
        }
        if let Some(cursor) = char_byte_offset_to_cursor(string, byte_offset) {
            self.cursor = cursor;
            self.full_byte_offset = byte_offset;
            true
        } else {
            false
        }
    }
    
    pub fn prev_char_cursor(&self, string: &str) -> Option<ByteCursor> {
        Self::from_byte_offset(previous_char_byte_offset(string, self.full_byte_offset)?, string)
    }
}

pub fn char_byte_offset_to_cursor(full_text: &str, char_byte_offset: usize) -> Option<Cursor> {
    let mut cumulative = 0;
    let mut line_heh = None;
    let mut char_heh = None;
    // Iterator over lines
    for (line_number, line) in full_text.lines().enumerate() {
        let line_len = line.len();
        // Check if char_index is in the current line.
        if char_byte_offset < cumulative + line_len {
            line_heh = Some(line_number);
            char_heh = Some(char_byte_offset.saturating_sub(cumulative));
            break;
        }
        // Add one for the newline character removed by .lines()
        cumulative += line_len + 1;
    }

    if let (Some(line), Some(index)) = (line_heh, char_heh) {
        Some(Cursor {
            line,
            index,
            affinity: Default::default(),
        })
    } else {
        None
    }
}

pub fn char_index_to_byte_offset(text: &str, char_index: usize) -> Option<usize> {
    if char_index > text.chars().count() {
        return None;
    }

    for (current_char_index, (byte_offset, _)) in text.char_indices().enumerate() {
        if current_char_index == char_index {
            return Some(byte_offset);
        }
    }

    // Handle the case where char_index is exactly at the end of the string
    if char_index == text.chars().count() {
        return Some(text.len());
    }

    None
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

fn byte_offset_cursor_to_char_index(string: &str, cursor: Cursor) -> Option<usize> {
    byte_offset_cursor_to_byte_offset(string, cursor).map(|byte_offset| {
        char_byte_offset_to_char_index(string, byte_offset).unwrap_or(string.len())
    })
}

fn byte_offset_cursor_to_byte_offset(string: &str, cursor: Cursor) -> Option<usize> {
    let mut char_byte_offset = 0;

    // Iterate through lines until we reach cursor.line
    for (line_number, line) in string.lines().enumerate() {
        if line_number == cursor.line {
            // Add the index within this line, but ensure it doesn't exceed the line length
            if cursor.index <= line.len() {
                char_byte_offset += cursor.index;
                return Some(char_byte_offset);
            } else {
                // Cursor index is out of bounds for this line
                return None;
            }
        }

        // Add line length plus 1 for the newline character
        char_byte_offset += line.len() + 1;
    }

    // If cursor.line is beyond the available lines
    None
}
