use crate::byte_cursor::{
    byte_offset_cursor_to_byte_offset, char_byte_offset_to_cursor, ByteCursor,
};
use cosmic_text::{Affinity, Cursor};

#[test]
pub fn test_char_byte_offset_to_cursor() {
    // Test with a simple Latin string
    let text = "Hello World";

    // Test byte offset at the beginning of the string
    let cursor = char_byte_offset_to_cursor(text, 0);
    assert_eq!(
        cursor,
        Some(Cursor {
            line: 0,
            index: 0,
            affinity: Affinity::Before,
        })
    );

    // Test byte offset at the beginning of the string
    let cursor = char_byte_offset_to_cursor(text, 1);
    assert_eq!(
        cursor,
        Some(Cursor {
            line: 0,
            index: 1,
            affinity: Affinity::Before,
        })
    );

    // Test byte offset in the middle of the string
    let cursor = char_byte_offset_to_cursor(text, 5);
    assert_eq!(
        cursor,
        Some(Cursor {
            line: 0,
            index: 5,
            affinity: Affinity::Before,
        })
    );

    // Test byte offset at the end of the string
    let cursor = char_byte_offset_to_cursor(text, 11);
    assert_eq!(
        cursor,
        Some(Cursor {
            line: 0,
            index: 11,
            affinity: Affinity::Before
        })
    );

    // Test byte offset beyond the end of the string
    let cursor = char_byte_offset_to_cursor(text, 12);
    assert_eq!(cursor, None);

    // Test with a multi-line string
    let text = "Hello\nWorld";

    // Test byte offset at the beginning of the second line
    let cursor = char_byte_offset_to_cursor(text, 6);
    assert_eq!(
        cursor,
        Some(Cursor {
            line: 1,
            index: 0,
            affinity: Affinity::Before,
        })
    );

    // Test byte offset in the middle of the second line
    let cursor = char_byte_offset_to_cursor(text, 8);
    assert_eq!(
        cursor,
        Some(Cursor {
            line: 1,
            index: 2,
            affinity: Affinity::Before,
        })
    );

    // Test byte offset at the end of the second line
    let cursor = char_byte_offset_to_cursor(text, 11);
    assert_eq!(
        cursor,
        Some(Cursor {
            line: 1,
            index: 5,
            affinity: Affinity::Before,
        })
    );

    // Test byte offset at the end of the second line
    let cursor = char_byte_offset_to_cursor(text, 12);
    assert_eq!(cursor, None);

    // Test with an invalid byte offset (beyond the end of the string)
    let cursor = char_byte_offset_to_cursor(text, 20);
    assert_eq!(cursor, None);
}

#[test]
pub fn test_byte_offset_cursor_to_byte_offset() {
    // Test with a simple Latin string
    let text = "Hello World";

    // Test cursor at the beginning of the string
    let byte_offset = byte_offset_cursor_to_byte_offset(
        text,
        Cursor {
            line: 0,
            index: 0,
            affinity: Affinity::Before,
        },
    );
    assert_eq!(byte_offset, Some(0));

    // Test cursor at the second character of the string
    let byte_offset = byte_offset_cursor_to_byte_offset(
        text,
        Cursor {
            line: 0,
            index: 1,
            affinity: Affinity::Before,
        },
    );
    assert_eq!(byte_offset, Some(1));

    // Test cursor in the middle of the string
    let byte_offset = byte_offset_cursor_to_byte_offset(
        text,
        Cursor {
            line: 0,
            index: 5,
            affinity: Affinity::Before,
        },
    );
    assert_eq!(byte_offset, Some(5));

    let byte_offset = byte_offset_cursor_to_byte_offset(
        text,
        Cursor {
            line: 0,
            index: 4,
            affinity: Affinity::After,
        },
    );
    // Foe the byte offset, we don't really care about the affinity, so we can use either Before or After
    assert_eq!(byte_offset, Some(4));

    // Test with a multi-line string
    let text = "Hello\nWorld";

    // Test cursor at the beginning of the second line
    let byte_offset = byte_offset_cursor_to_byte_offset(
        text,
        Cursor {
            line: 1,
            index: 0,
            affinity: Affinity::Before,
        },
    );
    assert_eq!(byte_offset, Some(6));

    // Test cursor in the middle of the second line
    let byte_offset = byte_offset_cursor_to_byte_offset(
        text,
        Cursor {
            line: 1,
            index: 2,
            affinity: Affinity::Before,
        },
    );
    assert_eq!(byte_offset, Some(8));

    // Test with an invalid cursor (line beyond available lines)
    let byte_offset = byte_offset_cursor_to_byte_offset(
        text,
        Cursor {
            line: 2,
            index: 0,
            affinity: Affinity::Before,
        },
    );
    assert_eq!(byte_offset, None);

    // Test with an invalid cursor (index beyond line length)
    let byte_offset = byte_offset_cursor_to_byte_offset(
        text,
        Cursor {
            line: 0,
            index: 20,
            affinity: Affinity::Before,
        },
    );
    assert_eq!(byte_offset, None);
}

#[test]
fn cursor_conversions_preserve_offsets_across_mixed_line_endings() {
    let text = "a\r\n\u{e9}\nb\rc\n\rd\r\n";
    for (byte_offset, line, index) in [
        (0, 0, 0),
        (1, 0, 1),
        (3, 1, 0),
        (5, 1, 2),
        (6, 2, 0),
        (7, 2, 1),
        (8, 3, 0),
        (9, 3, 1),
        (11, 4, 0),
        (12, 4, 1),
        (14, 5, 0),
    ] {
        let cursor = Cursor::new(line, index);
        assert_eq!(
            char_byte_offset_to_cursor(text, byte_offset),
            Some(cursor),
            "byte offset {byte_offset}"
        );
        assert_eq!(
            byte_offset_cursor_to_byte_offset(text, cursor),
            Some(byte_offset),
            "cursor {cursor:?}"
        );
    }
}

#[test]
fn cursor_conversions_reject_offsets_inside_characters_and_line_endings() {
    let text = "a\r\n\u{e9}\nb\rc\n\rd\r\n";
    for byte_offset in [2, 4, 10, 13, 15] {
        assert_eq!(
            char_byte_offset_to_cursor(text, byte_offset),
            None,
            "byte offset {byte_offset}"
        );
    }
    for cursor in [Cursor::new(0, 2), Cursor::new(1, 1), Cursor::new(6, 0)] {
        assert_eq!(byte_offset_cursor_to_byte_offset(text, cursor), None);
    }
}

#[test]
fn cursor_conversions_include_empty_and_trailing_lines() {
    for (text, line) in [
        ("", 0),
        ("a\n", 1),
        ("a\r\n", 1),
        ("a\r", 1),
        ("a\n\r", 1),
        ("\n", 1),
        ("\r\n", 1),
        ("\r", 1),
        ("\n\r", 1),
    ] {
        let cursor = Cursor::new(line, 0);
        assert_eq!(char_byte_offset_to_cursor(text, text.len()), Some(cursor));
        assert_eq!(
            byte_offset_cursor_to_byte_offset(text, cursor),
            Some(text.len())
        );
        assert_eq!(
            byte_offset_cursor_to_byte_offset(text, Cursor::new(line + 1, 0)),
            None
        );
    }
}

#[test]
fn cursor_update_recomputes_byte_offset_after_line_endings_change() {
    let line_start = Cursor::new(1, 0);
    let mut cursor = ByteCursor::from_cursor(line_start, "a\n\u{e9}").unwrap();
    assert_eq!(cursor.byte_character_start, 2);

    assert!(cursor.update_cursor(line_start, "a\r\n\u{e9}"));
    assert_eq!(cursor.byte_character_start, 3);
    assert_eq!(cursor.char_index("a\r\n\u{e9}"), Some(3));

    assert!(!cursor.update_cursor(Cursor::new(1, 1), "a\r\n\u{e9}"));
    assert_eq!(cursor.cursor, line_start);
    assert_eq!(cursor.byte_character_start, 3);
}
