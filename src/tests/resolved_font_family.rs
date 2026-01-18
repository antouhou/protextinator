use crate::style::{
    FontColor, FontFamily, FontSize, HorizontalTextAlignment, LineHeight, TextStyle, TextWrap,
    VerticalTextAlignment, Weight,
};
use crate::tests::mono_style_test;
use crate::{Point, TextContext, TextState};
use cosmic_text::Color;

#[test]
fn test_resolved_font_family_changes_to_monospace() {
    let mut ctx = TextContext::default();
    let initial_text = "Hello World".to_string();

    let mut text_state = TextState::new_with_text(initial_text, &mut ctx.font_system, ());

    // Initially should have the default (SansSerif)
    assert_eq!(text_state.resolved_font_family(), &FontFamily::SansSerif);

    // Set monospace style
    text_state.set_style(&mono_style_test());
    text_state.set_outer_size(&Point::from((200.0, 25.0)));

    // After recalculate, should be Monospace
    text_state.recalculate(&mut ctx);

    println!(
        "Resolved font family: {:?}",
        text_state.resolved_font_family()
    );
    assert_eq!(text_state.resolved_font_family(), &FontFamily::Monospace);
}

#[test]
fn test_resolved_font_family_changes_to_serif() {
    let mut ctx = TextContext::default();
    let initial_text = "Hello World".to_string();

    let mut text_state = TextState::new_with_text(initial_text, &mut ctx.font_system, ());

    // Initially should have the default (SansSerif)
    assert_eq!(text_state.resolved_font_family(), &FontFamily::SansSerif);

    // Set serif style
    let serif_style = TextStyle {
        font_size: FontSize(14.0),
        line_height: LineHeight(1.0),
        font_color: FontColor(Color::rgb(0, 0, 0)),
        horizontal_alignment: HorizontalTextAlignment::Start,
        vertical_alignment: VerticalTextAlignment::Start,
        wrap: Some(TextWrap::NoWrap),
        font_family: FontFamily::Serif,
        weight: Weight::NORMAL,
        letter_spacing: None,
    };

    text_state.set_style(&serif_style);
    text_state.set_outer_size(&Point::from((200.0, 25.0)));

    // After recalculate, should be Serif
    text_state.recalculate(&mut ctx);

    println!(
        "Resolved font family: {:?}",
        text_state.resolved_font_family()
    );
    assert_eq!(text_state.resolved_font_family(), &FontFamily::Serif);
}
