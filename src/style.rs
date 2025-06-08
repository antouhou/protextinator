use cosmic_text::{Align, Color, Family};
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
        serializer.serialize_u32(self.0 .0)
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FontFamily {
    Name(Box<str>),
    SansSerif,
    Serif,
    Monospace,
    Cursive,
    Fantasy,
}

impl From<&'static str> for FontFamily {
    fn from(value: &'static str) -> Self {
        Self::Name(value.into())
    }
}

impl From<String> for FontFamily {
    fn from(value: String) -> Self {
        Self::Name(value.into())
    }
}

impl FontFamily {
    pub fn new(family: impl Into<Box<str>>) -> Self {
        Self::Name(family.into())
    }

    pub fn sans_serif() -> Self {
        Self::SansSerif
    }

    pub fn serif() -> Self {
        Self::Serif
    }

    pub fn monospace() -> Self {
        Self::Monospace
    }

    pub fn to_fontdb_family<'a>(&'a self) -> Family<'a> {
        match self {
            FontFamily::Name(a) => Family::Name(a),
            FontFamily::SansSerif => Family::SansSerif,
            FontFamily::Serif => Family::Serif,
            FontFamily::Monospace => Family::Monospace,
            FontFamily::Cursive => Family::Cursive,
            FontFamily::Fantasy => Family::Fantasy,
        }
    }
}

#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub font_family: FontFamily,
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

impl From<TextAlignment> for Option<Align> {
    fn from(val: TextAlignment) -> Self {
        match val {
            TextAlignment::Start => None,
            TextAlignment::End => Some(Align::End),
            TextAlignment::Center => Some(Align::Center),
            TextAlignment::Left => Some(Align::Left),
            TextAlignment::Right => Some(Align::Right),
            TextAlignment::Justify => Some(Align::Justified),
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
            font_family: FontFamily::SansSerif,
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
            font_family: FontFamily::SansSerif,
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
