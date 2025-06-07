use crate::state::UpdateReason;
use crate::tests::mono_style_test;
use crate::{Action, ActionResult, Id, Point, Rect, TextContext, TextState};

#[test]
pub fn test() {
    let mut ctx = TextContext::default();
    let text_id = Id::new(0);
    let initial_text = "Hello World".to_string();

    let mut text_state = TextState::new_with_text(initial_text, text_id);
    text_state.text_style = mono_style_test();
    text_state.is_editable = true;
    text_state.is_editing = true;
    text_state.is_selectable = true;
    text_state.text_area = Rect::from(((0.0, 0.0), (200.0, 25.0)));
    text_state.recalculate(&mut ctx, true, UpdateReason::Unknown);
    let mono_width = ctx
        .text_manager
        .get_position_of_last_glyph(&text_id)
        .unwrap()
        .width;
    assert!(mono_width > 0.0);

    assert_eq!(text_state.cursor_before_glyph, 0);
    assert_eq!(text_state.relative_caret_offset_horizontal, 0.0);

    text_state.handle_click(&mut ctx, Point { x: 25.0, y: 10.0 });
    assert_eq!(text_state.cursor_before_glyph, 3);
    assert_eq!(
        text_state.relative_caret_offset_horizontal,
        mono_width * 3.0
    );

    let result = text_state.apply_action(&mut ctx, &Action::InsertChar("a".into()));
    assert!(matches!(result, ActionResult::TextChanged));
    assert_eq!(text_state.text_size(), 12);
    assert_eq!(text_state.cursor_before_glyph, 4);
    assert_eq!(
        text_state.relative_caret_offset_horizontal,
        mono_width * 4.0
    );
    assert_eq!(text_state.text(), "Helalo World");

    let result = text_state.apply_action(&mut ctx, &Action::MoveCursorRight);
    assert!(matches!(result, ActionResult::CursorUpdated));
    assert_eq!(text_state.text_size(), 12);
    assert_eq!(text_state.cursor_before_glyph, 5);
    assert_eq!(
        text_state.relative_caret_offset_horizontal,
        mono_width * 5.0
    );
}

#[test]
pub fn test_cyrillic() {
    let mut ctx = TextContext::default();
    let text_id = Id::new(0);
    let initial_text = "Привет Мир".to_string();

    let mut text_state = TextState::new_with_text(initial_text, text_id);
    text_state.text_style = mono_style_test();
    text_state.is_editable = true;
    text_state.is_editing = true;
    text_state.is_selectable = true;
    text_state.text_area = Rect::from(((0.0, 0.0), (200.0, 25.0)));
    text_state.recalculate(&mut ctx, true, UpdateReason::Unknown);
    let mono_width = ctx
        .text_manager
        .get_position_of_last_glyph(&text_id)
        .unwrap()
        .width;
    assert!(mono_width > 0.0);

    assert_eq!(text_state.text_size(), 10);
    assert_eq!(text_state.cursor_before_glyph, 0);
    assert_eq!(text_state.relative_caret_offset_horizontal, 0.0);

    // let result = text_state.apply_action(&mut ctx, &Action::MoveCursorRight);
    // assert!(matches!(result, ActionResult::CursorUpdated));
    // assert_eq!(text_state.text_size(), 10);
    // assert_eq!(text_state.cursor_before_glyph, 1);
    // assert_eq!(text_state.relative_caret_offset_horizontal, mono_width * 1.0);
    //
    // let result = text_state.apply_action(&mut ctx, &Action::MoveCursorRight);
    // assert!(matches!(result, ActionResult::CursorUpdated));
    // assert_eq!(text_state.text_size(), 10);
    // assert_eq!(text_state.cursor_before_glyph, 2);
    // assert_eq!(text_state.relative_caret_offset_horizontal, mono_width * 2.0);

    text_state.handle_click(&mut ctx, Point { x: 25.0, y: 10.0 });
    assert_eq!(text_state.cursor_before_glyph, 3);
    assert_eq!(
        text_state.relative_caret_offset_horizontal,
        mono_width * 3.0
    );

    let result = text_state.apply_action(&mut ctx, &Action::InsertChar("ш".into()));
    assert!(matches!(result, ActionResult::TextChanged));
    assert_eq!(text_state.text_size(), 11);
    assert_eq!(text_state.cursor_before_glyph, 4);
    assert_eq!(
        text_state.relative_caret_offset_horizontal,
        mono_width * 4.0
    );
    assert_eq!(text_state.text(), "Пришвет Мир");

    let result = text_state.apply_action(&mut ctx, &Action::MoveCursorRight);
    assert!(matches!(result, ActionResult::CursorUpdated));
    assert_eq!(text_state.text_size(), 11);
    assert_eq!(text_state.cursor_before_glyph, 5);
    assert_eq!(
        text_state.relative_caret_offset_horizontal,
        mono_width * 5.0
    );
}
