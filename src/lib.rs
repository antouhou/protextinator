mod action;
mod ctx;
mod id;
pub mod math;
mod state;
mod style;
mod text;

pub use action::{Action, ActionResult};
pub use cosmic_text;
pub use ctx::{Kek, TextContext};
pub use id::Id;
pub use math::{Point, Rect};
pub use state::{Selection, SelectionLine, TextState};
pub use style::*;
pub use text::{GlyphPosition, TextManager};

#[cfg(test)]
mod tests {
    #[cfg(feature = "serialization")]
    #[test]
    fn test_font_color_serialization() {
        use cosmic_text::Color;

        // Test FontColor serialization/deserialization
        let original_color = FontColor(Color::rgba(255, 128, 64, 32));

        // Serialize to JSON
        let serialized =
            serde_json::to_string(&original_color).expect("Failed to serialize FontColor");

        // Deserialize back
        let deserialized: FontColor =
            serde_json::from_str(&serialized).expect("Failed to deserialize FontColor");

        // Should be equal
        assert_eq!(original_color.0 .0, deserialized.0 .0);
    }

    #[cfg(feature = "serialization")]
    #[test]
    fn test_text_style_serialization() {
        use cosmic_text::Color;

        // Test TextStyle serialization/deserialization
        let original_style = TextStyle {
            font_size: FontSize(16.0),
            line_height: LineHeight(1.5),
            font_color: FontColor(Color::rgb(255, 255, 255)),
            overflow: Some(TextOverflow::Clip),
            horizontal_alignment: TextAlignment::Center,
            vertical_alignment: VerticalTextAlignment::Center,
            wrap: Some(TextWrap::Wrap),
        };

        // Serialize to JSON
        let serialized =
            serde_json::to_string(&original_style).expect("Failed to serialize TextStyle");

        // Deserialize back
        let deserialized: TextStyle =
            serde_json::from_str(&serialized).expect("Failed to deserialize TextStyle");

        // Should be equal
        assert_eq!(original_style.font_size.0, deserialized.font_size.0);
        assert_eq!(original_style.line_height.0, deserialized.line_height.0);
        assert_eq!(original_style.font_color.0 .0, deserialized.font_color.0 .0);
        assert_eq!(original_style.overflow, deserialized.overflow);
        assert_eq!(
            original_style.horizontal_alignment,
            deserialized.horizontal_alignment
        );
        assert_eq!(
            original_style.vertical_alignment,
            deserialized.vertical_alignment
        );
        assert_eq!(original_style.wrap, deserialized.wrap);
    }
}
