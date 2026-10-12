use crate::math::Size;
use crate::tests::mono_style_test;
use crate::{Action, ActionResult, TextContext, TextState};

fn editable_state(text: &str, ctx: &mut TextContext) -> TextState<()> {
    let mut state = TextState::new_with_text(text, &mut ctx.font_system, ());
    state.set_style(&mono_style_test());
    state.set_outer_size(&Size::new(300.0, 100.0));
    state.is_editable = true;
    state.is_selectable = true;
    state.are_actions_enabled = true;
    state.recalculate(ctx);
    state
}

#[test]
fn replacement_clears_selection_before_recalculation() {
    let mut ctx = TextContext::default();

    for replacement in ["x", "", "a much longer replacement", "\u{e9}"] {
        let mut state = editable_state("Hello World", &mut ctx);
        state.apply_action(&mut ctx, &Action::SelectAll);
        assert_eq!(state.selected_text(), Some("Hello World"));
        assert!(!state.selection().lines().is_empty());

        state.set_text(replacement);

        assert_eq!(state.selected_text(), None);
        assert!(!state.is_text_selected());
        assert!(state.selection().is_empty());
        assert!(state.selection().lines().is_empty());
        assert_eq!(
            state.apply_action(&mut ctx, &Action::CopySelectedText),
            ActionResult::TextCopied(String::new())
        );

        state.recalculate(&mut ctx);
        assert_eq!(state.selected_text(), None);
        assert!(state.selection().lines().is_empty());
        assert_eq!(
            state.apply_action(&mut ctx, &Action::InsertChar("z".into())),
            ActionResult::TextChanged
        );
        assert_eq!(state.text(), format!("z{replacement}"));
    }
}

#[test]
fn replacement_clamps_cursor_to_a_utf8_boundary_before_insertion() {
    let mut ctx = TextContext::default();
    let mut state = editable_state("ab", &mut ctx);
    state.apply_action(&mut ctx, &Action::MoveCursorRight);
    assert_eq!(state.cursor_char_index(), Some(1));

    state.set_text("\u{e9}");

    assert_eq!(state.cursor_char_index(), Some(0));
    assert_eq!(state.caret_position_relative(), None);
    assert_eq!(
        state.apply_action(&mut ctx, &Action::InsertChar("x".into())),
        ActionResult::TextChanged
    );
    assert_eq!(state.text(), "x\u{e9}");
    assert_eq!(state.cursor_char_index(), Some(1));
}

#[test]
fn replacement_clamps_cursor_to_visible_text_end() {
    let mut ctx = TextContext::default();

    for replacement in ["x", "", "x\n", "\u{e9}"] {
        let mut state = editable_state("Hello World", &mut ctx);
        state.apply_action(&mut ctx, &Action::SelectAll);
        state.apply_action(&mut ctx, &Action::MoveCursorRight);
        assert_eq!(state.cursor_char_index(), Some(11));

        state.set_text(replacement);

        assert_eq!(state.cursor_char_index(), Some(replacement.chars().count()));
        state.recalculate(&mut ctx);
        assert_eq!(state.cursor_char_index(), Some(replacement.chars().count()));
        assert_eq!(
            state.apply_action(&mut ctx, &Action::Paste("z".into())),
            ActionResult::TextChanged
        );
        assert_eq!(state.text(), format!("{replacement}z"));
        assert_eq!(
            state.cursor_char_index(),
            Some(replacement.chars().count() + 1)
        );
    }
}

#[test]
fn replacement_refreshes_cursor_when_line_structure_changes() {
    let mut ctx = TextContext::default();
    let mut state = editable_state("abcd", &mut ctx);
    let first_line_y = state.caret_position_relative().unwrap().y;
    state.apply_action(&mut ctx, &Action::MoveCursorRight);
    state.apply_action(&mut ctx, &Action::MoveCursorRight);
    assert_eq!(state.cursor_char_index(), Some(2));

    state.set_text("a\nbc");
    state.recalculate(&mut ctx);

    assert_eq!(state.cursor_char_index(), Some(2));
    assert!(state.caret_position_relative().unwrap().y > first_line_y);
    assert_eq!(
        state.apply_action(&mut ctx, &Action::InsertChar("x".into())),
        ActionResult::TextChanged
    );
    assert_eq!(state.text(), "a\nxbc");
    assert_eq!(state.cursor_char_index(), Some(3));

    state.set_text("abcd");
    state.recalculate(&mut ctx);

    assert_eq!(state.cursor_char_index(), Some(3));
    assert_eq!(state.caret_position_relative().unwrap().y, first_line_y);
    state.apply_action(&mut ctx, &Action::InsertChar("y".into()));
    assert_eq!(state.text(), "abcyd");
    assert_eq!(state.cursor_char_index(), Some(4));
}

#[test]
fn replacement_preserves_a_valid_cursor_byte_offset() {
    let mut ctx = TextContext::default();
    let mut state = editable_state("abcd", &mut ctx);
    state.apply_action(&mut ctx, &Action::MoveCursorRight);
    state.apply_action(&mut ctx, &Action::MoveCursorRight);

    state.set_text("\u{e9}xy");
    state.recalculate(&mut ctx);

    assert_eq!(state.cursor_char_index(), Some(1));
    state.apply_action(&mut ctx, &Action::DeleteBackward);
    assert_eq!(state.text(), "xy");
    assert_eq!(state.cursor_char_index(), Some(0));
}

#[test]
fn identical_text_preserves_cursor_selection_and_caret() {
    let mut ctx = TextContext::default();
    let mut state = editable_state("Hello World", &mut ctx);
    state.apply_action(&mut ctx, &Action::MoveCursorRight);
    state.apply_action(&mut ctx, &Action::SelectAll);
    let caret_position = state.caret_position_relative();
    let selection_line_count = state.selection().lines().len();

    state.set_text("Hello World");

    assert_eq!(state.cursor_char_index(), Some(1));
    assert_eq!(state.selected_text(), Some("Hello World"));
    assert_eq!(state.caret_position_relative(), caret_position);
    assert_eq!(state.selection().lines().len(), selection_line_count);
}
