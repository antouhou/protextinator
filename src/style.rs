//! Text styling and formatting options.
//!
//! This module provides comprehensive text styling capabilities including fonts,
//! colors, alignment, wrapping, and other visual properties for text rendering.

use crate::utils::ArcCowStr;
use cosmic_text::{Align, Color, Family};
#[cfg(feature = "serialization")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::hash::Hash;

/// Defines how text should wrap within its container.
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextWrap {
    /// Text does not wrap and may overflow the container.
    #[default]
    NoWrap,
    /// Text wraps at word boundaries when it reaches the container edge.
    Wrap,
    /// Text wraps even within words if necessary to fit the container.
    BreakWord,
}

/// Represents the line height as a multiplier of the font size.
///
/// A line height of 1.0 means the line height equals the font size.
/// Values greater than 1.0 create more spacing between lines.
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineHeight(pub f32);

impl Default for LineHeight {
    /// Returns a default line height of 1.5.
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
    /// Creates a `LineHeight` from a floating-point multiplier.
    ///
    /// # Examples
    /// ```
    /// use protextinator::style::LineHeight;
    ///
    /// let line_height: LineHeight = 1.2.into();
    /// assert_eq!(line_height.0, 1.2);
    /// ```
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl Eq for LineHeight {}

/// Represents a font size in points.
///
/// Font size determines the height of characters in the text.
/// Typical font sizes range from 8pt to 72pt, with 12pt-16pt being common for body text.
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontSize(pub f32);

impl FontSize {
    /// Creates a new `FontSize` with the specified size in points.
    ///
    /// # Arguments
    /// * `size` - The font size in points
    ///
    /// # Examples
    /// ```
    /// use protextinator::style::FontSize;
    ///
    /// let font_size = FontSize::new(16.0);
    /// assert_eq!(font_size.value(), 16.0);
    /// ```
    pub fn new(size: f32) -> Self {
        Self(size)
    }

    /// Returns the font size value in points.
    ///
    /// # Examples
    /// ```
    /// use protextinator::style::FontSize;
    ///
    /// let font_size = FontSize::new(14.0);
    /// assert_eq!(font_size.value(), 14.0);
    /// ```
    pub fn value(&self) -> f32 {
        self.0
    }
}

impl Default for FontSize {
    /// Returns a default font size of 1.5 points.
    ///
    /// Note: This is likely a placeholder value. Consider using a more standard
    /// default like 12.0 or 16.0 points for typical text.
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
    /// Creates a `FontSize` from a floating-point value in points.
    ///
    /// # Examples
    /// ```
    /// use protextinator::style::FontSize;
    ///
    /// let font_size: FontSize = 18.0.into();
    /// assert_eq!(font_size.value(), 18.0);
    /// ```
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl Eq for FontSize {}

/// Wrapper around [`cosmic_text::Color`] for text color representation.
///
/// Provides convenient constructors for creating colors from RGB and RGBA values.
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
    /// Creates a new `FontColor` from a [`cosmic_text::Color`].
    ///
    /// # Arguments
    /// * `color` - The color value
    ///
    /// # Examples
    /// ```
    /// use protextinator::style::FontColor;
    /// use cosmic_text::Color;
    ///
    /// let color = FontColor::new(Color::rgb(255, 0, 0));
    /// ```
    pub fn new(color: Color) -> Self {
        Self(color)
    }

    /// Creates a new `FontColor` from RGB values.
    ///
    /// # Arguments
    /// * `r` - Red component (0-255)
    /// * `g` - Green component (0-255)  
    /// * `b` - Blue component (0-255)
    ///
    /// # Examples
    /// ```
    /// use protextinator::style::FontColor;
    ///
    /// let red = FontColor::rgb(255, 0, 0);
    /// let green = FontColor::rgb(0, 255, 0);
    /// let blue = FontColor::rgb(0, 0, 255);
    /// ```
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self(Color::rgb(r, g, b))
    }

    /// Creates a new `FontColor` from RGBA values.
    ///
    /// # Arguments
    /// * `r` - Red component (0-255)
    /// * `g` - Green component (0-255)
    /// * `b` - Blue component (0-255)
    /// * `a` - Alpha component (0-255), where 0 is transparent and 255 is opaque
    ///
    /// # Examples
    /// ```
    /// use protextinator::style::FontColor;
    ///
    /// let semi_transparent_red = FontColor::rgba(255, 0, 0, 128);
    /// let opaque_blue = FontColor::rgba(0, 0, 255, 255);
    /// ```
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

/// Represents different font families available for text rendering.
///
/// Font families define the typeface used for rendering text. This can be
/// either a specific named font or a generic family category.
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FontFamily {
    /// A specific named font family. Have to be loaded into the font system with
    /// [`crate::TextManager::load_fonts`].
    Name(ArcCowStr),
    /// Generic sans-serif font family (e.g., Arial, Helvetica).
    SansSerif,
    /// Generic serif font family (e.g., Times New Roman, Georgia).
    Serif,
    /// Generic monospace font family (e.g., Courier, Monaco).
    Monospace,
    /// Generic cursive font family.
    Cursive,
    /// Generic fantasy font family.
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
    /// Creates a new named font family.
    ///
    /// # Arguments
    /// * `family` - The font family name. For custom fonts, the font must be loaded into the font
    ///   system using [`crate::TextManager::load_fonts`].
    ///
    /// # Examples
    /// ```
    /// use protextinator::style::FontFamily;
    ///
    /// let arial = FontFamily::new("Arial");
    /// let custom_font = FontFamily::new("MyCustomFont".to_string());
    /// ```
    pub fn new(family: impl Into<ArcCowStr>) -> Self {
        Self::Name(family.into())
    }

    /// Creates a sans-serif font family.
    ///
    /// # Examples
    /// ```
    /// use protextinator::style::FontFamily;
    ///
    /// let sans_serif = FontFamily::sans_serif();
    /// ```
    pub fn sans_serif() -> Self {
        Self::SansSerif
    }

    /// Creates a serif font family.
    ///
    /// # Examples
    /// ```
    /// use protextinator::style::FontFamily;
    ///
    /// let serif = FontFamily::serif();
    /// ```
    pub fn serif() -> Self {
        Self::Serif
    }

    /// Creates a monospace font family.
    ///
    /// # Examples
    /// ```
    /// use protextinator::style::FontFamily;
    ///
    /// let monospace = FontFamily::monospace();
    /// ```
    pub fn monospace() -> Self {
        Self::Monospace
    }

    /// Converts this font family to a [`cosmic_text::Family`] for use with the text engine.
    ///
    /// This is used internally by the text rendering system.
    pub fn to_fontdb_family(&self) -> Family {
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

/// Comprehensive text styling configuration.
///
/// `TextStyle` combines all visual aspects of text rendering, including font properties,
/// colors, alignment and wrapping behavior
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextStyle {
    /// The font size in points.
    pub font_size: FontSize,
    /// The line height is a multiplier of the font size.
    pub line_height: LineHeight,
    /// The color of the text.
    pub font_color: FontColor,
    /// Horizontal text alignment within the container.
    pub horizontal_alignment: HorizontalTextAlignment,
    /// Vertical text alignment within the container.
    pub vertical_alignment: VerticalTextAlignment,
    /// Text wrapping behavior.
    pub wrap: Option<TextWrap>,
    /// The font family to use for rendering.
    pub font_family: FontFamily,
}

/// Horizontal text alignment options.
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub enum HorizontalTextAlignment {
    /// No horizontal alignment, defaulting to the start of the text area. Set alignment
    /// to `None` to be able to scroll horizontally.
    #[default]
    None,
    /// Align text to the start of the container (left in LTR, right in RTL).
    Start,
    /// Align text to the end of the container (right in LTR, left in RTL).
    End,
    /// Center text within the container.
    Center,
    /// Align text to the left edge of the container.
    Left,
    /// Align text to the right edge of the container.
    Right,
    /// Justify text to fill the container width.
    Justify,
}

impl From<HorizontalTextAlignment> for Option<Align> {
    /// Converts a `TextAlignment` to a [`cosmic_text::Align`] option.
    ///
    /// Returns `None` for `Start` alignment (default behavior), and the corresponding
    /// `Align` variant for other alignment types.
    fn from(val: HorizontalTextAlignment) -> Self {
        match val {
            HorizontalTextAlignment::None => None,
            HorizontalTextAlignment::Start => None,
            HorizontalTextAlignment::End => Some(Align::End),
            HorizontalTextAlignment::Center => Some(Align::Center),
            HorizontalTextAlignment::Left => Some(Align::Left),
            HorizontalTextAlignment::Right => Some(Align::Right),
            HorizontalTextAlignment::Justify => Some(Align::Justified),
        }
    }
}

/// Vertical text alignment options within a container.
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub enum VerticalTextAlignment {
    /// No vertical alignment, defaulting to the top of the text area. Text can be scrolled vertically.
    #[default]
    None,
    /// Aligns text to the top of the text area.
    Start,
    /// Aligns text to the bottom of the text area.
    End,
    /// Centers text vertically within the text area.
    Center,
}

impl Hash for TextStyle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.font_size.hash(state);
        self.line_height.hash(state);
        self.font_color.hash(state);
        self.horizontal_alignment.hash(state);
        self.vertical_alignment.hash(state);
        self.wrap.hash(state);
    }
}

impl Default for TextStyle {
    /// Creates a default text style with standard settings.
    ///
    /// Default values:
    /// - Font size: 16.0 points
    /// - Line height: 1.5x font size
    /// - Font color: White (255, 255, 255)
    /// - No overflow handling
    /// - Start horizontal alignment
    /// - Start vertical alignment  
    /// - No text wrapping
    /// - Sans-serif font family
    fn default() -> Self {
        Self {
            font_size: FontSize(16.0),
            line_height: LineHeight::default(),
            font_color: FontColor(Color::rgb(255, 255, 255)),
            horizontal_alignment: HorizontalTextAlignment::Start,
            vertical_alignment: VerticalTextAlignment::Start,
            wrap: None,
            font_family: FontFamily::SansSerif,
        }
    }
}

impl TextStyle {
    /// Creates a new `TextStyle` with the specified font size and color.
    ///
    /// Other properties are set to their default values.
    ///
    /// # Arguments
    /// * `font_size` - The font size in points
    /// * `font_color` - The text color
    ///
    /// # Examples
    /// ```
    /// use protextinator::style::TextStyle;
    /// use cosmic_text::Color;
    ///
    /// let style = TextStyle::new(14.0, Color::rgb(0, 0, 0));
    /// ```
    pub fn new(font_size: f32, font_color: Color) -> Self {
        Self {
            font_size: font_size.into(),
            line_height: LineHeight::default(),
            font_color: FontColor(font_color),
            horizontal_alignment: HorizontalTextAlignment::Start,
            vertical_alignment: VerticalTextAlignment::Start,
            wrap: None,
            font_family: FontFamily::SansSerif,
        }
    }

    /// Sets the font size and returns the modified style.
    ///
    /// # Arguments
    /// * `font_size` - The font size in points
    ///
    /// # Examples
    /// ```
    /// use protextinator::style::TextStyle;
    ///
    /// let style = TextStyle::default().with_font_size(18.0);
    /// ```
    pub fn with_font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size.into();
        self
    }

    /// Sets the line height and returns the modified style.
    ///
    /// # Arguments
    /// * `line_height` - The line height as a multiplier of font size
    ///
    /// # Examples
    /// ```
    /// use protextinator::style::TextStyle;
    ///
    /// let style = TextStyle::default().with_line_height(1.2);
    /// ```
    pub fn with_line_height(mut self, line_height: f32) -> Self {
        self.line_height = line_height.into();
        self
    }

    /// Sets the font color and returns the modified style.
    ///
    /// # Arguments
    /// * `font_color` - The text color
    ///
    /// # Examples
    /// ```
    /// use protextinator::style::{TextStyle, FontColor};
    ///
    /// let style = TextStyle::default().with_font_color(FontColor::rgb(255, 0, 0));
    /// ```
    pub fn with_font_color(mut self, font_color: impl Into<FontColor>) -> Self {
        self.font_color = font_color.into();
        self
    }

    /// Sets the horizontal alignment and returns the modified style.
    ///
    /// # Arguments
    /// * `alignment` - The horizontal alignment
    ///
    /// # Examples
    /// ```
    /// use protextinator::style::{TextStyle, HorizontalTextAlignment};
    ///
    /// let style = TextStyle::default().with_horizontal_alignment(HorizontalTextAlignment::Center);
    /// ```
    pub fn with_horizontal_alignment(mut self, alignment: HorizontalTextAlignment) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    /// Sets the vertical alignment and returns the modified style.
    ///
    /// # Arguments
    /// * `alignment` - The vertical alignment
    ///
    /// # Examples
    /// ```
    /// use protextinator::style::{TextStyle, VerticalTextAlignment};
    ///
    /// let style = TextStyle::default().with_vertical_alignment(VerticalTextAlignment::Center);
    /// ```
    pub fn with_vertical_alignment(mut self, alignment: VerticalTextAlignment) -> Self {
        self.vertical_alignment = alignment;
        self
    }

    /// Sets both horizontal and vertical alignment and returns the modified style.
    ///
    /// # Arguments
    /// * `horizontal` - The horizontal alignment
    /// * `vertical` - The vertical alignment
    ///
    /// # Examples
    /// ```
    /// use protextinator::style::{TextStyle, HorizontalTextAlignment, VerticalTextAlignment};
    ///
    /// let style = TextStyle::default().with_alignment(
    ///     HorizontalTextAlignment::Center,
    ///     VerticalTextAlignment::Center
    /// );
    /// ```
    pub fn with_alignment(
        mut self,
        horizontal: HorizontalTextAlignment,
        vertical: VerticalTextAlignment,
    ) -> Self {
        self.horizontal_alignment = horizontal;
        self.vertical_alignment = vertical;
        self
    }

    /// Sets the text wrapping behavior and returns the modified style.
    ///
    /// # Arguments
    /// * `wrap` - The text wrapping behavior
    ///
    /// # Examples
    /// ```
    /// use protextinator::style::{TextStyle, TextWrap};
    ///
    /// let style = TextStyle::default().with_wrap(TextWrap::Wrap);
    /// ```
    pub fn with_wrap(mut self, wrap: TextWrap) -> Self {
        self.wrap = Some(wrap);
        self
    }

    /// Calculates the line height in points based on the font size and line height multiplier.
    ///
    /// # Returns
    /// The line height in points (font_size * line_height_multiplier)
    ///
    /// # Examples
    /// ```
    /// use protextinator::style::TextStyle;
    ///
    /// let style = TextStyle::default().with_font_size(16.0).with_line_height(1.5);
    /// assert_eq!(style.line_height_pt(), 24.0); // 16.0 * 1.5
    /// ```
    #[inline(always)]
    pub fn line_height_pt(&self) -> f32 {
        self.line_height.0 * self.font_size.0
    }
}
