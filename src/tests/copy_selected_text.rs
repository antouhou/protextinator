use crate::state::UpdateReason;
use crate::tests::mono_style_test;
use crate::{Action, ActionResult, Id, Point, TextContext, TextState};

#[test]
pub fn test_copy_empty_selection() {
    let mut ctx = TextContext::default();
    let text_id = Id::new(0);
    let initial_text = "Hello World".to_string();

    let mut text_state = TextState::new_with_text(initial_text, text_id);

    text_state.params.set_style(&mono_style_test());
    text_state.params.set_size(&Point::from((200.0, 25.0)));
    text_state.is_editable = true;
    text_state.is_editing = true;
    text_state.is_selectable = true;
    text_state.are_actions_enabled = true;

    text_state.recalculate(&mut ctx, UpdateReason::Unknown);

    // No selection, cursor at the beginning
    assert_eq!(text_state.cursor.char_index(text_state.text()), Some(0));
    assert!(!text_state.is_text_selected());

    // Try to copy with no selection
    let result = text_state.apply_action(&mut ctx, &Action::CopySelectedText);
    assert!(matches!(result, ActionResult::InsertToClipboard(s) if s.is_empty()));
}

#[test]
pub fn test_copy_partial_selection() {
    let mut ctx = TextContext::default();
    let text_id = Id::new(0);
    let initial_text = "Hello World".to_string();

    let mut text_state = TextState::new_with_text(initial_text, text_id);
    text_state.params.set_style(&mono_style_test());
    text_state.params.set_size(&Point::from((200.0, 25.0)));
    text_state.is_editable = true;
    text_state.is_editing = true;
    text_state.is_selectable = true;
    text_state.are_actions_enabled = true;

    text_state.recalculate(&mut ctx, UpdateReason::Unknown);

    // Set up a selection by clicking and dragging
    text_state.handle_press(&mut ctx, Point { x: 0.0, y: 10.0 });
    assert_eq!(text_state.cursor.char_index(text_state.text()), Some(0));

    // Simulate dragging to select "Hello"
    text_state.handle_drag(&mut ctx, true, Point { x: 40.0, y: 10.0 });
    assert!(text_state.is_text_selected());

    // Copy the selected text
    let result = text_state.apply_action(&mut ctx, &Action::CopySelectedText);
    assert!(matches!(result, ActionResult::InsertToClipboard(s) if s == "Hello"));
}

#[test]
pub fn test_copy_full_selection() {
    let mut ctx = TextContext::default();
    let text_id = Id::new(0);
    let initial_text = "Hello World".to_string();

    let mut text_state = TextState::new_with_text(initial_text, text_id);
    text_state.params.set_style(&mono_style_test());
    text_state.params.set_size(&Point::from((200.0, 25.0)));
    text_state.is_editable = true;
    text_state.is_editing = true;
    text_state.is_selectable = true;
    text_state.are_actions_enabled = true;

    text_state.recalculate(&mut ctx, UpdateReason::Unknown);

    // Select all text
    let result = text_state.apply_action(&mut ctx, &Action::SelectAll);
    assert!(matches!(result, ActionResult::CursorUpdated));
    assert!(text_state.is_text_selected());

    // Copy the selected text
    let result = text_state.apply_action(&mut ctx, &Action::CopySelectedText);
    match result {
        ActionResult::InsertToClipboard(s) => assert_eq!(s, "Hello World"),
        _ => panic!("Result is {result:?}, expected InsertToClipboard"),
    }
}

#[test]
pub fn test_copy_cyrillic_text() {
    let mut ctx = TextContext::default();
    let text_id = Id::new(0);
    let initial_text = "Привет Мир".to_string();

    let mut text_state = TextState::new_with_text(initial_text, text_id);
    text_state.params.set_style(&mono_style_test());
    text_state.params.set_size(&Point::from((200.0, 25.0)));
    text_state.is_editable = true;
    text_state.is_editing = true;
    text_state.is_selectable = true;
    text_state.are_actions_enabled = true;

    text_state.recalculate(&mut ctx, UpdateReason::Unknown);

    // Set up a selection by clicking and dragging
    text_state.handle_press(&mut ctx, Point { x: 0.0, y: 10.0 });
    assert_eq!(text_state.cursor.char_index(text_state.text()), Some(0));

    // Simulate dragging to select "Привет"
    text_state.handle_drag(&mut ctx, true, Point { x: 50.0, y: 10.0 });
    assert!(text_state.is_text_selected());

    // Copy the selected text
    let result = text_state.apply_action(&mut ctx, &Action::CopySelectedText);
    assert!(matches!(result, ActionResult::InsertToClipboard(s) if s == "Привет"));
}

#[test]
pub fn test_copy_after_editing() {
    let mut ctx = TextContext::default();
    let text_id = Id::new(0);
    let initial_text = "Hello World".to_string();

    let mut text_state = TextState::new_with_text(initial_text, text_id);
    text_state.params.set_style(&mono_style_test());
    text_state.params.set_size(&Point::from((200.0, 25.0)));

    text_state.is_editable = true;
    text_state.is_editing = true;
    text_state.is_selectable = true;
    text_state.are_actions_enabled = true;

    text_state.recalculate(&mut ctx, UpdateReason::Unknown);

    // Insert text at the beginning
    text_state.apply_action(&mut ctx, &Action::InsertChar("Test ".into()));
    assert_eq!(text_state.text(), "Test Hello World");

    // Set up a selection by clicking and dragging
    text_state.handle_press(&mut ctx, Point { x: 0.0, y: 10.0 });
    assert_eq!(text_state.cursor.char_index(text_state.text()), Some(0));

    // Simulate dragging to select "Test "
    text_state.handle_drag(&mut ctx, true, Point { x: 40.0, y: 10.0 });
    assert!(text_state.is_text_selected());

    // Copy the selected text
    let result = text_state.apply_action(&mut ctx, &Action::CopySelectedText);
    assert!(matches!(result, ActionResult::InsertToClipboard(s) if s == "Test "));
}

#[test]
pub fn test_copy_selection_from_middle() {
    let mut ctx = TextContext::default();
    let text_id = Id::new(0);
    let initial_text = "The quick brown fox jumps over the lazy dog".to_string();

    let mut text_state = TextState::new_with_text(initial_text, text_id);
    text_state.params.set_style(&mono_style_test());
    text_state.params.set_size(&Point::from((400.0, 25.0)));
    text_state.is_editable = true;
    text_state.is_editing = true;
    text_state.is_selectable = true;
    text_state.are_actions_enabled = true;

    text_state.recalculate(&mut ctx, UpdateReason::Unknown);

    // Position cursor before the middle of the line (at 'b' in "brown")
    // First, get the mono width to calculate the position
    let mono_width = ctx.buffer_cache.first_glyph(&text_id).unwrap().w;
    let position_x = mono_width * 10.0; // Position at "b" in "brown"

    // Set up a selection by clicking at the position
    text_state.handle_press(
        &mut ctx,
        Point {
            x: position_x,
            y: 10.0,
        },
    );

    // Verify cursor position after clicking
    let cursor_pos_after_click = text_state.cursor.char_index(text_state.text());
    assert_eq!(cursor_pos_after_click, Some(10)); // Should be at 'b' in "brown"

    // Drag for a couple of symbols (select "bro")
    let drag_position_x = position_x + (mono_width * 3.0); // Drag to include 3 more characters
    text_state.handle_drag(
        &mut ctx,
        true,
        Point {
            x: drag_position_x,
            y: 10.0,
        },
    );

    // Verify text is selected
    assert!(text_state.is_text_selected());

    // Get the selected text and verify it's correct
    let selected_text = text_state.selected_text();
    assert!(selected_text.is_some());
    assert_eq!(selected_text.unwrap(), "bro");

    // Copy the selected text
    let result = text_state.apply_action(&mut ctx, &Action::CopySelectedText);

    // Verify the copy operation was successful and copied the correct text
    assert!(matches!(result, ActionResult::InsertToClipboard(s) if s == "bro"));
}
