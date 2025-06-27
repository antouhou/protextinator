use crate::style::{
    FontColor, FontFamily, FontSize, LineHeight, TextAlignment, TextStyle, TextWrap,
    VerticalTextAlignment,
};
use cosmic_text::Color;

mod byte_offset;
mod caret_positioning;
mod copy_selected_text;
mod serialization;

fn mono_style_test() -> TextStyle {
    TextStyle {
        font_size: FontSize(14.0),
        line_height: LineHeight(1.0),
        font_color: FontColor(Color::rgb(0, 0, 0)),
        overflow: None,
        horizontal_alignment: TextAlignment::Start,
        vertical_alignment: VerticalTextAlignment::Start,
        wrap: Some(TextWrap::NoWrap), // No wrapping to ensure single line
        font_family: FontFamily::Monospace,
    }
}
