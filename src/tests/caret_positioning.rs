use crate::tests::mono_style_test;
use crate::{Action, ActionResult, Id, Point, TextContext, TextState};

#[test]
pub fn test() {
    let mut ctx = TextContext::default();
    let text_id = Id::new(0);
    let initial_text = "Hello World".to_string();

    let mut text_state = TextState::new_with_text(initial_text, text_id, &mut ctx.font_system);
    text_state.set_style(&mono_style_test());
    text_state.set_outer_size(&Point::from((200.0, 25.0)));
    text_state.is_editable = true;
    text_state.is_editing = true;
    text_state.is_selectable = true;
    text_state.are_actions_enabled = true;
    text_state.recalculate(&mut ctx);
    let mono_width = text_state.first_glyph().unwrap().w;

    assert!(mono_width > 0.0);

    assert_eq!(text_state.cursor_char_index(), Some(0));
    assert_eq!(text_state.caret_position_relative().x, 0.0);

    text_state.handle_press(&mut ctx, Point { x: 25.0, y: 10.0 });
    assert_eq!(text_state.cursor_char_index(), Some(3));
    assert_eq!(
        text_state.caret_position_relative().x,
        (mono_width * 3.0).floor()
    );

    let result = text_state.apply_action(&mut ctx, &Action::InsertChar("a".into()));
    assert!(matches!(result, ActionResult::TextChanged));
    assert_eq!(text_state.text_char_len(), 12);
    assert_eq!(text_state.cursor_char_index(), Some(4));
    assert_eq!(
        text_state.caret_position_relative().x,
        (mono_width * 4.0).floor()
    );
    assert_eq!(text_state.text(), "Helalo World");

    let result = text_state.apply_action(&mut ctx, &Action::MoveCursorRight);
    assert!(matches!(result, ActionResult::CursorUpdated));
    assert_eq!(text_state.text_char_len(), 12);
    assert_eq!(text_state.cursor_char_index(), Some(5));
    assert_eq!(
        text_state.caret_position_relative().x,
        (mono_width * 5.0).floor()
    );
}

#[test]
pub fn test_cyrillic() {
    let mut ctx = TextContext::default();
    let text_id = Id::new(0);
    let initial_text = "Привет Мир".to_string();

    let mut text_state = TextState::new_with_text(initial_text, text_id, &mut ctx.font_system);
    text_state.set_style(&mono_style_test());
    text_state.set_outer_size(&Point::from((200.0, 25.0)));
    text_state.is_editable = true;
    text_state.is_editing = true;
    text_state.is_selectable = true;
    text_state.are_actions_enabled = true;

    text_state.recalculate(&mut ctx);
    let mono_width = text_state.first_glyph().unwrap().w;

    assert!(mono_width > 0.0);

    assert_eq!(text_state.text_char_len(), 10);
    assert_eq!(text_state.cursor_char_index(), Some(0));
    assert_eq!(text_state.caret_position_relative().x, 0.0);

    let result = text_state.apply_action(&mut ctx, &Action::MoveCursorRight);
    assert!(matches!(result, ActionResult::CursorUpdated));
    assert_eq!(text_state.text_char_len(), 10);
    assert_eq!(text_state.cursor_char_index(), Some(1));
    assert_eq!(
        text_state.caret_position_relative().x,
        (mono_width * 1.0).floor()
    );

    let result = text_state.apply_action(&mut ctx, &Action::MoveCursorRight);
    assert!(matches!(result, ActionResult::CursorUpdated));
    assert_eq!(text_state.text_char_len(), 10);
    assert_eq!(text_state.cursor_char_index(), Some(2));
    assert_eq!(
        text_state.caret_position_relative().x,
        (mono_width * 2.0).floor()
    );

    text_state.handle_press(&mut ctx, Point { x: 25.0, y: 10.0 });
    assert_eq!(text_state.cursor_char_index(), Some(3));
    assert_eq!(
        text_state.caret_position_relative().x,
        (mono_width * 3.0).floor()
    );

    let result = text_state.apply_action(&mut ctx, &Action::InsertChar("ш".into()));
    assert!(matches!(result, ActionResult::TextChanged));
    assert_eq!(text_state.text_char_len(), 11);
    assert_eq!(text_state.cursor_char_index(), Some(4));
    assert_eq!(
        text_state.caret_position_relative().x,
        (mono_width * 4.0).floor()
    );
    assert_eq!(text_state.text(), "Пришвет Мир");

    let result = text_state.apply_action(&mut ctx, &Action::MoveCursorRight);
    assert!(matches!(result, ActionResult::CursorUpdated));
    assert_eq!(text_state.text_char_len(), 11);
    assert_eq!(text_state.cursor_char_index(), Some(5));
    assert_eq!(
        text_state.caret_position_relative().x,
        (mono_width * 5.0).floor()
    );
}

#[test]
pub fn test_insert_into_empty_text() {
    // Test for the bug: If starting with an empty text and the cursor is at 0,
    // the caret doesn't move when inserting the first character
    let mut ctx = TextContext::default();
    let text_id = Id::new(0);
    let initial_text = "".to_string(); // Empty text

    let mut text_state = TextState::new_with_text(initial_text, text_id, &mut ctx.font_system);
    text_state.set_style(&mono_style_test());
    text_state.set_outer_size(&Point::from((200.0, 25.0)));
    text_state.is_editable = true;
    text_state.is_editing = true;
    text_state.is_selectable = true;
    text_state.are_actions_enabled = true;

    text_state.recalculate(&mut ctx);

    // Verify initial state
    assert_eq!(text_state.text_char_len(), 0);
    assert_eq!(text_state.cursor_char_index(), Some(0));
    assert_eq!(text_state.caret_position_relative().x, 0.0);

    // Insert the first character
    let result = text_state.apply_action(&mut ctx, &Action::InsertChar("a".into()));
    assert!(matches!(result, ActionResult::TextChanged));

    // Verify text was inserted
    assert_eq!(text_state.text_char_len(), 1);
    assert_eq!(text_state.text(), "a");

    // Verify cursor position - this should fail due to the bug
    // The cursor should be at position 1 (after the inserted character)
    assert_eq!(text_state.cursor_char_index(), Some(1));

    // Verify caret offset - this should fail due to the bug
    // The caret should have moved to the right
    let mono_width = text_state.first_glyph().unwrap().w;

    assert_eq!(
        text_state.caret_position_relative().x,
        (mono_width * 1.0).floor()
    );
}

#[test]
pub fn test_delete_at_end_of_text() {
    // Test for the bug: If the caret is at the very end of the string and trying to delete a character,
    // the code panics
    let mut ctx = TextContext::default();
    let text_id = Id::new(0);
    let initial_text = "Hello".to_string();

    let mut text_state = TextState::new_with_text(initial_text, text_id, &mut ctx.font_system);
    text_state.set_style(&mono_style_test());
    text_state.set_outer_size(&Point::from((200.0, 25.0)));
    text_state.is_editable = true;
    text_state.is_editing = true;
    text_state.is_selectable = true;
    text_state.are_actions_enabled = true;

    text_state.recalculate(&mut ctx);

    // Move cursor to the end of the text
    for _ in 0..5 {
        text_state.apply_action(&mut ctx, &Action::MoveCursorRight);
    }

    // Verify cursor is at the end of the text
    assert_eq!(text_state.cursor_char_index(), Some(5));
    assert_eq!(text_state.text_char_len(), 5);

    // Try to delete a character at the end of the text
    // This should panic due to the bug
    let result = text_state.apply_action(&mut ctx, &Action::DeleteBackward);

    // The following assertions should not be reached if the code panics
    assert!(matches!(result, ActionResult::TextChanged));
    assert_eq!(text_state.text_char_len(), 4);
    assert_eq!(text_state.text(), "Hell");
    assert_eq!(text_state.cursor_char_index(), Some(4));
}

#[test]
pub fn test_insert_newline_at_end_of_text() {
    // Test for the bug: If the caret is at the very end of the string and trying to delete a character,
    // the code panics
    let mut ctx = TextContext::default();
    let text_id = Id::new(0);
    let initial_text = "Hello".to_string();

    let mut text_state = TextState::new_with_text(initial_text, text_id, &mut ctx.font_system);
    text_state.set_style(&mono_style_test());
    text_state.set_outer_size(&Point::from((200.0, 25.0)));
    text_state.is_editable = true;
    text_state.is_editing = true;
    text_state.is_selectable = true;
    text_state.are_actions_enabled = true;

    text_state.recalculate(&mut ctx);

    // Move cursor to the end of the text
    for _ in 0..5 {
        text_state.apply_action(&mut ctx, &Action::MoveCursorRight);
    }

    // Verify cursor is at the end of the text
    assert_eq!(text_state.cursor_char_index(), Some(5));
    assert_eq!(text_state.text_char_len(), 5);

    // Try to delete a character at the end of the text
    // This should panic due to the bug
    let result = text_state.apply_action(&mut ctx, &Action::InsertChar("\n".into()));

    // The following assertions should not be reached if the code panics
    assert!(matches!(result, ActionResult::TextChanged));
    assert_eq!(text_state.text_char_len(), 6);
    assert_eq!(text_state.text(), "Hello\n");
    assert_eq!(text_state.cursor_char_index(), Some(6));
}
