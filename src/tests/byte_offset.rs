use crate::byte_cursor::{byte_offset_cursor_to_byte_offset, char_byte_offset_to_cursor};
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
    // The function returns None for the end of the string
    let cursor = char_byte_offset_to_cursor(text, 11);
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
    // The function returns None for the end of the string
    let cursor = char_byte_offset_to_cursor(text, 11);
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
    assert_eq!(byte_offset, Some(5));

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
