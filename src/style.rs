use cosmic_text::{Align, Color};
#[cfg(feature = "serialization")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::hash::Hash;

#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextOverflow {
    Clip,
}

#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextWrap {
    #[default]
    NoWrap,
    Wrap,
    BreakWord,
}

#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineHeight(pub f32);

impl Default for LineHeight {
    fn default() -> Self {
        Self(1.5)
    }
}

impl Hash for LineHeight {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl From<f32> for LineHeight {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl Eq for LineHeight {}

#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontSize(pub f32);

impl Default for FontSize {
    fn default() -> Self {
        Self(1.5)
    }
}

impl Hash for FontSize {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl From<f32> for FontSize {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl Eq for FontSize {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FontColor(pub Color);

#[cfg(feature = "serialization")]
impl Serialize for FontColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.0.0)
    }
}

#[cfg(feature = "serialization")]
impl<'de> Deserialize<'de> for FontColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let color_value = u32::deserialize(deserializer)?;
        Ok(FontColor(Color(color_value)))
    }
}

impl FontColor {
    pub fn new(color: Color) -> Self {
        Self(color)
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self(Color::rgb(r, g, b))
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(Color::rgba(r, g, b, a))
    }
}

impl From<Color> for FontColor {
    fn from(color: Color) -> Self {
        Self(color)
    }
}

impl From<FontColor> for Color {
    fn from(font_color: FontColor) -> Self {
        font_color.0
    }
}

#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextStyle {
    /// The font size in points.
    pub font_size: FontSize,
    /// The line height is a multiplier of the font size.
    pub line_height: LineHeight,
    /// The color of the text.
    pub font_color: FontColor,
    pub overflow: Option<TextOverflow>,
    pub horizontal_alignment: TextAlignment,
    pub vertical_alignment: VerticalTextAlignment,
    pub wrap: Option<TextWrap>,
}

#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub enum TextAlignment {
    #[default]
    Start,
    End,
    Center,
    Left,
    Right,
    Justify,
}

impl Into<Option<Align>> for TextAlignment {
    fn into(self) -> Option<Align> {
        match self {
            Self::Start => None,
            Self::End => Some(Align::End),
            Self::Center => Some(Align::Center),
            Self::Left => Some(Align::Left),
            Self::Right => Some(Align::Right),
            Self::Justify => Some(Align::Justified),
        }
    }
}

#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub enum VerticalTextAlignment {
    #[default]
    Start,
    End,
    Center,
}

impl Hash for TextStyle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.font_size.hash(state);
        self.line_height.hash(state);
        self.font_color.hash(state);
        self.overflow.hash(state);
        self.horizontal_alignment.hash(state);
        self.vertical_alignment.hash(state);
        self.wrap.hash(state);
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: FontSize(16.0),
            line_height: LineHeight::default(),
            font_color: FontColor(Color::rgb(255, 255, 255)),
            overflow: None,
            horizontal_alignment: TextAlignment::Start,
            vertical_alignment: VerticalTextAlignment::Start,
            wrap: None,
        }
    }
}

impl TextStyle {
    pub fn new(font_size: f32, font_color: Color) -> Self {
        Self {
            font_size: font_size.into(),
            line_height: LineHeight::default(),
            font_color: FontColor(font_color),
            overflow: None,
            horizontal_alignment: TextAlignment::Start,
            vertical_alignment: VerticalTextAlignment::Start,
            wrap: None,
        }
    }

    pub fn with_font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size.into();
        self
    }

    pub fn with_line_height(mut self, line_height: f32) -> Self {
        self.line_height = line_height.into();
        self
    }

    pub fn with_overflow(mut self, overflow: TextOverflow) -> Self {
        self.overflow = Some(overflow);
        self
    }

    pub fn with_font_color(mut self, font_color: impl Into<FontColor>) -> Self {
        self.font_color = font_color.into();
        self
    }

    pub fn with_horizontal_alignment(mut self, alignment: TextAlignment) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    pub fn with_vertical_alignment(mut self, alignment: VerticalTextAlignment) -> Self {
        self.vertical_alignment = alignment;
        self
    }

    pub fn with_alignment(
        mut self,
        horizontal: TextAlignment,
        vertical: VerticalTextAlignment,
    ) -> Self {
        self.horizontal_alignment = horizontal;
        self.vertical_alignment = vertical;
        self
    }

    pub fn with_wrap(mut self, wrap: TextWrap) -> Self {
        self.wrap = Some(wrap);
        self
    }

    #[inline(always)]
    pub fn line_height_pt(&self) -> f32 {
        self.line_height.0 * self.font_size.0
    }
}
