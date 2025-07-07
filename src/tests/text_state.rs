use crate::math::Size;
use crate::style::{
    FontColor, FontFamily, FontSize, HorizontalTextAlignment, LineHeight, TextStyle, TextWrap,
    VerticalTextAlignment,
};
use crate::tests::mono_style_test;
use crate::{Action, Point, TextContext, TextState};
use cosmic_text::Color;

// Helper functions for creating styles with different alignments
fn mono_style_with_alignment(
    h_align: HorizontalTextAlignment,
    v_align: VerticalTextAlignment,
) -> TextStyle {
    TextStyle {
        font_size: FontSize(14.0),
        line_height: LineHeight(1.0),
        font_color: FontColor(Color::rgb(0, 0, 0)),
        horizontal_alignment: h_align,
        vertical_alignment: v_align,
        wrap: Some(TextWrap::NoWrap), // No wrapping to ensure a single line
        font_family: FontFamily::Monospace,
    }
}

fn mono_style_with_wrap(
    h_align: HorizontalTextAlignment,
    v_align: VerticalTextAlignment,
    wrap: Option<TextWrap>,
) -> TextStyle {
    TextStyle {
        font_size: FontSize(14.0),
        line_height: LineHeight(1.0),
        font_color: FontColor(Color::rgb(0, 0, 0)),
        horizontal_alignment: h_align,
        vertical_alignment: v_align,
        wrap,
        font_family: FontFamily::Monospace,
    }
}

#[test]
pub fn test_set_text() {
    let mut ctx = TextContext::default();
    let initial_text = "Hello World".to_string();

    let mut text_state = TextState::new_with_text(initial_text, &mut ctx.font_system, ());
    text_state.set_style(&mono_style_test());
    text_state.set_outer_size(&Size::new(200.0, 25.0));
    text_state.recalculate(&mut ctx);

    // Test initial text
    assert_eq!(text_state.text(), "Hello World");
    assert_eq!(text_state.text_char_len(), 11);

    // Test setting new text
    text_state.set_text("New text");
    text_state.recalculate(&mut ctx);
    assert_eq!(text_state.text(), "New text");
    assert_eq!(text_state.text_char_len(), 8);

    // Test setting empty text
    text_state.set_text("");
    text_state.recalculate(&mut ctx);
    assert_eq!(text_state.text(), "");
    assert_eq!(text_state.text_char_len(), 0);
}

#[test]
pub fn test_set_style() {
    let mut ctx = TextContext::default();
    let initial_text = "Hello World".to_string();

    let mut text_state = TextState::new_with_text(initial_text, &mut ctx.font_system, ());

    // Initial style (not used in this test but kept for documentation)
    let _initial_style = text_state.style().clone();

    // Create a new style
    let new_style = TextStyle::new(20.0, Color::rgb(255, 0, 0))
        .with_horizontal_alignment(HorizontalTextAlignment::Center)
        .with_vertical_alignment(VerticalTextAlignment::Center);

    // Set the new style
    text_state.set_style(&new_style);

    // Verify style was updated
    let updated_style = text_state.style();
    assert_eq!(updated_style.font_size.value(), 20.0);
    assert_eq!(updated_style.font_color.0, Color::rgb(255, 0, 0));
    assert!(matches!(
        updated_style.horizontal_alignment,
        HorizontalTextAlignment::Center
    ));
    assert!(matches!(
        updated_style.vertical_alignment,
        VerticalTextAlignment::Center
    ));
}

#[test]
pub fn test_set_outer_size() {
    let mut ctx = TextContext::default();
    let initial_text = "Hello World".to_string();

    let mut text_state = TextState::new_with_text(initial_text, &mut ctx.font_system, ());

    // Initial size should be zero
    assert_eq!(text_state.outer_size(), Size::ZERO);

    // Set a new size
    let new_size = Size::new(300.0, 150.0);
    text_state.set_outer_size(&new_size);

    // Verify size was updated
    assert_eq!(text_state.outer_size(), new_size);
}

#[test]
pub fn test_inner_size() {
    let mut ctx = TextContext::default();
    let initial_text = "Hello World".to_string();

    let mut text_state = TextState::new_with_text(initial_text, &mut ctx.font_system, ());
    text_state.set_style(&mono_style_test());
    text_state.set_outer_size(&Size::new(200.0, 25.0));

    // Before recalculation, inner size should be zero
    assert_eq!(text_state.inner_size(), Size::ZERO);

    // After recalculation, inner size should reflect text content
    text_state.recalculate(&mut ctx);
    let inner_size = text_state.inner_size();

    // Inner size should be non-zero after recalculation
    assert!(inner_size.x > 0.0);
    assert!(inner_size.y > 0.0);

    // Adding more text should increase inner size
    text_state.set_text("Hello World with more text to increase the size");
    text_state.recalculate(&mut ctx);
    let new_inner_size = text_state.inner_size();

    // New inner size should be larger than before
    assert!(new_inner_size.x > inner_size.x);
}

#[test]
pub fn test_buffer_metadata() {
    let mut ctx = TextContext::default();
    let initial_text = "Hello World".to_string();

    let mut text_state = TextState::new_with_text(initial_text, &mut ctx.font_system, ());

    // Default metadata should be 0
    assert_eq!(text_state.buffer_metadata(), 0);

    // Set new metadata
    text_state.set_buffer_metadata(42);

    // Verify metadata was updated
    assert_eq!(text_state.buffer_metadata(), 42);
}

#[test]
pub fn test_caret_width() {
    let mut ctx = TextContext::default();
    let initial_text = "Hello World".to_string();

    let mut text_state = TextState::new_with_text(initial_text, &mut ctx.font_system, ());

    // Default caret width should be 3.0
    assert_eq!(text_state.caret_width(), 3.0);

    // Set new caret width
    text_state.set_caret_width(5.0);

    // Verify caret width was updated
    assert_eq!(text_state.caret_width(), 5.0);
}

#[test]
pub fn test_selection_methods() {
    let mut ctx = TextContext::default();
    let initial_text = "Hello World".to_string();

    let mut text_state = TextState::new_with_text(initial_text, &mut ctx.font_system, ());
    text_state.set_style(&mono_style_test());
    text_state.set_outer_size(&Size::new(200.0, 25.0));
    text_state.is_editable = true;
    text_state.is_selectable = true;
    text_state.are_actions_enabled = true;
    text_state.recalculate(&mut ctx);

    // Initially no text should be selected
    assert!(!text_state.is_text_selected());
    assert!(text_state.selected_text().is_none());

    // Select all text
    text_state.apply_action(&mut ctx, &Action::SelectAll);

    // Verify text is selected
    assert!(text_state.is_text_selected());
    assert_eq!(text_state.selected_text(), Some("Hello World"));

    // Reset selection
    text_state.reset_selection();

    // Verify selection was reset
    assert!(!text_state.is_text_selected());
    assert!(text_state.selected_text().is_none());
}

#[test]
pub fn test_cursor_char_index() {
    let mut ctx = TextContext::default();
    let initial_text = "Hello World".to_string();

    let mut text_state = TextState::new_with_text(initial_text, &mut ctx.font_system, ());
    text_state.set_style(&mono_style_test());
    text_state.set_outer_size(&Size::new(200.0, 25.0));
    text_state.is_editable = true;
    text_state.is_editing = true;
    text_state.is_selectable = true;
    text_state.are_actions_enabled = true;
    text_state.recalculate(&mut ctx);

    // Initial cursor position should be at the beginning
    assert_eq!(text_state.cursor_char_index(), Some(0));

    // Move cursor right
    text_state.apply_action(&mut ctx, &Action::MoveCursorRight);
    assert_eq!(text_state.cursor_char_index(), Some(1));

    // Move cursor right again
    text_state.apply_action(&mut ctx, &Action::MoveCursorRight);
    assert_eq!(text_state.cursor_char_index(), Some(2));

    // Move cursor left
    text_state.apply_action(&mut ctx, &Action::MoveCursorLeft);
    assert_eq!(text_state.cursor_char_index(), Some(1));
}

#[test]
pub fn test_absolute_scroll() {
    let mut ctx = TextContext::default();
    // Create a multi-line text to test scrolling
    let initial_text = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5".to_string();

    let mut text_state = TextState::new_with_text(initial_text, &mut ctx.font_system, ());
    // Use a style with vertical alignment set to None to allow vertical scrolling
    text_state.set_style(&mono_style_with_alignment(
        HorizontalTextAlignment::Left,
        VerticalTextAlignment::None,
    ));
    text_state.set_outer_size(&Size::new(200.0, 50.0)); // Small height to ensure scrolling is needed
    text_state.recalculate(&mut ctx);

    // Initial scroll should be at the top
    let initial_scroll = text_state.absolute_scroll();
    assert_eq!(initial_scroll.x, 0.0);
    assert_eq!(initial_scroll.y, 0.0);

    // Set scroll position
    let new_scroll = Point::new(0.0, 20.0);
    text_state.set_absolute_scroll(new_scroll);

    // Verify scroll position was updated
    let updated_scroll = text_state.absolute_scroll();
    assert!(updated_scroll.y > 0.0); // Should have scrolled down
}

#[test]
pub fn test_handle_press() {
    let mut ctx = TextContext::default();
    let initial_text = "Hello World".to_string();

    let mut text_state = TextState::new_with_text(initial_text, &mut ctx.font_system, ());
    text_state.set_style(&mono_style_test());
    text_state.set_outer_size(&Size::new(200.0, 25.0));
    text_state.is_editable = true;
    text_state.is_selectable = true;
    text_state.recalculate(&mut ctx);

    // Initial cursor position
    assert_eq!(text_state.cursor_char_index(), Some(0));

    // Handle press in the middle of the text
    text_state.handle_press(&mut ctx, Point::new(30.0, 10.0));

    // Cursor should have moved
    assert!(text_state.cursor_char_index().unwrap() > 0);
}

#[test]
pub fn test_horizontal_alignment() {
    let mut ctx = TextContext::default();
    let text = "Hello World".to_string();
    let container_width = 200.0;
    let container_height = 25.0;

    // Test left alignment (Start)
    let mut text_state = TextState::new_with_text(text.clone(), &mut ctx.font_system, ());
    text_state.set_style(&mono_style_with_alignment(
        HorizontalTextAlignment::Left,
        VerticalTextAlignment::Start,
    ));
    text_state.set_outer_size(&Size::new(container_width, container_height));
    text_state.recalculate(&mut ctx);

    // Get the first glyph position (should be at the left edge)
    let first_glyph_x = text_state.first_glyph().unwrap().x;
    assert!(
        first_glyph_x < 5.0,
        "Left-aligned text should start near the left edge"
    );

    // Test center alignment
    let mut text_state = TextState::new_with_text(text.clone(), &mut ctx.font_system, ());
    text_state.set_style(&mono_style_with_alignment(
        HorizontalTextAlignment::Center,
        VerticalTextAlignment::Start,
    ));
    text_state.set_outer_size(&Size::new(container_width, container_height));
    text_state.recalculate(&mut ctx);

    // Get text width and then the first glyph position
    let text_width = text_state.inner_size().x;
    let first_glyph_x = text_state.first_glyph().unwrap().x;
    let expected_x = (container_width - text_width) / 2.0;
    assert!(
        (first_glyph_x - expected_x).abs() < 5.0,
        "Center-aligned text should start near the center. Expected: {expected_x}, Actual: {first_glyph_x}"
    );

    // Test right alignment
    let mut text_state = TextState::new_with_text(text.clone(), &mut ctx.font_system, ());
    text_state.set_style(&mono_style_with_alignment(
        HorizontalTextAlignment::Right,
        VerticalTextAlignment::Start,
    ));
    text_state.set_outer_size(&Size::new(container_width, container_height));
    text_state.recalculate(&mut ctx);

    // Get text width and then the first glyph position
    let text_width = text_state.inner_size().x;
    let first_glyph_x = text_state.first_glyph().unwrap().x;
    let expected_x = container_width - text_width;
    assert!((first_glyph_x - expected_x).abs() < 5.0,
            "Right-aligned text should start near the right edge minus text width. Expected: {expected_x}, Actual: {first_glyph_x}");
}

#[test]
pub fn test_vertical_alignment() {
    let mut ctx = TextContext::default();
    let text = "Hello World".to_string();
    let container_width = 200.0;
    let container_height = 100.0; // Taller container to see vertical alignment effects

    // Test top alignment (Start)
    let mut text_state = TextState::new_with_text(text.clone(), &mut ctx.font_system, ());
    text_state.set_style(&mono_style_with_alignment(
        HorizontalTextAlignment::Left,
        VerticalTextAlignment::Start,
    ));
    text_state.set_outer_size(&Size::new(container_width, container_height));
    text_state.recalculate(&mut ctx);

    // Verify the style is set correctly
    assert!(
        matches!(
            text_state.style().vertical_alignment,
            VerticalTextAlignment::Start
        ),
        "Vertical alignment should be Start"
    );

    // Test center vertical alignment
    let mut text_state = TextState::new_with_text(text.clone(), &mut ctx.font_system, ());
    text_state.set_style(&mono_style_with_alignment(
        HorizontalTextAlignment::Left,
        VerticalTextAlignment::Center,
    ));
    text_state.set_outer_size(&Size::new(container_width, container_height));
    text_state.recalculate(&mut ctx);

    // Verify the style is set correctly
    assert!(
        matches!(
            text_state.style().vertical_alignment,
            VerticalTextAlignment::Center
        ),
        "Vertical alignment should be Center"
    );

    // Test bottom alignment (End)
    let mut text_state = TextState::new_with_text(text.clone(), &mut ctx.font_system, ());
    text_state.set_style(&mono_style_with_alignment(
        HorizontalTextAlignment::Left,
        VerticalTextAlignment::End,
    ));
    text_state.set_outer_size(&Size::new(container_width, container_height));
    text_state.recalculate(&mut ctx);

    // Verify the style is set correctly
    assert!(
        matches!(
            text_state.style().vertical_alignment,
            VerticalTextAlignment::End
        ),
        "Vertical alignment should be End"
    );
}

#[test]
pub fn test_combined_alignment() {
    let mut ctx = TextContext::default();
    let text = "Hello World".to_string();
    let container_width = 200.0;
    let container_height = 100.0;

    // Test center-center alignment
    let mut text_state = TextState::new_with_text(text.clone(), &mut ctx.font_system, ());
    text_state.set_style(&mono_style_with_alignment(
        HorizontalTextAlignment::Center,
        VerticalTextAlignment::Center,
    ));
    text_state.set_outer_size(&Size::new(container_width, container_height));
    text_state.recalculate(&mut ctx);

    // Verify the style is set correctly
    assert!(
        matches!(
            text_state.style().horizontal_alignment,
            HorizontalTextAlignment::Center
        ),
        "Horizontal alignment should be Center"
    );
    assert!(
        matches!(
            text_state.style().vertical_alignment,
            VerticalTextAlignment::Center
        ),
        "Vertical alignment should be Center"
    );

    // Get text width and first glyph position to check horizontal centering
    let text_width = text_state.inner_size().x;
    let first_glyph_x = text_state.first_glyph().unwrap().x;
    let expected_x = (container_width - text_width) / 2.0;
    assert!(
        (first_glyph_x - expected_x).abs() < 5.0,
        "Horizontally centered text should start at the right position. Expected: {expected_x}, Actual: {first_glyph_x}"
    );

    // Test right-bottom alignment
    let mut text_state = TextState::new_with_text(text.clone(), &mut ctx.font_system, ());
    text_state.set_style(&mono_style_with_alignment(
        HorizontalTextAlignment::Right,
        VerticalTextAlignment::End,
    ));
    text_state.set_outer_size(&Size::new(container_width, container_height));
    text_state.recalculate(&mut ctx);

    // Verify the style is set correctly
    assert!(
        matches!(
            text_state.style().horizontal_alignment,
            HorizontalTextAlignment::Right
        ),
        "Horizontal alignment should be Right"
    );
    assert!(
        matches!(
            text_state.style().vertical_alignment,
            VerticalTextAlignment::End
        ),
        "Vertical alignment should be End"
    );

    // Get text width and first glyph position to check right alignment
    let text_width = text_state.inner_size().x;
    let first_glyph_x = text_state.first_glyph().unwrap().x;
    let expected_x = container_width - text_width;
    assert!(
        (first_glyph_x - expected_x).abs() < 5.0,
        "Right-aligned text should start at the right position. Expected: {expected_x}, Actual: {first_glyph_x}"
    );
}

#[test]
pub fn test_horizontal_overflow() {
    let mut ctx = TextContext::default();
    let long_text =
        "This is a very long text that should overflow the container horizontally".to_string();
    let container_width = 100.0; // Small width to ensure overflow
    let container_height = 50.0;

    // Test with no wrapping (should overflow)
    let mut text_state = TextState::new_with_text(long_text.clone(), &mut ctx.font_system, ());
    text_state.set_style(&mono_style_with_wrap(
        HorizontalTextAlignment::Left,
        VerticalTextAlignment::Start,
        Some(TextWrap::NoWrap), // No wrapping
    ));
    text_state.set_outer_size(&Size::new(container_width, container_height));
    text_state.recalculate(&mut ctx);

    // Check that text width exceeds container width
    let text_width = text_state.inner_size().x;
    assert!(
        text_width > container_width,
        "Text should overflow horizontally. Text width: {text_width}, Container width: {container_width}"
    );

    // Test with wrapping (should not overflow horizontally)
    let mut text_state = TextState::new_with_text(long_text.clone(), &mut ctx.font_system, ());
    text_state.set_style(&mono_style_with_wrap(
        HorizontalTextAlignment::Left,
        VerticalTextAlignment::Start,
        Some(TextWrap::Wrap), // With wrapping
    ));
    text_state.set_outer_size(&Size::new(container_width, container_height));
    text_state.recalculate(&mut ctx);

    // Check that text width is close to container width (allow small margin of error)
    let text_width = text_state.inner_size().x;
    assert!(text_width <= container_width + 2.0,
            "Text should not overflow horizontally with wrapping by much. Text width: {text_width}, Container width: {container_width}");
}

#[test]
pub fn test_vertical_overflow() {
    let mut ctx = TextContext::default();
    let multi_line_text =
        "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10"
            .to_string();
    let container_width = 200.0;
    let container_height = 50.0; // Small height to ensure overflow

    // Test vertical overflow
    let mut text_state =
        TextState::new_with_text(multi_line_text.clone(), &mut ctx.font_system, ());
    text_state.set_style(&mono_style_test());
    text_state.set_outer_size(&Size::new(container_width, container_height));
    text_state.recalculate(&mut ctx);

    // Check that text height exceeds container height
    let text_height = text_state.inner_size().y;
    assert!(
        text_height > container_height,
        "Text should overflow vertically. Text height: {text_height}, Container height: {container_height}"
    );
}

#[test]
pub fn test_both_overflow() {
    let mut ctx = TextContext::default();
    // Create text that will overflow both horizontally and vertically
    let long_multi_line_text =
        "This is a very long line that should overflow horizontally\n".repeat(10);
    let container_width = 100.0; // Small width to ensure horizontal overflow
    let container_height = 50.0; // Small height to ensure vertical overflow

    // Test both horizontal and vertical overflow
    let mut text_state =
        TextState::new_with_text(long_multi_line_text.clone(), &mut ctx.font_system, ());
    text_state.set_style(&mono_style_with_wrap(
        HorizontalTextAlignment::Left,
        VerticalTextAlignment::Start,
        Some(TextWrap::NoWrap), // No wrapping to ensure horizontal overflow
    ));
    text_state.set_outer_size(&Size::new(container_width, container_height));
    text_state.recalculate(&mut ctx);

    // Check that text dimensions exceed container dimensions
    let text_size = text_state.inner_size();
    assert!(
        text_size.x > container_width,
        "Text should overflow horizontally. Text width: {}, Container width: {}",
        text_size.x,
        container_width
    );
    assert!(
        text_size.y > container_height,
        "Text should overflow vertically. Text height: {}, Container height: {}",
        text_size.y,
        container_height
    );
}

#[test]
pub fn test_vertical_scroll_with_alignment() {
    let mut ctx = TextContext::default();
    // Create a multi-line text to test scrolling
    let initial_text = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5".to_string();
    let container_width = 200.0;
    let container_height = 50.0; // Small height to ensure scrolling is needed

    // Test with vertical alignment None (should allow scrolling)
    let mut text_state = TextState::new_with_text(initial_text.clone(), &mut ctx.font_system, ());
    text_state.set_style(&mono_style_with_alignment(
        HorizontalTextAlignment::Left,
        VerticalTextAlignment::None,
    ));
    text_state.set_outer_size(&Size::new(container_width, container_height));
    text_state.recalculate(&mut ctx);

    // Initial scroll should be at the top
    let initial_scroll = text_state.absolute_scroll();
    assert_eq!(initial_scroll.y, 0.0);

    // Set vertical scroll position
    let new_scroll = Point::new(0.0, 20.0);
    text_state.set_absolute_scroll(new_scroll);

    // Verify scroll position was updated (should work with VerticalTextAlignment::None)
    let updated_scroll = text_state.absolute_scroll();
    assert!(
        updated_scroll.y > 0.0,
        "Vertical scrolling should work with VerticalTextAlignment::None"
    );

    // Test with vertical alignment Start (should not allow scrolling)
    let mut text_state = TextState::new_with_text(initial_text.clone(), &mut ctx.font_system, ());
    text_state.set_style(&mono_style_with_alignment(
        HorizontalTextAlignment::Left,
        VerticalTextAlignment::Start,
    ));
    text_state.set_outer_size(&Size::new(container_width, container_height));
    text_state.recalculate(&mut ctx);

    // Get the initial scroll position
    // Note: We don't assert that it's 0.0 because absolute_scroll() might return
    // non-zero values even after setting scroll to 0.0 due to internal calculations
    text_state.set_absolute_scroll(Point::new(0.0, 0.0));
    let initial_scroll = text_state.absolute_scroll();

    // Try to set vertical scroll position
    let new_scroll = Point::new(0.0, 20.0);
    text_state.set_absolute_scroll(new_scroll);

    // Verify scroll position was not updated (should not work with VerticalTextAlignment::Start)
    let updated_scroll = text_state.absolute_scroll();
    assert_eq!(
        updated_scroll.y, initial_scroll.y,
        "Vertical scrolling should not work with VerticalTextAlignment::Start"
    );

    // Test with vertical alignment Center (should not allow scrolling)
    let mut text_state = TextState::new_with_text(initial_text.clone(), &mut ctx.font_system, ());
    text_state.set_style(&mono_style_with_alignment(
        HorizontalTextAlignment::Left,
        VerticalTextAlignment::Center,
    ));
    text_state.set_outer_size(&Size::new(container_width, container_height));
    text_state.recalculate(&mut ctx);

    // Get the initial scroll position
    // Note: We don't assert that it's 0.0 because absolute_scroll() might return
    // non-zero values even after setting scroll to 0.0 due to internal calculations
    text_state.set_absolute_scroll(Point::new(0.0, 0.0));
    let initial_scroll = text_state.absolute_scroll();

    // Try to set vertical scroll position
    text_state.set_absolute_scroll(new_scroll);

    // Verify scroll position was not updated (should not work with VerticalTextAlignment::Center)
    let updated_scroll = text_state.absolute_scroll();
    assert_eq!(
        updated_scroll.y, initial_scroll.y,
        "Vertical scrolling should not work with VerticalTextAlignment::Center"
    );

    // Test with vertical alignment End (should not allow scrolling)
    let mut text_state = TextState::new_with_text(initial_text.clone(), &mut ctx.font_system, ());
    text_state.set_style(&mono_style_with_alignment(
        HorizontalTextAlignment::Left,
        VerticalTextAlignment::End,
    ));
    text_state.set_outer_size(&Size::new(container_width, container_height));
    text_state.recalculate(&mut ctx);

    // Get the initial scroll position
    // Note: We don't assert that it's 0.0 because absolute_scroll() might return
    // non-zero values even after setting scroll to 0.0 due to internal calculations
    text_state.set_absolute_scroll(Point::new(0.0, 0.0));
    let initial_scroll = text_state.absolute_scroll();

    // Try to set vertical scroll position
    text_state.set_absolute_scroll(new_scroll);

    // Verify scroll position was not updated (should not work with VerticalTextAlignment::End)
    let updated_scroll = text_state.absolute_scroll();
    assert_eq!(
        updated_scroll.y, initial_scroll.y,
        "Vertical scrolling should not work with VerticalTextAlignment::End"
    );
}

#[test]
pub fn test_horizontal_scroll_with_alignment() {
    let mut ctx = TextContext::default();
    // Create a long text to test horizontal scrolling
    let initial_text =
        "This is a very long text that should overflow the container horizontally".to_string();
    let container_width = 100.0; // Small width to ensure scrolling is needed
    let container_height = 50.0;

    // Test with horizontal alignment None (should allow scrolling)
    let mut text_state = TextState::new_with_text(initial_text.clone(), &mut ctx.font_system, ());
    text_state.set_style(&mono_style_with_alignment(
        HorizontalTextAlignment::None,
        VerticalTextAlignment::Start,
    ));
    text_state.set_outer_size(&Size::new(container_width, container_height));
    text_state.recalculate(&mut ctx);

    // Initial scroll should be at the left
    let initial_scroll = text_state.absolute_scroll();
    assert_eq!(initial_scroll.x, 0.0);

    // Set horizontal scroll position
    let new_scroll = Point::new(20.0, 0.0);
    text_state.set_absolute_scroll(new_scroll);

    // Verify scroll position was updated (should work with HorizontalTextAlignment::None)
    let updated_scroll = text_state.absolute_scroll();
    assert!(
        updated_scroll.x > 0.0,
        "Horizontal scrolling should work with HorizontalTextAlignment::None"
    );

    // Test with horizontal alignment Left (should not allow scrolling)
    let mut text_state = TextState::new_with_text(initial_text.clone(), &mut ctx.font_system, ());
    text_state.set_style(&mono_style_with_alignment(
        HorizontalTextAlignment::Left,
        VerticalTextAlignment::Start,
    ));
    text_state.set_outer_size(&Size::new(container_width, container_height));
    text_state.recalculate(&mut ctx);

    // Initial scroll should be at the left
    let initial_scroll = text_state.absolute_scroll();
    assert_eq!(initial_scroll.x, 0.0);

    // Try to set horizontal scroll position
    let new_scroll = Point::new(20.0, 0.0);
    text_state.set_absolute_scroll(new_scroll);

    // Verify scroll position was not updated (should not work with HorizontalTextAlignment::Left)
    let updated_scroll = text_state.absolute_scroll();
    assert_eq!(
        updated_scroll.x, 0.0,
        "Horizontal scrolling should not work with HorizontalTextAlignment::Left"
    );

    // Test with horizontal alignment Center (should not allow scrolling)
    let mut text_state = TextState::new_with_text(initial_text.clone(), &mut ctx.font_system, ());
    text_state.set_style(&mono_style_with_alignment(
        HorizontalTextAlignment::Center,
        VerticalTextAlignment::Start,
    ));
    text_state.set_outer_size(&Size::new(container_width, container_height));
    text_state.recalculate(&mut ctx);

    // Try to set horizontal scroll position
    text_state.set_absolute_scroll(new_scroll);

    // Verify scroll position was not updated (should not work with HorizontalTextAlignment::Center)
    let updated_scroll = text_state.absolute_scroll();
    assert_eq!(
        updated_scroll.x, 0.0,
        "Horizontal scrolling should not work with HorizontalTextAlignment::Center"
    );

    // Test with horizontal alignment Right (should not allow scrolling)
    let mut text_state = TextState::new_with_text(initial_text.clone(), &mut ctx.font_system, ());
    text_state.set_style(&mono_style_with_alignment(
        HorizontalTextAlignment::Right,
        VerticalTextAlignment::Start,
    ));
    text_state.set_outer_size(&Size::new(container_width, container_height));
    text_state.recalculate(&mut ctx);

    // Try to set horizontal scroll position
    text_state.set_absolute_scroll(new_scroll);

    // Verify scroll position was not updated (should not work with HorizontalTextAlignment::Right)
    let updated_scroll = text_state.absolute_scroll();
    assert_eq!(
        updated_scroll.x, 0.0,
        "Horizontal scrolling should not work with HorizontalTextAlignment::Right"
    );
}

#[test]
pub fn test_combined_scroll_with_alignment() {
    let mut ctx = TextContext::default();
    // Create a long multi-line text to test both scrolling directions
    let initial_text = "This is a very long text that should overflow horizontally\nLine 2\nLine 3\nLine 4\nLine 5".to_string();
    let container_width = 100.0; // Small width to ensure horizontal scrolling is needed
    let container_height = 50.0; // Small height to ensure vertical scrolling is needed

    // Test with both alignments set to None (should allow both scrolling directions)
    let mut text_state = TextState::new_with_text(initial_text.clone(), &mut ctx.font_system, ());
    text_state.set_style(&mono_style_with_alignment(
        HorizontalTextAlignment::None,
        VerticalTextAlignment::None,
    ));
    text_state.set_outer_size(&Size::new(container_width, container_height));
    text_state.recalculate(&mut ctx);

    // Initial scroll should be at the top-left
    let initial_scroll = text_state.absolute_scroll();
    assert_eq!(initial_scroll.x, 0.0);
    assert_eq!(initial_scroll.y, 0.0);

    // Set both scroll positions
    let new_scroll = Point::new(20.0, 30.0);
    text_state.set_absolute_scroll(new_scroll);

    // Verify both scroll positions were updated
    let updated_scroll = text_state.absolute_scroll();
    assert!(
        updated_scroll.x > 0.0,
        "Horizontal scrolling should work with HorizontalTextAlignment::None"
    );
    assert!(
        updated_scroll.y > 0.0,
        "Vertical scrolling should work with VerticalTextAlignment::None"
    );

    // Test with horizontal alignment None but vertical alignment Start (should only allow horizontal scrolling)
    let mut text_state = TextState::new_with_text(initial_text.clone(), &mut ctx.font_system, ());
    text_state.set_style(&mono_style_with_alignment(
        HorizontalTextAlignment::None,
        VerticalTextAlignment::Start,
    ));
    text_state.set_outer_size(&Size::new(container_width, container_height));
    text_state.recalculate(&mut ctx);

    // Set both scroll positions
    text_state.set_absolute_scroll(new_scroll);

    // Verify only horizontal scroll was updated
    let updated_scroll = text_state.absolute_scroll();
    assert!(
        updated_scroll.x > 0.0,
        "Horizontal scrolling should work with HorizontalTextAlignment::None"
    );
    assert_eq!(
        updated_scroll.y, 0.0,
        "Vertical scrolling should not work with VerticalTextAlignment::Start"
    );

    // Test with vertical alignment None but horizontal alignment Left (should only allow vertical scrolling)
    let mut text_state = TextState::new_with_text(initial_text.clone(), &mut ctx.font_system, ());
    text_state.set_style(&mono_style_with_alignment(
        HorizontalTextAlignment::Left,
        VerticalTextAlignment::None,
    ));
    text_state.set_outer_size(&Size::new(container_width, container_height));
    text_state.recalculate(&mut ctx);

    // Set both scroll positions
    text_state.set_absolute_scroll(new_scroll);

    // Verify only vertical scroll was updated
    let updated_scroll = text_state.absolute_scroll();
    assert_eq!(
        updated_scroll.x, 0.0,
        "Horizontal scrolling should not work with HorizontalTextAlignment::Left"
    );
    assert!(
        updated_scroll.y > 0.0,
        "Vertical scrolling should work with VerticalTextAlignment::None"
    );
}
