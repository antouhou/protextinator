use crate::math::Size;
use crate::style::{TextStyle, VerticalTextAlignment};
use crate::{Action, TextContext, TextState};

const ALIGNMENT_OFFSETS: [(VerticalTextAlignment, f32); 3] = [
    (VerticalTextAlignment::Start, 0.0),
    (VerticalTextAlignment::Center, 38.0),
    (VerticalTextAlignment::End, 76.0),
];

fn configured_state(
    ctx: &mut TextContext,
    alignment: VerticalTextAlignment,
    scale: f32,
) -> TextState<()> {
    let mut state = TextState::new_with_text("a", &mut ctx.font_system, ());
    state.set_outer_size(&Size::new(200.0, 100.0));
    state.set_scale_factor(scale);
    state.set_style(&TextStyle {
        vertical_alignment: alignment,
        ..TextStyle::default()
    });
    state.is_selectable = true;
    state.are_actions_enabled = true;
    state
}

fn assert_single_line_geometry(state: &TextState<()>, expected_y: f32, scale: f32) {
    let run = state.buffer().layout_runs().next().unwrap();
    assert!(!run.glyphs.is_empty());
    assert_eq!(run.line_top / scale, expected_y);
    assert_eq!(state.caret_position_relative().unwrap().y, expected_y);
    for line in state.selection().lines() {
        assert_eq!(line.start_y_pt, Some(expected_y));
        assert_eq!(line.end_y_pt, Some(expected_y + run.line_height / scale));
    }
}

#[test]
fn initial_alignment_positions_caret_after_one_recalculation() {
    let mut ctx = TextContext::default();
    for scale in [1.0, 2.0] {
        for (alignment, expected_y) in ALIGNMENT_OFFSETS {
            let mut state = configured_state(&mut ctx, alignment, scale);

            state.recalculate(&mut ctx);

            assert_single_line_geometry(&state, expected_y, scale);
        }
    }
}

#[test]
fn alignment_changes_position_existing_selection_after_one_recalculation() {
    let mut ctx = TextContext::default();
    for scale in [1.0, 2.0] {
        for (initial_alignment, initial_y) in ALIGNMENT_OFFSETS {
            for (target_alignment, expected_y) in ALIGNMENT_OFFSETS {
                let mut state = configured_state(&mut ctx, initial_alignment, scale);
                state.recalculate(&mut ctx);
                state.apply_action(&mut ctx, &Action::SelectAll);
                assert_eq!(state.selection().lines().len(), 1);
                assert_single_line_geometry(&state, initial_y, scale);

                let mut style = state.style().clone();
                style.vertical_alignment = target_alignment;
                state.set_style(&style);
                state.recalculate(&mut ctx);

                assert_eq!(state.selected_text(), Some("a"));
                assert_eq!(state.selection().lines().len(), 1);
                assert_single_line_geometry(&state, expected_y, scale);
            }
        }
    }
}

#[test]
fn resizing_positions_caret_and_selection_after_one_recalculation() {
    let mut ctx = TextContext::default();
    for scale in [1.0, 2.0] {
        for (alignment, resized_offsets) in [
            (VerticalTextAlignment::Start, [(200.0, 0.0), (50.0, 0.0)]),
            (VerticalTextAlignment::Center, [(200.0, 88.0), (50.0, 13.0)]),
            (VerticalTextAlignment::End, [(200.0, 176.0), (50.0, 26.0)]),
        ] {
            let mut state = configured_state(&mut ctx, alignment, scale);
            state.recalculate(&mut ctx);
            state.apply_action(&mut ctx, &Action::SelectAll);
            assert_eq!(state.selection().lines().len(), 1);

            for (height, expected_y) in resized_offsets {
                state.set_outer_size(&Size::new(200.0, height));
                state.recalculate(&mut ctx);

                assert_eq!(state.selected_text(), Some("a"));
                assert_eq!(state.selection().lines().len(), 1);
                assert_single_line_geometry(&state, expected_y, scale);
            }
        }
    }
}
