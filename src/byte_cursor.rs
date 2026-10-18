use cosmic_text::{Affinity, Cursor, LineIter};
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

fn line_ranges(text: &str) -> impl Iterator<Item = Range<usize>> + '_ {
    // Match the buffer's final empty line for empty or terminated text.
    let trailing_empty_line = if text.is_empty() || text.ends_with(['\r', '\n']) {
        Some(text.len()..text.len())
    } else {
        None
    };
    LineIter::new(text)
        .map(|(range, _)| range)
        .chain(trailing_empty_line)
}

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
        if let Some(byte_offset) = byte_offset_cursor_to_byte_offset(string, cursor) {
            self.cursor = cursor;
            self.byte_character_start = byte_offset;
            true
        } else {
            false
        }
    }

    /// Sets the nearest preceding valid cursor position within the string.
    pub fn set_byte_offset(&mut self, byte_offset: usize, string: &str) {
        let mut byte_offset = byte_offset.min(string.len());
        loop {
            if let Some(cursor) = char_byte_offset_to_cursor(string, byte_offset) {
                self.cursor = cursor;
                self.byte_character_start = byte_offset;
                return;
            }
            byte_offset -= 1;
        }
    }

    /// Sets the cursor at or after the insertion end, on a grapheme and line boundary.
    pub fn set_byte_offset_after_insertion(&mut self, byte_offset: usize, string: &str) {
        for (grapheme_start, _) in string.grapheme_indices(true) {
            if grapheme_start < byte_offset {
                continue;
            }
            if let Some(cursor) = char_byte_offset_to_cursor(string, grapheme_start) {
                self.cursor = cursor;
                self.byte_character_start = grapheme_start;
                return;
            }
        }
        *self = Self::after_last_character(string);
    }

    pub fn prev_char_byte_offset(&self, string: &str) -> Option<usize> {
        previous_char_byte_offset(string, self.byte_character_start)
    }
}

pub fn char_byte_offset_to_cursor(full_text: &str, char_byte_offset: usize) -> Option<Cursor> {
    if !full_text.is_char_boundary(char_byte_offset) {
        return None;
    }

    for (line, line_range) in line_ranges(full_text).enumerate() {
        if (line_range.start..=line_range.end).contains(&char_byte_offset) {
            return Some(Cursor {
                line,
                index: char_byte_offset - line_range.start,
                affinity: Affinity::Before,
            });
        }
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

pub fn byte_offset_cursor_to_byte_offset(string: &str, cursor: Cursor) -> Option<usize> {
    let line_range = line_ranges(string).nth(cursor.line)?;
    let line = &string[line_range.start..line_range.end];
    if line.is_char_boundary(cursor.index) {
        Some(line_range.start + cursor.index)
    } else {
        None
    }
}
