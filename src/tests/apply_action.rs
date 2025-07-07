use crate::tests::mono_style_test;
use crate::{Action, ActionResult, Point, TextContext, TextState};

// Test when actions are disabled
#[test]
pub fn test_actions_disabled() {
    let mut ctx = TextContext::default();
    let initial_text = "Hello World".to_string();

    let mut text_state = TextState::new_with_text(initial_text, &mut ctx.font_system, ());
    text_state.set_style(&mono_style_test());
    text_state.set_outer_size(&Point::from((200.0, 25.0)));

    // Ensure actions are disabled
    text_state.are_actions_enabled = false;
    text_state.recalculate(&mut ctx);

    // Test all action types
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::InsertChar("x".into())),
        ActionResult::ActionsDisabled
    ));
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::MoveCursorRight),
        ActionResult::ActionsDisabled
    ));
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::MoveCursorLeft),
        ActionResult::ActionsDisabled
    ));
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::MoveCursorUp),
        ActionResult::ActionsDisabled
    ));
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::MoveCursorDown),
        ActionResult::ActionsDisabled
    ));
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::DeleteBackward),
        ActionResult::ActionsDisabled
    ));
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::Paste("test".to_string())),
        ActionResult::ActionsDisabled
    ));
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::Cut),
        ActionResult::ActionsDisabled
    ));
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::CopySelectedText),
        ActionResult::ActionsDisabled
    ));
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::SelectAll),
        ActionResult::ActionsDisabled
    ));
}

// Test when text is not selectable
#[test]
pub fn test_not_selectable() {
    let mut ctx = TextContext::default();
    let initial_text = "Hello World".to_string();

    let mut text_state = TextState::new_with_text(initial_text, &mut ctx.font_system, ());
    text_state.set_style(&mono_style_test());
    text_state.set_outer_size(&Point::from((200.0, 25.0)));

    // Enable actions but disable selectability
    text_state.are_actions_enabled = true;
    text_state.is_selectable = false;
    text_state.recalculate(&mut ctx);

    // Test all action types - all should return None
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::InsertChar("x".into())),
        ActionResult::None
    ));
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::MoveCursorRight),
        ActionResult::None
    ));
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::MoveCursorLeft),
        ActionResult::None
    ));
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::MoveCursorUp),
        ActionResult::None
    ));
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::MoveCursorDown),
        ActionResult::None
    ));
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::DeleteBackward),
        ActionResult::None
    ));
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::Paste("test".to_string())),
        ActionResult::None
    ));
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::Cut),
        ActionResult::None
    ));
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::CopySelectedText),
        ActionResult::None
    ));
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::SelectAll),
        ActionResult::None
    ));
}

// Test when text is selectable but not editable
#[test]
pub fn test_selectable_not_editable() {
    let mut ctx = TextContext::default();
    let initial_text = "Hello World".to_string();

    let mut text_state = TextState::new_with_text(initial_text, &mut ctx.font_system, ());
    text_state.set_style(&mono_style_test());
    text_state.set_outer_size(&Point::from((200.0, 25.0)));

    // Enable actions and selectability, but disable editability
    text_state.are_actions_enabled = true;
    text_state.is_selectable = true;
    text_state.is_editable = false;
    text_state.recalculate(&mut ctx);

    // Test edit actions - should return None
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::InsertChar("x".into())),
        ActionResult::None
    ));
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::MoveCursorRight),
        ActionResult::None
    ));
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::MoveCursorLeft),
        ActionResult::None
    ));
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::MoveCursorUp),
        ActionResult::None
    ));
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::MoveCursorDown),
        ActionResult::None
    ));
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::DeleteBackward),
        ActionResult::None
    ));
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::Paste("test".to_string())),
        ActionResult::None
    ));
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::Cut),
        ActionResult::None
    ));

    // Test selection actions - should work
    let select_all_result = text_state.apply_action(&mut ctx, &Action::SelectAll);
    assert!(matches!(select_all_result, ActionResult::CursorUpdated));

    // After selecting all, copy should work
    let copy_result = text_state.apply_action(&mut ctx, &Action::CopySelectedText);
    assert!(matches!(copy_result, ActionResult::InsertToClipboard(s) if s == "Hello World"));
}

// Test when text is both selectable and editable
#[test]
pub fn test_selectable_and_editable() {
    let mut ctx = TextContext::default();
    let initial_text = "Hello World".to_string();

    let mut text_state = TextState::new_with_text(initial_text, &mut ctx.font_system, ());
    text_state.set_style(&mono_style_test());
    text_state.set_outer_size(&Point::from((200.0, 25.0)));

    // Enable everything
    text_state.are_actions_enabled = true;
    text_state.is_selectable = true;
    text_state.is_editable = true;
    text_state.is_editing = true;
    text_state.recalculate(&mut ctx);

    // Test cursor movement
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::MoveCursorRight),
        ActionResult::CursorUpdated
    ));
    assert_eq!(text_state.cursor_char_index(), Some(1));

    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::MoveCursorLeft),
        ActionResult::CursorUpdated
    ));
    assert_eq!(text_state.cursor_char_index(), Some(0));

    // Test insert character
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::InsertChar("X".into())),
        ActionResult::TextChanged
    ));
    assert_eq!(text_state.text(), "XHello World");

    // Test delete backward
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::DeleteBackward),
        ActionResult::TextChanged
    ));
    assert_eq!(text_state.text(), "Hello World");

    // Test select all and cut
    text_state.apply_action(&mut ctx, &Action::SelectAll);
    assert!(text_state.is_text_selected());

    // After selecting all, copy should work
    let copy_result = text_state.apply_action(&mut ctx, &Action::CopySelectedText);
    assert!(matches!(copy_result, ActionResult::InsertToClipboard(s) if s == "Hello World"));

    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::Cut),
        ActionResult::InsertToClipboard(s) if s == "Hello World"
    ));
    assert_eq!(text_state.text(), "");

    // Test paste
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::Paste("Pasted Text".to_string())),
        ActionResult::TextChanged
    ));
    assert_eq!(text_state.text(), "Pasted Text");
}

// Test cursor movement with selection
#[test]
pub fn test_cursor_movement_with_selection() {
    let mut ctx = TextContext::default();
    let initial_text = "Hello World".to_string();

    let mut text_state = TextState::new_with_text(initial_text, &mut ctx.font_system, ());
    text_state.set_style(&mono_style_test());
    text_state.set_outer_size(&Point::from((200.0, 25.0)));

    // Enable everything
    text_state.are_actions_enabled = true;
    text_state.is_selectable = true;
    text_state.is_editable = true;
    text_state.is_editing = true;
    text_state.recalculate(&mut ctx);

    // Select all text
    text_state.apply_action(&mut ctx, &Action::SelectAll);
    assert!(text_state.is_text_selected());

    // Move cursor right should move to the end of selection and clear selection
    text_state.apply_action(&mut ctx, &Action::MoveCursorRight);
    assert!(!text_state.is_text_selected());
    assert_eq!(text_state.cursor_char_index(), Some(11)); // End of "Hello World"

    // Select all again
    text_state.apply_action(&mut ctx, &Action::SelectAll);
    assert!(text_state.is_text_selected());

    // Move cursor left should move to the beginning of selection and clear selection
    text_state.apply_action(&mut ctx, &Action::MoveCursorLeft);
    assert!(!text_state.is_text_selected());
    assert_eq!(text_state.cursor_char_index(), Some(0)); // Beginning of text
}

// Test vertical cursor movement
#[test]
pub fn test_vertical_cursor_movement() {
    let mut ctx = TextContext::default();
    let initial_text = "Line 1\nLine 2\nLine 3".to_string();

    let mut text_state = TextState::new_with_text(initial_text, &mut ctx.font_system, ());
    text_state.set_style(&mono_style_test());
    text_state.set_outer_size(&Point::from((200.0, 100.0)));

    // Enable everything
    text_state.are_actions_enabled = true;
    text_state.is_selectable = true;
    text_state.is_editable = true;
    text_state.is_editing = true;
    text_state.recalculate(&mut ctx);

    // Initial position
    assert_eq!(text_state.cursor_char_index(), Some(0));

    // Move down
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::MoveCursorDown),
        ActionResult::CursorUpdated
    ));

    // Should be at the beginning of the second line
    let pos_after_down = text_state.cursor_char_index();
    assert!(pos_after_down.is_some());
    assert!(pos_after_down.unwrap() > 6); // After "Line 1\n"

    // Move up
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::MoveCursorUp),
        ActionResult::CursorUpdated
    ));

    // Should be back at the beginning of the first line
    assert_eq!(text_state.cursor_char_index(), Some(0));
}

// Test delete with selection
#[test]
pub fn test_delete_with_selection() {
    let mut ctx = TextContext::default();
    let initial_text = "Hello World".to_string();

    let mut text_state = TextState::new_with_text(initial_text, &mut ctx.font_system, ());
    text_state.set_style(&mono_style_test());
    text_state.set_outer_size(&Point::from((200.0, 25.0)));

    // Enable everything
    text_state.are_actions_enabled = true;
    text_state.is_selectable = true;
    text_state.is_editable = true;
    text_state.is_editing = true;
    text_state.recalculate(&mut ctx);

    // Select "Hello"
    text_state.handle_press(&mut ctx, Point { x: 0.0, y: 10.0 });
    text_state.handle_drag(&mut ctx, true, Point { x: 40.0, y: 10.0 });
    assert!(text_state.is_text_selected());

    // Delete selected text
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::DeleteBackward),
        ActionResult::TextChanged
    ));

    // "Hello" should be deleted, leaving " World"
    assert_eq!(text_state.text(), " World");
    assert!(!text_state.is_text_selected());
}

// Test insert character with selection
#[test]
pub fn test_insert_char_with_selection() {
    let mut ctx = TextContext::default();
    let initial_text = "Hello World".to_string();

    let mut text_state = TextState::new_with_text(initial_text, &mut ctx.font_system, ());
    text_state.set_style(&mono_style_test());
    text_state.set_outer_size(&Point::from((200.0, 25.0)));

    // Enable everything
    text_state.are_actions_enabled = true;
    text_state.is_selectable = true;
    text_state.is_editable = true;
    text_state.is_editing = true;
    text_state.recalculate(&mut ctx);

    // Select "Hello"
    text_state.handle_press(&mut ctx, Point { x: 0.0, y: 10.0 });
    text_state.handle_drag(&mut ctx, true, Point { x: 40.0, y: 10.0 });
    assert!(text_state.is_text_selected());

    // Insert character (should replace selection)
    assert!(matches!(
        text_state.apply_action(&mut ctx, &Action::InsertChar("X".into())),
        ActionResult::TextChanged
    ));

    // "Hello" should be replaced with "X", resulting in "X World"
    assert_eq!(text_state.text(), "X World");
    assert!(!text_state.is_text_selected());
}
