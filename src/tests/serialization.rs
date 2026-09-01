#![cfg(feature = "serialization")]

use crate::style::{
    FontColor, FontFamily, FontSize, HorizontalTextAlignment, LetterSpacing, LineHeight, TextStyle,
    TextWrap, VerticalTextAlignment, Weight,
};
use cosmic_text::Color;

#[test]
fn test_font_color_serialization() {
    let original_color = FontColor(Color::rgba(255, 128, 64, 32));
    let serialized = serde_json::to_string(&original_color).expect("Failed to serialize FontColor");
    let deserialized: FontColor =
        serde_json::from_str(&serialized).expect("Failed to deserialize FontColor");

    assert_eq!(original_color, deserialized);
}

#[test]
fn test_text_style_serialization() {
    let original_style = TextStyle {
        font_size: FontSize(16.0),
        line_height: LineHeight(1.5),
        font_color: FontColor(Color::rgb(255, 255, 255)),
        horizontal_alignment: HorizontalTextAlignment::Center,
        vertical_alignment: VerticalTextAlignment::Center,
        wrap: Some(TextWrap::Wrap),
        font_family: FontFamily::Name("Example Sans".into()),
        weight: Weight::BOLD,
        letter_spacing: Some(LetterSpacing(0.1)),
    };
    let serialized = serde_json::to_string(&original_style).expect("Failed to serialize TextStyle");
    let deserialized: TextStyle =
        serde_json::from_str(&serialized).expect("Failed to deserialize TextStyle");

    assert_eq!(original_style, deserialized);
}
