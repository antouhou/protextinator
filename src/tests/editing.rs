use crate::math::Size;
use crate::tests::mono_style_test;
use crate::{Action, ActionResult, Point, TextContext, TextState};

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

fn select_middle_character(state: &mut TextState<()>, ctx: &mut TextContext, is_reversed: bool) {
    let (start, end) = {
        let run = state
            .buffer()
            .layout_runs()
            .find(|run| run.text == "x")
            .unwrap();
        let glyph = run.glyphs.first().unwrap();
        let y = run.line_top + run.line_height / 2.0;
        (
            Point::new(glyph.x, y),
            Point::new(glyph.x + glyph.w + 1.0, y),
        )
    };
    let (origin, end) = if is_reversed {
        (end, start)
    } else {
        (start, end)
    };
    state.handle_press(ctx, origin);
    state.handle_drag(ctx, true, end);
    assert_eq!(state.selected_text(), Some("x"));
}

#[test]
fn dragging_below_trailing_newlines_selects_only_public_text() {
    let mut ctx = TextContext::default();
    for text in ["a\n", "a\r\n", "\u{e9}\n", "\n", "a\n\n"] {
        let mut state = editable_state(text, &mut ctx);
        state.handle_press(&mut ctx, Point::new(0.0, 5.0));
        assert_eq!(state.cursor_char_index(), Some(0));

        state.handle_drag(&mut ctx, true, Point::new(0.0, 200.0));

        assert_eq!(state.selected_text(), Some(text));
        assert_eq!(
            state.apply_action(&mut ctx, &Action::CopySelectedText),
            ActionResult::TextCopied(text.into())
        );
        state.apply_action(&mut ctx, &Action::Cut);
        assert_eq!(state.text(), "");
        assert_eq!(state.cursor_char_index(), Some(0));
        state.apply_action(&mut ctx, &Action::InsertChar("xy".into()));
        assert_eq!(state.text(), "xy");
        assert_eq!(state.cursor_char_index(), Some(2));
    }
}

#[test]
fn navigation_cannot_pass_public_text_end() {
    let mut ctx = TextContext::default();
    for text in ["a\n", "a\r\n", "\u{e9}\n", "\n", "a\n\n"] {
        for motion in [Action::MoveCursorRight, Action::MoveCursorDown] {
            let mut state = editable_state(text, &mut ctx);
            for _ in 0..5 {
                state.apply_action(&mut ctx, &motion);
                assert!(state.cursor_char_index().unwrap() <= state.text_char_len());
            }
            assert_eq!(state.cursor_char_index(), Some(text.chars().count()));

            state.apply_action(&mut ctx, &Action::InsertChar("x".into()));
            state.apply_action(&mut ctx, &Action::InsertChar("y".into()));
            assert_eq!(state.text(), format!("{text}xy"));
            assert_eq!(state.cursor_char_index(), Some(text.chars().count() + 2));
        }
    }
}

#[test]
fn cutting_a_range_that_forms_crlf_normalizes_the_cursor() {
    let mut ctx = TextContext::default();
    let mut state = editable_state("a\rx\nb", &mut ctx);
    state.apply_action(&mut ctx, &Action::SelectAll);
    state.apply_action(&mut ctx, &Action::MoveCursorRight);

    assert_eq!(
        state.apply_action(&mut ctx, &Action::CutRange { start: 2, end: 3 }),
        ActionResult::TextCut("x".into())
    );
    assert_eq!(state.text(), "a\r\nb");
    assert_eq!(state.cursor_char_index(), Some(1));

    state.apply_action(&mut ctx, &Action::InsertChar("xy".into()));
    assert_eq!(state.text(), "axy\r\nb");
    assert_eq!(state.cursor_char_index(), Some(3));
}

#[test]
fn backspacing_between_line_endings_normalizes_the_cursor() {
    let mut ctx = TextContext::default();
    for (text, joined, edited) in [
        ("a\rx\nb", "a\r\nb", "axy\r\nb"),
        ("a\nx\rb", "a\n\rb", "axy\n\rb"),
    ] {
        let mut state = editable_state(text, &mut ctx);
        for _ in 0..3 {
            state.apply_action(&mut ctx, &Action::MoveCursorRight);
        }
        assert_eq!(state.cursor_char_index(), Some(3));

        state.apply_action(&mut ctx, &Action::DeleteBackward);

        assert_eq!(state.text(), joined);
        assert_eq!(state.cursor_char_index(), Some(1));
        state.apply_action(&mut ctx, &Action::InsertChar("xy".into()));
        assert_eq!(state.text(), edited);
        assert_eq!(state.cursor_char_index(), Some(3));
    }
}

#[test]
fn deleting_a_selection_that_forms_crlf_normalizes_the_cursor() {
    let mut ctx = TextContext::default();
    for action in [
        Action::Cut,
        Action::DeleteBackward,
        Action::Paste(String::new()),
        Action::InsertChar("".into()),
    ] {
        let mut state = editable_state("a\rx\nb", &mut ctx);
        select_middle_character(&mut state, &mut ctx, false);

        state.apply_action(&mut ctx, &action);

        assert_eq!(state.text(), "a\r\nb");
        assert_eq!(state.cursor_char_index(), Some(1));
        assert!(state.selection().is_empty());
        state.apply_action(&mut ctx, &Action::InsertChar("xy".into()));
        assert_eq!(state.text(), "axy\r\nb");
        assert_eq!(state.cursor_char_index(), Some(3));
    }
}

#[test]
fn selection_replacement_keeps_the_original_insertion_position() {
    let mut ctx = TextContext::default();
    for (text, replaced, edited) in [
        ("a\rx\nb", "a\rz\nb", "a\rzxy\nb"),
        ("a\nx\rb", "a\nz\rb", "a\nzxy\rb"),
    ] {
        for action in [Action::Paste("z".into()), Action::InsertChar("z".into())] {
            for is_reversed in [false, true] {
                let mut state = editable_state(text, &mut ctx);
                select_middle_character(&mut state, &mut ctx, is_reversed);

                state.apply_action(&mut ctx, &action);

                assert_eq!(state.text(), replaced);
                assert_eq!(state.cursor_char_index(), Some(3));
                assert!(state.selection().is_empty());
                state.apply_action(&mut ctx, &Action::InsertChar("xy".into()));
                assert_eq!(state.text(), edited);
                assert_eq!(state.cursor_char_index(), Some(5));
            }
        }
    }
}

#[test]
fn right_arrow_advances_after_insertion_before_a_combining_mark() {
    let mut ctx = TextContext::default();
    for action in [Action::Paste("a".into()), Action::InsertChar("a".into())] {
        let mut state = editable_state("\u{301}x", &mut ctx);
        state.apply_action(&mut ctx, &action);
        assert_eq!(state.text(), "a\u{301}x");

        state.apply_action(&mut ctx, &Action::MoveCursorRight);

        assert_eq!(state.cursor_char_index(), Some(3));
        state.apply_action(&mut ctx, &Action::MoveCursorLeft);
        assert_eq!(state.cursor_char_index(), Some(2));
        state.apply_action(&mut ctx, &Action::InsertChar("y".into()));
        assert_eq!(state.text(), "a\u{301}yx");
        assert_eq!(state.cursor_char_index(), Some(3));
    }
}

#[test]
fn insertion_leaves_the_cursor_after_a_complete_grapheme() {
    let mut ctx = TextContext::default();
    for (text, inserted, cluster_char_count) in [
        ("\u{301}\u{308}x", "a", 3),
        ("\u{200d}\u{1f4bb}x", "\u{1f469}", 3),
        ("\u{1f3fd}x", "\u{1f44d}", 2),
        ("\u{1f1f8}x", "\u{1f1fa}", 2),
    ] {
        for action in [
            Action::Paste(inserted.into()),
            Action::InsertChar(inserted.into()),
        ] {
            let mut state = editable_state(text, &mut ctx);

            state.apply_action(&mut ctx, &action);

            assert_eq!(state.text(), format!("{inserted}{text}"));
            assert_eq!(state.cursor_char_index(), Some(cluster_char_count));
            state.apply_action(&mut ctx, &Action::MoveCursorRight);
            assert_eq!(state.cursor_char_index(), Some(cluster_char_count + 1));
            state.apply_action(&mut ctx, &Action::MoveCursorLeft);
            state.apply_action(&mut ctx, &Action::MoveCursorLeft);
            assert_eq!(state.cursor_char_index(), Some(0));
        }
    }
}

#[test]
fn inserting_a_paired_line_ending_normalizes_the_cursor() {
    let mut ctx = TextContext::default();
    for (text, action, joined, edited) in [
        ("a\nb", Action::Paste("\r".into()), "a\r\nb", "a\r\nxyb"),
        (
            "a\nb",
            Action::InsertChar("\r".into()),
            "a\r\nb",
            "a\r\nxyb",
        ),
        ("a\rb", Action::Paste("\n".into()), "a\n\rb", "a\n\rxyb"),
        (
            "a\rb",
            Action::InsertChar("\n".into()),
            "a\n\rb",
            "a\n\rxyb",
        ),
    ] {
        let mut state = editable_state(text, &mut ctx);
        state.apply_action(&mut ctx, &Action::MoveCursorRight);

        state.apply_action(&mut ctx, &action);

        assert_eq!(state.text(), joined);
        assert_eq!(state.cursor_char_index(), Some(3));
        state.apply_action(&mut ctx, &Action::InsertChar("xy".into()));
        assert_eq!(state.text(), edited);
        assert_eq!(state.cursor_char_index(), Some(5));
    }
}
