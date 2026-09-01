use crate::style::{
    FontColor, FontFamily, FontSize, HorizontalTextAlignment, LineHeight, TextStyle, TextWrap,
    VerticalTextAlignment, Weight,
};
use crate::tests::mono_style_test;
use crate::{Point, TextContext, TextState};
use cosmic_text::{fontdb, Color};

#[test]
fn default_text_context_preserves_a_valid_system_sans_serif_family() {
    let mut system_font_database = fontdb::Database::new();
    system_font_database.load_system_fonts();
    let system_sans_serif_family = system_font_database
        .family_name(&fontdb::Family::SansSerif)
        .to_owned();
    let ctx = TextContext::default();

    assert_eq!(
        ctx.font_system.db().family_name(&fontdb::Family::SansSerif),
        system_sans_serif_family
    );

    let resolved_face = ctx.font_system.db().query(&fontdb::Query {
        families: &[fontdb::Family::SansSerif],
        ..Default::default()
    });

    assert!(
        resolved_face.is_some(),
        "the default sans-serif family must resolve to an installed font"
    );
}

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
