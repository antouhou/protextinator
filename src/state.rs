//! Text state management and editing functionality.
//!
//! This module provides the core `TextState` type and related functionality for managing
//! text content, cursor position, selection, scrolling, and text editing operations.

use crate::action::{Action, ActionResult};
use crate::buffer_utils::{
    adjust_vertical_scroll_to_make_caret_visible, char_under_position, update_buffer,
    vertical_offset,
};
use crate::byte_cursor::ByteCursor;
use crate::math::Size;
use crate::style::{TextStyle, VerticalTextAlignment};
use crate::text_manager::TextContext;
use crate::text_params::TextParams;
use crate::{Point, Rect};
#[cfg(test)]
use cosmic_text::LayoutGlyph;
use cosmic_text::{Buffer, Cursor, Edit, Editor, FontSystem, Motion};
use smol_str::SmolStr;
use std::time::{Duration, Instant};

/// Size comparison epsilon for floating-point calculations.
pub const SIZE_EPSILON: f32 = 0.0001;

/// Represents a single line of text selection with visual boundaries.
///
/// Selection lines define the visual appearance of selected text, with start and end
/// coordinates for rendering selection highlights.
#[derive(Clone, Default, Debug, Copy)]
pub struct SelectionLine {
    /// X coordinate where the selection starts on this line.
    pub start_x_pt: Option<f32>,
    /// Y coordinate where the selection starts on this line.
    pub start_y_pt: Option<f32>,
    /// X coordinate where the selection ends on this line.
    pub end_x_pt: Option<f32>,
    /// Y coordinate where the selection ends on this line.
    pub end_y_pt: Option<f32>,
}

/// Represents the current text selection state.
///
/// A selection is defined by an origin point (where selection started) and an end point
/// (where selection currently ends). The selection can span multiple lines, with each
/// line's visual boundaries stored in the `lines` vector.
#[derive(Clone, Default, Debug)]
pub struct Selection {
    origin_character_byte_cursor: Option<ByteCursor>,
    ends_before_character_byte_cursor: Option<ByteCursor>,
    lines: Vec<SelectionLine>,
}

impl Selection {
    /// Returns `true` if there is no active selection.
    ///
    /// A selection is considered empty if either the origin or end cursor is not set.
    ///
    /// # Examples
    /// ```
    /// use protextinator::Selection;
    ///
    /// let selection = Selection::default();
    /// assert!(selection.is_empty());
    /// ```
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.origin_character_byte_cursor.is_none()
            || self.ends_before_character_byte_cursor.is_none()
    }

    /// Returns the visual selection lines for rendering.
    ///
    /// Each line represents a portion of the selection with its visual boundaries.
    ///
    /// # Returns
    /// A slice of `SelectionLine` objects representing the visual selection
    #[inline(always)]
    pub fn lines(&self) -> &[SelectionLine] {
        &self.lines
    }
}

/// The main text state container that manages text content, cursor, selection, and styling.
///
/// `TextState` is the core type for text editing functionality. It maintains the text buffer,
/// cursor position, selection state, scroll position, and handles text editing operations.
///
/// # Type Parameters
/// * `T` - Custom metadata type that can be attached to the text state
///
/// # Features
/// - Text editing (insert, delete, copy, paste, cut)
/// - Cursor movement and positioning
/// - Text selection with visual feedback
/// - Automatic scrolling to keep cursor visible
/// - Rich text styling
/// - Configurable editing behavior
#[derive(Debug)]
pub struct TextState<T> {
    params: TextParams,
    cursor: ByteCursor,
    // Caret position relative to the buffer viewport with scroll applied
    relative_caret_position: Option<Point>,
    caret_width: f32,
    selection: Selection,

    last_scroll_timestamp: Instant,

    inner_dimensions: Size,
    buffer: Buffer,

    // Settings
    /// Can text be selected?
    pub is_selectable: bool,
    /// Can text be edited?
    pub is_editable: bool,
    /// Various actions, such as copy, paste, cut, etc., are going to be performed
    pub is_editing: bool,
    /// Are actions enabled? If false, no actions will be performed.
    pub are_actions_enabled: bool,
    /// Interval between scroll updates when dragging the selection
    pub scroll_interval: Duration,

    /// Doesn't affect anything - just some metadata that you can later use during rendering
    pub metadata: T,
}

impl<T> TextState<T> {
    /// Creates a new text state with the specified text content and metadata.
    ///
    /// The text state is created with default settings:
    /// - Editing and selection disabled
    /// - Actions disabled
    /// - Default caret width of 3.0 pixels
    /// - 50ms scroll interval
    ///
    /// # Arguments
    /// * `text` - The initial text content
    /// * `font_system` - Mutable reference to the font system for text layout
    /// * `metadata` - Custom metadata to associate with this text state
    ///
    /// # Returns
    /// A new `TextState` instance
    ///
    /// # Examples
    /// ```
    /// use protextinator::{TextState, TextContext};
    /// use cosmic_text::FontSystem;
    ///
    /// let mut font_system = FontSystem::new();
    /// let state = TextState::new_with_text("Hello, world!", &mut font_system, ());
    /// ```
    pub fn new_with_text(
        text: impl Into<String>,
        font_system: &mut FontSystem,
        metadata: T,
    ) -> Self {
        let text = text.into();
        let params = TextParams::new(Size::ZERO, TextStyle::default(), text, 0);
        let metrics = params.metrics();

        Self {
            params,

            is_editing: false,
            are_actions_enabled: false,

            cursor: ByteCursor::default(),
            relative_caret_position: None,

            selection: Selection::default(),
            last_scroll_timestamp: Instant::now(),
            scroll_interval: Duration::from_millis(50),
            caret_width: 3.0,
            is_selectable: false,
            is_editable: false,

            inner_dimensions: Size::ZERO,
            buffer: Buffer::new(font_system, metrics),

            metadata,
        }
    }

    /// Sets the caret width, which is the width of the cursor when editing text.
    ///
    /// # Arguments
    /// * `width` - The caret width in pixels
    ///
    /// # Examples
    /// ```
    /// # use protextinator::TextState;
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let mut state = TextState::new_with_text("", &mut font_system, ());
    /// state.set_caret_width(2.0);
    /// assert_eq!(state.caret_width(), 2.0);
    /// ```
    pub fn set_caret_width(&mut self, width: f32) {
        self.caret_width = width;
    }

    /// Returns the caret width, which is the width of the cursor when editing text.
    ///
    /// # Returns
    /// The caret width in pixels
    ///
    /// # Examples
    /// ```
    /// # use protextinator::TextState;
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let state = TextState::new_with_text("", &mut font_system, ());
    /// let width = state.caret_width();
    /// ```
    pub fn caret_width(&self) -> f32 {
        self.caret_width
    }

    /// Caret position relative to the buffer viewport with scroll applied. Returns `None` if
    /// the caret is not visible or the buffer is not shaped yet.
    ///
    /// # Returns
    /// The caret position relative to the viewport, or `None` if not visible
    ///
    /// # Examples
    /// ```
    /// # use protextinator::TextState;
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let state = TextState::new_with_text("Hello", &mut font_system, ());
    /// if let Some(position) = state.caret_position_relative() {
    ///     println!("Caret at: ({}, {})", position.x, position.y);
    /// }
    /// ```
    pub fn caret_position_relative(&self) -> Option<Point> {
        self.relative_caret_position
    }

    /// Returns the position of the selection lines in the buffer viewport.
    ///
    /// # Returns
    /// A reference to the current selection state
    ///
    /// # Examples
    /// ```
    /// # use protextinator::TextState;
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let state = TextState::new_with_text("Hello", &mut font_system, ());
    /// let selection = state.selection();
    /// if !selection.is_empty() {
    ///     println!("Text is selected");
    /// }
    /// ```
    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    /// Sets the text in the buffer and updates the cursor position if necessary.
    ///
    /// This method only updates the text content without reshaping. You'll need to call
    /// `recalculate` or `reshape_if_params_changed` separately to update the layout.
    ///
    /// # Arguments
    /// * `text` - The new text content
    ///
    /// # Examples
    /// ```
    /// # use protextinator::TextState;
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let mut state = TextState::new_with_text("", &mut font_system, ());
    /// state.set_text("Updated text");
    /// assert_eq!(state.text(), "Updated text");
    /// ```
    pub fn set_text(&mut self, text: &str) {
        self.params.set_text(text);

        if self.cursor.byte_character_start > self.params.text_for_internal_use().len() {
            self.update_cursor_before_glyph_with_bytes_offset(
                self.params.text_for_internal_use().len(),
            );
        }
    }

    /// Returns the text in the buffer
    ///
    /// # Returns
    /// The current text content as a string slice
    ///
    /// # Examples
    /// ```
    /// # use protextinator::TextState;
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let state = TextState::new_with_text("Hello, world!", &mut font_system, ());
    /// assert_eq!(state.text(), "Hello, world!");
    /// ```
    pub fn text(&self) -> &str {
        self.params.original_text()
    }

    /// Sets the text style
    ///
    /// # Arguments
    /// * `style` - The new text style to apply
    ///
    /// # Examples
    /// ```
    /// # use protextinator::{TextState, style::TextStyle};
    /// # use cosmic_text::{FontSystem, Color};
    /// # let mut font_system = FontSystem::new();
    /// # let mut state = TextState::new_with_text("", &mut font_system, ());
    /// let style = TextStyle::new(16.0, Color::rgb(255, 0, 0));
    /// state.set_style(&style);
    /// ```
    pub fn set_style(&mut self, style: &TextStyle) {
        self.params.set_style(style);
    }

    /// Returns the text style
    ///
    /// # Returns
    /// A reference to the current text style
    ///
    /// # Examples
    /// ```
    /// # use protextinator::TextState;
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let state = TextState::new_with_text("", &mut font_system, ());
    /// let style = state.style();
    /// println!("Font size: {}", style.font_size.value());
    /// ```
    pub fn style(&self) -> &TextStyle {
        self.params.style()
    }

    /// Sets the visible area of the text buffer. This is going to be used to determine the buffer's
    /// viewport size and how much text is visible.
    ///
    /// # Arguments
    /// * `size` - The new visible area size
    ///
    /// # Examples
    /// ```
    /// # use protextinator::{TextState, math::Size};
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let mut state = TextState::new_with_text("", &mut font_system, ());
    /// state.set_outer_size(&Size::new(400.0, 200.0));
    /// ```
    pub fn set_outer_size(&mut self, size: &Size) {
        self.params.set_size(size)
    }

    /// Metadata set to a cosmic_text's buffer
    ///
    /// # Returns
    /// The current buffer metadata value
    ///
    /// # Examples
    /// ```
    /// # use protextinator::TextState;
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let state = TextState::new_with_text("", &mut font_system, ());
    /// let metadata = state.buffer_metadata();
    /// ```
    pub fn buffer_metadata(&self) -> usize {
        self.params.metadata()
    }

    /// Sets the metadata for the text cosmic_text's buffer. Note that this is different from
    /// the `metadata` field in `TextState`, which is a custom type.
    ///
    /// # Arguments
    /// * `metadata` - The metadata value to set
    ///
    /// # Examples
    /// ```
    /// # use protextinator::TextState;
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let mut state = TextState::new_with_text("", &mut font_system, ());
    /// state.set_buffer_metadata(42);
    /// assert_eq!(state.buffer_metadata(), 42);
    /// ```
    #[inline(always)]
    pub fn set_buffer_metadata(&mut self, metadata: usize) {
        self.params.set_metadata(metadata)
    }

    /// Returns the visible area size of the text buffer. Note that this is set directly by the
    /// `set_outer_size` method, and it does not represent the actual text dimensions. To get the
    /// inner dimensions of the text buffer, use `inner_size`.
    ///
    /// # Returns
    /// The outer size (visible area) of the text buffer
    ///
    /// # Examples
    /// ```
    /// # use protextinator::{TextState, math::Size};
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let mut state = TextState::new_with_text("", &mut font_system, ());
    /// state.set_outer_size(&Size::new(400.0, 200.0));
    /// assert_eq!(state.outer_size(), Size::new(400.0, 200.0));
    /// ```
    pub fn outer_size(&self) -> Size {
        self.params.size()
    }

    /// Returns the inner dimensions of the text buffer. This represents the actual size of the text
    /// content, which may differ from the outer size if the text is larger than the visible area.
    ///
    /// # Returns
    /// The inner dimensions representing the actual text content size
    ///
    /// # Examples
    /// ```
    /// # use protextinator::TextState;
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let state = TextState::new_with_text("Some text", &mut font_system, ());
    /// let inner_size = state.inner_size();
    /// println!("Text content size: {}x{}", inner_size.x, inner_size.y);
    /// ```
    pub fn inner_size(&self) -> Size {
        self.inner_dimensions
    }

    /// Returns the text buffer that can be used for rendering
    ///
    /// # Returns
    /// A reference to the underlying cosmic-text Buffer
    ///
    /// # Examples
    /// ```
    /// # use protextinator::TextState;
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let state = TextState::new_with_text("Hello", &mut font_system, ());
    /// let buffer = state.buffer();
    /// // Use buffer for rendering operations
    /// ```
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Returns the length of the text in characters. Note that this is different from the
    /// string .len(), which returns the length in bytes.
    ///
    /// # Returns
    /// The number of Unicode characters in the text
    ///
    /// # Examples
    /// ```
    /// # use protextinator::TextState;
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let state = TextState::new_with_text("Hello 🦀", &mut font_system, ());
    /// assert_eq!(state.text_char_len(), 7); // 5 ASCII chars + 1 space + 1 emoji
    /// ```
    pub fn text_char_len(&self) -> usize {
        self.params.original_text().chars().count()
    }

    /// Returns the char index of the cursor in the text buffer. Note that this returns the
    /// char index, not the char byte index.
    ///
    /// # Returns
    /// The character index of the cursor, or `None` if the cursor position is invalid
    ///
    /// # Examples
    /// ```
    /// # use protextinator::TextState;
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let state = TextState::new_with_text("Hello", &mut font_system, ());
    /// if let Some(index) = state.cursor_char_index() {
    ///     println!("Cursor is at character index: {}", index);
    /// }
    /// ```
    pub fn cursor_char_index(&self) -> Option<usize> {
        self.cursor.char_index(self.params.text_for_internal_use())
    }

    fn insert_char_at_cursor(&mut self, character: char, ctx: &mut TextContext) -> ActionResult {
        self.params
            .insert_char(self.cursor.byte_character_start, character);
        self.reshape_if_params_changed(ctx);
        self.move_cursor(ctx, Motion::Next);

        ActionResult::TextChanged
    }

    // fn insert_text_at_cursor(&mut self, text: &str) {
    //     self.params
    //         .insert_str(self.cursor.byte_character_start, text);
    //     self.update_cursor_before_glyph_with_bytes_offset(
    //         self.cursor.byte_character_start + text.len(),
    //     );
    // }

    fn remove_char_at_cursor(&mut self) {
        if !self.params.text_for_internal_use().is_empty() {
            if let Some(prev_char) = self
                .cursor
                .prev_char_byte_offset(self.params.text_for_internal_use())
            {
                self.remove_character(prev_char);
                if !self
                    .cursor
                    .update_byte_offset(prev_char, self.params.text_for_internal_use())
                {
                    // TODO: print a warning
                }
            }
        }
    }

    fn remove_characters(&mut self, byte_offset_start: usize, byte_offset_end: usize) {
        self.params.remove_range(byte_offset_start, byte_offset_end);
    }

    fn set_cursor_before_glyph(&mut self, cursor: ByteCursor) {
        self.cursor = cursor;
    }

    fn update_cursor_before_glyph_with_cursor(&mut self, cursor: Cursor) {
        self.cursor
            .update_cursor(cursor, self.params.text_for_internal_use());
    }

    fn update_cursor_before_glyph_with_bytes_offset(&mut self, byte_offset: usize) {
        self.cursor
            .update_byte_offset(byte_offset, self.params.text_for_internal_use());
    }

    fn remove_character(&mut self, byte_offset: usize) -> Option<char> {
        self.params.remove_char(byte_offset)
    }

    fn remove_selected_text(&mut self) -> Option<()> {
        if let (Some(origin), Some(end)) = (
            self.selection.origin_character_byte_cursor,
            self.selection.ends_before_character_byte_cursor,
        ) {
            let origin_offset = origin.byte_character_start;
            let end_offset = end.byte_character_start;

            if origin > end {
                self.remove_characters(end_offset, origin_offset);
                self.cursor = end;
            } else {
                self.remove_characters(origin_offset, end_offset);
                self.cursor = origin;
            }
            self.reset_selection();
            Some(())
        } else {
            None
        }
    }

    fn move_cursor_to_selection_left(&mut self) {
        if let (Some(origin), Some(end)) = (
            self.selection.origin_character_byte_cursor,
            self.selection.ends_before_character_byte_cursor,
        ) {
            if origin > end {
                self.set_cursor_before_glyph(end);
            } else {
                self.set_cursor_before_glyph(origin);
            }
        }
    }

    fn move_cursor_to_selection_right(&mut self) {
        if let (Some(origin), Some(end)) = (
            self.selection.origin_character_byte_cursor,
            self.selection.ends_before_character_byte_cursor,
        ) {
            if origin < end {
                self.set_cursor_before_glyph(end);
            } else {
                self.set_cursor_before_glyph(origin);
            }
        }
    }

    /// Checks if there is a text selection that is not empty.
    ///
    /// # Returns
    /// `true` if text is currently selected, `false` otherwise
    ///
    /// # Examples
    /// ```
    /// # use protextinator::TextState;
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let state = TextState::new_with_text("Hello", &mut font_system, ());
    /// if state.is_text_selected() {
    ///     println!("Some text is selected");
    /// }
    /// ```
    pub fn is_text_selected(&self) -> bool {
        if let Some(origin) = self.selection.origin_character_byte_cursor {
            if let Some(end) = self.selection.ends_before_character_byte_cursor {
                origin != end
            } else {
                false
            }
        } else {
            false
        }
    }

    fn reset_selection_end(&mut self) {
        self.selection.ends_before_character_byte_cursor = None;
        self.selection.lines.clear();
    }

    /// Clears the current text selection.
    ///
    /// This removes any active text selection, returning the text state to having
    /// only a cursor position without any selected text.
    ///
    /// # Examples
    /// ```
    /// # use protextinator::TextState;
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let mut state = TextState::new_with_text("Hello", &mut font_system, ());
    /// state.reset_selection();
    /// assert!(!state.is_text_selected());
    /// ```
    pub fn reset_selection(&mut self) {
        self.selection.origin_character_byte_cursor = None;
        self.selection.ends_before_character_byte_cursor = None;
        self.selection.lines.clear();
    }

    fn select_all(&mut self) {
        self.selection.origin_character_byte_cursor = Some(ByteCursor::string_start());
        if !self.params.original_text().is_empty() {
            self.selection.ends_before_character_byte_cursor = Some(
                ByteCursor::after_last_character(self.params.original_text()),
            )
        } else {
            self.selection.ends_before_character_byte_cursor = None;
        }
    }

    fn substring_byte_offset(&self, start: usize, end: usize) -> &str {
        // TODO: add bounds checking
        &self.params.original_text()[start..end]
    }

    /// Returns the selected text as a substring based on the selection start and end byte offsets.
    /// You also can get the selected text by using [`TextState::apply_action`] with
    /// [`Action::CopySelectedText`].
    ///
    /// # Returns
    /// The currently selected text, or `None` if no text is selected
    ///
    /// # Examples
    /// ```
    /// # use protextinator::TextState;
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let state = TextState::new_with_text("Hello, world!", &mut font_system, ());
    /// if let Some(selected) = state.selected_text() {
    ///     println!("Selected text: {}", selected);
    /// }
    /// ```
    pub fn selected_text(&self) -> Option<&str> {
        if let (Some(mut origin), Some(mut end)) = (
            self.selection.origin_character_byte_cursor,
            self.selection.ends_before_character_byte_cursor,
        ) {
            if origin > end {
                std::mem::swap(&mut origin, &mut end);
            }
            Some(self.substring_byte_offset(origin.byte_character_start, end.byte_character_start))
        } else {
            None
        }
    }

    //
    /// Gets the current absolute scroll position of the text buffer. Note that
    /// the buffer must be shaped and updated before calling this function, i.e. if anything
    /// changed in the text, you should call [`TextState::recalculate`].
    ///
    /// The scroll position represents how much the text content has been scrolled
    /// from its original position. This accounts for both horizontal and vertical scrolling.
    ///
    /// # Returns
    /// A `Point` representing the absolute scroll offset
    ///
    /// # Note
    /// The buffer must be shaped and updated before calling this function for accurate results.
    ///
    /// # Examples
    /// ```
    /// # use protextinator::TextState;
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let state = TextState::new_with_text("Hello", &mut font_system, ());
    /// let scroll = state.absolute_scroll();
    /// println!("Scrolled by: ({}, {})", scroll.x, scroll.y);
    /// ```
    pub fn absolute_scroll(&self) -> Point {
        let scroll = self.buffer.scroll();
        let scroll_line = scroll.line;
        let scroll_vertical = scroll.vertical;
        let scroll_horizontal = scroll.horizontal;
        let mut line_vertical_start = 0.0;
        let line_height = self.style().line_height_pt();
        for (line_i, line) in self.buffer.lines.iter().enumerate() {
            if line_i == scroll_line {
                // Found line
                break;
            }
            if let Some(layout_lines) = line.layout_opt() {
                for layout_line in layout_lines {
                    line_vertical_start += layout_line.line_height_opt.unwrap_or(line_height);
                }
            }
        }

        Point {
            x: scroll_horizontal,
            y: scroll_vertical + line_vertical_start,
        }
    }

    /// Sets the absolute scroll position of the text buffer. Note that text that has fixed
    /// alignment (e.g. `VerticalTextAlignment::Top`) will not be affected by this method,
    /// and the scroll position will be calculated based on the current text layout and line
    /// heights. For the scroll to take effect, alignment must be set to
    /// `VerticalTextAlignment::None`.
    ///
    /// This allows you to programmatically scroll the text content to a specific position.
    /// The scroll position is calculated based on line heights and text layout.
    ///
    /// # Arguments
    /// * `scroll` - The absolute scroll position to set
    ///
    /// # Examples
    /// ```
    /// # use protextinator::{TextState, math::Point};
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let mut state = TextState::new_with_text("Hello\nWorld\nMore\nText", &mut font_system, ());
    /// state.set_absolute_scroll(Point::new(0.0, 50.0));
    /// ```
    pub fn set_absolute_scroll(&mut self, scroll: Point) {
        let mut new_scroll = self.buffer.scroll();

        let can_scroll_vertically =
            matches!(self.style().vertical_alignment, VerticalTextAlignment::None);

        new_scroll.horizontal = scroll.x;

        if can_scroll_vertically {
            let line_height = self.style().line_height_pt();
            let mut line_index = 0;
            let mut accumulated_height = 0.0;

            for (i, line) in self.buffer.lines.iter().enumerate() {
                let mut line_height_total = 0.0;

                if let Some(layout_lines) = line.layout_opt() {
                    for layout_line in layout_lines {
                        line_height_total += layout_line.line_height_opt.unwrap_or(line_height);
                    }
                }

                if accumulated_height + line_height_total > scroll.y {
                    line_index = i;
                    break;
                }

                accumulated_height += line_height_total;
                line_index = i + 1; // In case we don't break, this will be the last line
            }

            // Set the line and calculate the remaining vertical offset
            new_scroll.line = line_index;
            new_scroll.vertical = scroll.y - accumulated_height;
        }

        self.buffer.set_scroll(new_scroll);
    }

    /// Calculates physical selection area based on the selection start and end glyph indices
    fn recalculate_selection_area(&mut self) -> Option<()> {
        if !self.is_selectable {
            return None;
        }

        let mut selection_starts_at_index = self.selection.origin_character_byte_cursor?;
        let mut selection_ends_before_char_index =
            self.selection.ends_before_character_byte_cursor?;
        if selection_starts_at_index > selection_ends_before_char_index {
            // Swap the values
            std::mem::swap(
                &mut selection_ends_before_char_index,
                &mut selection_starts_at_index,
            );
        }

        let start_cursor = selection_starts_at_index;
        let end_cursor = selection_ends_before_char_index;

        self.selection.lines.clear();
        for run in self.buffer.layout_runs() {
            if let Some((start_x, width)) = run.highlight(start_cursor.cursor, end_cursor.cursor) {
                self.selection.lines.push(SelectionLine {
                    // TODO: cosmic test doesn't seem to correctly apply horizontal scrolling
                    start_x_pt: Some(start_x - self.buffer.scroll().horizontal),
                    end_x_pt: Some(start_x + width - self.buffer.scroll().horizontal),
                    start_y_pt: Some(run.line_top),
                    end_y_pt: Some(run.line_top + run.line_height),
                });
            }
        }

        None
    }

    pub fn recalculate_with_update_reason(
        &mut self,
        ctx: &mut TextContext,
        update_reason: UpdateReason,
    ) {
        self.reshape_if_params_changed(ctx);
        self.adjust_scroll_if_cursor_moved(update_reason, &mut ctx.font_system);
        // TODO: do only if scroll/selection changed
        self.recalculate_selection_area();

        // TODO: do that if the buffer was reshaped
        self.relative_caret_position = self.calculate_caret_position();
        self.align_vertically();
    }

    /// Recalculates and reshapes the text buffer, scroll, caret position, and selection area.
    /// The results are cached, so don't be afraid to call this function multiple times.
    ///
    /// This is the main method to call after making changes to text content, style, or size
    /// to ensure the visual representation is updated correctly.
    ///
    /// # Arguments
    /// * `ctx` - Mutable reference to the text context for processing
    ///
    /// # Examples
    /// ```
    /// # use protextinator::{TextState, TextContext};
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let mut state = TextState::new_with_text("Hello", &mut font_system, ());
    /// # let mut ctx = TextContext::default();
    /// state.recalculate(&mut ctx);
    /// ```
    pub fn recalculate(&mut self, ctx: &mut TextContext) {
        self.recalculate_with_update_reason(ctx, UpdateReason::Unknown);
    }

    fn calculate_caret_position(&mut self) -> Option<Point> {
        let horizontal_scroll = self.buffer.scroll().horizontal;
        let mut editor = Editor::new(&mut self.buffer);
        editor.set_cursor(self.cursor.cursor);

        editor.cursor_position().map(|pos| {
            let mut point = Point::from(pos);
            // Adjust the point to account for horizontal scroll, as cosmic_text does not
            //  support horizontal scrolling natively.
            point.x -= horizontal_scroll;
            point
        })
    }

    fn align_vertically(&mut self) {
        if matches!(self.style().vertical_alignment, VerticalTextAlignment::None) {
            return;
        }

        let mut scroll = self.buffer.scroll();
        let text_area_size = self.params.size();
        let vertical_scroll_to_align_text =
            calculate_vertical_offset(self.params.style(), text_area_size, self.inner_dimensions);
        scroll.vertical = vertical_scroll_to_align_text;
        self.buffer.set_scroll(scroll);
    }

    /// Buffer needs to be shaped before calling this function, as it relies on the buffer's layout
    /// and dimensions.
    fn adjust_scroll_if_cursor_moved(
        &mut self,
        update_reason: UpdateReason,
        font_system: &mut FontSystem,
    ) -> Option<()> {
        if update_reason.is_cursor_updated() {
            let text_area_size = self.params.size();
            let old_scroll = self.buffer.scroll();
            let old_relative_caret_x = self.relative_caret_position.map_or(0.0, |p| p.x);
            let old_absolute_caret_x = old_relative_caret_x + old_scroll.horizontal;

            let caret_position_relative_to_buffer = adjust_vertical_scroll_to_make_caret_visible(
                &mut self.buffer,
                self.cursor,
                font_system,
                self.params.size(),
                self.params.style(),
            )?;
            let mut new_scroll = self.buffer.scroll();
            let text_area_width = text_area_size.x;

            // TODO: there was some other implementation that took horizontal alignment into account,
            //  check if it is needed
            let new_absolute_caret_offset = caret_position_relative_to_buffer.x;

            // TODO: A little hack to set horizontal scroll
            let current_absolute_visible_text_area = (
                old_scroll.horizontal,
                old_scroll.horizontal + text_area_width,
            );
            let min = current_absolute_visible_text_area.0;
            let max = current_absolute_visible_text_area.1;
            let is_new_caret_visible =
                new_absolute_caret_offset >= min && new_absolute_caret_offset <= max;

            // If the caret is within the visible text area, we don't need to scroll.
            //  In that case, we should return the old scroll and modify the caret offset
            if is_new_caret_visible {
                let is_moving_caret_without_updating_the_text =
                    matches!(update_reason, UpdateReason::MoveCaret);
                if !is_moving_caret_without_updating_the_text {
                    let text_shift = old_absolute_caret_x - new_absolute_caret_offset;

                    // If a text was deleted (caret moved left), adjust the scroll to compensate
                    if text_shift > 0.0 {
                        // Adjust scroll to keep the caret visually in the same position
                        new_scroll.horizontal = (old_scroll.horizontal - text_shift).max(0.0);

                        // Ensure we don't scroll beyond the text boundaries
                        let inner_dimensions = self.inner_size();
                        let area_width = self.outer_size().x;

                        if inner_dimensions.x > area_width {
                            // Text is larger than viewport - clamp scroll to valid range
                            let max_scroll = inner_dimensions.x - area_width + self.caret_width;
                            new_scroll.horizontal = new_scroll.horizontal.min(max_scroll);
                        } else {
                            // Text fits within the viewport - no scroll needed
                            new_scroll.horizontal = 0.0;
                        }
                    }
                }
            } else if new_absolute_caret_offset > max {
                new_scroll.horizontal =
                    new_absolute_caret_offset - text_area_width + self.caret_width;
            } else if new_absolute_caret_offset < min {
                new_scroll.horizontal = new_absolute_caret_offset;
            } else if new_absolute_caret_offset < 0.0 {
                new_scroll.horizontal = 0.0;
            } else {
                // Do nothing?
            }

            self.buffer.set_scroll(new_scroll);
        }

        None
    }

    /// Reshapes the text buffer if parameters have changed since the last reshape.
    ///
    /// This method checks if any text parameters (content, style, size) have changed
    /// and only performs the expensive reshape operation if necessary.
    fn reshape_if_params_changed(&mut self, ctx: &mut TextContext) {
        let params_changed = self.params.changed_since_last_shape();
        if params_changed {
            let new_size = update_buffer(&self.params, &mut self.buffer, &mut ctx.font_system);
            self.inner_dimensions = new_size;
            self.params.reset_changed();
        }
    }

    fn copy_selected_text(&mut self) -> ActionResult {
        let selected_text = self.selected_text().unwrap_or("");
        ActionResult::InsertToClipboard(selected_text.to_string())
    }

    fn paste_text_at_cursor(&mut self, ctx: &mut TextContext, text: &str) -> ActionResult {
        if !text.is_empty() {
            self.reset_selection_end();
        }

        self.params
            .insert_str(self.cursor.byte_character_start, text);
        self.recalculate_with_update_reason(ctx, UpdateReason::InsertedText);
        ActionResult::TextChanged
    }

    fn select_all_recalculate(&mut self, ctx: &mut TextContext) -> ActionResult {
        self.select_all();
        self.recalculate_with_update_reason(ctx, UpdateReason::SelectionChanged);
        ActionResult::CursorUpdated
    }

    fn cut_selected_text(&mut self, ctx: &mut TextContext) -> ActionResult {
        let selected_text = self.selected_text().unwrap_or("").to_string();
        self.remove_selected_text();
        self.recalculate_with_update_reason(ctx, UpdateReason::DeletedTextAtCursor);
        ActionResult::InsertToClipboard(selected_text)
    }

    fn delete_selected_text_or_text_before_cursor(
        &mut self,
        ctx: &mut TextContext,
    ) -> ActionResult {
        if self.is_text_selected() {
            self.remove_selected_text();
        } else {
            self.remove_char_at_cursor();
        }
        self.recalculate_with_update_reason(ctx, UpdateReason::DeletedTextAtCursor);
        ActionResult::TextChanged
    }

    fn move_cursor_right_recalculate(&mut self, ctx: &mut TextContext) -> ActionResult {
        if self.is_text_selected() {
            self.move_cursor_to_selection_right();
        } else {
            self.move_cursor(ctx, Motion::Right);
        }
        self.reset_selection();
        self.recalculate_with_update_reason(ctx, UpdateReason::MoveCaret);
        ActionResult::CursorUpdated
    }

    fn move_cursor_left_recalculate(&mut self, ctx: &mut TextContext) -> ActionResult {
        if self.is_text_selected() {
            self.move_cursor_to_selection_left();
        } else {
            self.move_cursor(ctx, Motion::Left);
        }
        self.reset_selection();
        self.recalculate_with_update_reason(ctx, UpdateReason::MoveCaret);
        ActionResult::CursorUpdated
    }

    fn move_cursor(&mut self, ctx: &mut TextContext, motion: Motion) -> ActionResult {
        let buffer = &mut self.buffer;
        let old_cursor = self.cursor.cursor;
        let mut edit = Editor::new(buffer);
        edit.set_cursor(self.cursor.cursor);
        edit.action(&mut ctx.font_system, cosmic_text::Action::Motion(motion));
        let new_cursor = edit.cursor();
        self.update_cursor_before_glyph_with_cursor(new_cursor);

        if self.cursor.cursor == old_cursor {
            return ActionResult::None;
        }

        ActionResult::CursorUpdated
    }

    fn move_cursor_recalculate(&mut self, ctx: &mut TextContext, motion: Motion) -> ActionResult {
        let res = self.move_cursor(ctx, motion);
        self.reset_selection();
        self.recalculate_with_update_reason(ctx, UpdateReason::MoveCaret);
        res
    }

    fn insert_character(&mut self, character: &SmolStr, ctx: &mut TextContext) -> ActionResult {
        if self.is_text_selected() {
            self.move_cursor(ctx, Motion::Left);
            self.remove_selected_text();
        }
        for character in character.chars() {
            self.insert_char_at_cursor(character, ctx);
            self.reset_selection_end();
        }

        self.recalculate_with_update_reason(ctx, UpdateReason::InsertedText);
        ActionResult::TextChanged
    }

    /// Applies a text editing action and returns the result.
    ///
    /// This is the main method for processing text editing operations like inserting text,
    /// moving the cursor, copying/pasting, etc. The method respects the current text state
    /// configuration (editable, selectable, actions enabled).
    ///
    /// # Arguments
    /// * `ctx` - Mutable reference to the text context for processing
    /// * `action` - The action to apply
    ///
    /// # Returns
    /// An `ActionResult` indicating what happened as a result of the action
    ///
    /// # Examples
    /// ```
    /// # use protextinator::{TextState, TextContext, Action, ActionResult};
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let mut state = TextState::new_with_text("Hello", &mut font_system, ());
    /// # let mut ctx = TextContext::default();
    /// # state.is_editable = true;
    /// # state.are_actions_enabled = true;
    /// let result = state.apply_action(&mut ctx, &Action::InsertChar("x".into()));
    /// match result {
    ///     ActionResult::TextChanged => println!("Text was modified"),
    ///     ActionResult::CursorUpdated => println!("Cursor position changed"),
    ///     _ => {}
    /// }
    /// ```
    pub fn apply_action(&mut self, ctx: &mut TextContext, action: &Action) -> ActionResult {
        if !self.are_actions_enabled {
            return ActionResult::ActionsDisabled;
        }

        if self.is_selectable {
            let res = if self.is_editable {
                match action {
                    Action::Paste(text) => self.paste_text_at_cursor(ctx, text),
                    Action::Cut => self.cut_selected_text(ctx),
                    Action::DeleteBackward => self.delete_selected_text_or_text_before_cursor(ctx),
                    Action::MoveCursorRight => self.move_cursor_right_recalculate(ctx),
                    Action::MoveCursorLeft => self.move_cursor_left_recalculate(ctx),
                    Action::MoveCursorUp => self.move_cursor_recalculate(ctx, Motion::Up),
                    Action::MoveCursorDown => self.move_cursor_recalculate(ctx, Motion::Down),
                    Action::InsertChar(character) => self.insert_character(character, ctx),
                    _ => ActionResult::None,
                }
            } else {
                ActionResult::None
            };

            if res.is_none() {
                match action {
                    Action::CopySelectedText => self.copy_selected_text(),
                    Action::SelectAll => self.select_all_recalculate(ctx),
                    _ => ActionResult::None,
                }
            } else {
                res
            }
        } else {
            ActionResult::None
        }
    }

    // TODO: make it an action
    /// Handles a mouse press event on the text area.
    ///
    /// This method processes mouse clicks for cursor positioning and selection start.
    /// It converts the click position to a character position in the text and updates
    /// the cursor accordingly.
    ///
    /// # Arguments
    /// * `text_context` - Mutable reference to the text context
    /// * `click_position_relative_to_area` - The click position relative to the text area
    ///
    /// # Returns
    /// `Some(())` if the press was handled, `None` otherwise
    ///
    /// # Examples
    /// ```
    /// # use protextinator::{TextState, TextContext, math::Point};
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let mut state = TextState::new_with_text("Hello", &mut font_system, ());
    /// # let mut ctx = TextContext::default();
    /// # state.is_selectable = true;
    /// let click_pos = Point::new(10.0, 5.0);
    /// state.handle_press(&mut ctx, click_pos);
    /// ```
    pub fn handle_press(
        &mut self,
        text_context: &mut TextContext,
        click_position_relative_to_area: Point,
    ) -> Option<()> {
        if self.is_selectable || self.is_editable {
            self.reset_selection();

            let byte_offset_cursor =
                char_under_position(&self.buffer, click_position_relative_to_area)?;
            self.update_cursor_before_glyph_with_cursor(byte_offset_cursor);

            // Reset selection to start at the press location
            self.selection.origin_character_byte_cursor = Some(self.cursor);
            self.selection.ends_before_character_byte_cursor = None;

            self.recalculate_with_update_reason(text_context, UpdateReason::MoveCaret);
        }

        None
    }

    /// Handles mouse drag events for text selection.
    ///
    /// This method processes mouse drag operations to create and update text selections.
    /// It includes automatic scrolling when dragging beyond the visible text area.
    ///
    /// # Arguments
    /// * `ctx` - Mutable reference to the text context
    /// * `is_dragging` - Whether a drag operation is currently in progress
    /// * `pointer_relative_position` - The current pointer position relative to the text area
    ///
    /// # Returns
    /// `Some(())` if the drag was handled, `None` otherwise
    ///
    /// # Examples
    /// ```
    /// # use protextinator::{TextState, TextContext, math::Point};
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let mut state = TextState::new_with_text("Hello", &mut font_system, ());
    /// # let mut ctx = TextContext::default();
    /// # state.is_selectable = true;
    /// let drag_pos = Point::new(50.0, 10.0);
    /// state.handle_drag(&mut ctx, true, drag_pos);
    /// ```
    pub fn handle_drag(
        &mut self,
        ctx: &mut TextContext,
        is_dragging: bool,
        pointer_relative_position: Point,
    ) -> Option<()> {
        if !is_dragging {
            return None;
        }
        if self.is_selectable {
            let byte_cursor_under_position =
                char_under_position(&self.buffer, pointer_relative_position)?;

            if let Some(_origin) = self.selection.origin_character_byte_cursor {
                self.selection.ends_before_character_byte_cursor = ByteCursor::from_cursor(
                    byte_cursor_under_position,
                    self.params.text_for_internal_use(),
                );
            }

            // Simple debounce to make scroll speed consistent
            let now = std::time::Instant::now();
            if now > self.last_scroll_timestamp + self.scroll_interval && is_dragging {
                let element_area = self.params.size();
                let is_dragging_to_the_right = pointer_relative_position.x > 0.0;
                let is_dragging_to_the_left = pointer_relative_position.x < element_area.x;

                if is_dragging_to_the_right || is_dragging_to_the_left {
                    self.update_cursor_before_glyph_with_cursor(byte_cursor_under_position);
                    self.last_scroll_timestamp = now;
                }
            }

            self.recalculate_with_update_reason(ctx, UpdateReason::MoveCaret);
        }

        None
    }

    #[cfg(test)]
    pub fn first_glyph(&mut self) -> Option<&LayoutGlyph> {
        self.buffer
            .layout_runs()
            .next()
            .and_then(|run| run.glyphs.first())
    }
}

/// Takes element height, text buffer height, and vertical alignment and returns the vertical offset
/// needed to align the text vertically.
///
/// This function calculates the appropriate vertical offset for text alignment based on
/// the text area size, buffer dimensions, and vertical alignment settings.
///
/// # Arguments
/// * `text_style` - The text style containing alignment information
/// * `text_area_size` - The size of the text area container
/// * `buffer_inner_dimensions` - The actual dimensions of the text content
///
/// # Returns
/// The vertical offset needed to achieve the desired alignment
pub(crate) fn calculate_vertical_offset(
    text_style: &TextStyle,
    text_area_size: Size,
    buffer_inner_dimensions: Size,
) -> f32 {
    let text_area_rect = Rect::new((0.0, 0.0).into(), text_area_size);
    let style = text_style;

    let vertical_alignment = style.vertical_alignment;
    // TODO: fix scaling
    let buffer_height = buffer_inner_dimensions.y;
    // TODO: FIX TOP.
    let vertical_offset = vertical_offset(vertical_alignment, text_area_rect, buffer_height);

    0.0 - vertical_offset
}

/// Describes the reason for a text state update operation.
///
/// This enum is used internally to optimize recalculation operations by providing
/// context about what type of change triggered the update, allowing for more
/// targeted and efficient updates.
pub enum UpdateReason {
    /// Text content was inserted at the cursor position.
    InsertedText,
    /// The cursor position was moved.
    MoveCaret,
    /// Text was deleted at or around the cursor position.
    DeletedTextAtCursor,
    /// The text selection was modified.
    SelectionChanged,
    /// The reason for the update is unknown or doesn't fit other categories.
    Unknown,
}

impl UpdateReason {
    /// Returns `true` if this update reason indicates a selection change.
    pub fn is_selection_changed(&self) -> bool {
        matches!(self, UpdateReason::SelectionChanged)
    }

    /// Returns `true` if this update reason indicates text was inserted.
    pub fn is_inserted_text(&self) -> bool {
        matches!(self, UpdateReason::InsertedText)
    }

    /// Returns `true` if this update reason indicates the cursor was moved.
    pub fn is_move_caret(&self) -> bool {
        matches!(self, UpdateReason::MoveCaret)
    }

    /// Returns `true` if this update reason indicates text was deleted.
    pub fn is_deleted_text_at_cursor(&self) -> bool {
        matches!(self, UpdateReason::DeletedTextAtCursor)
    }

    /// Returns `true` if this update reason indicates any cursor-related change.
    ///
    /// This includes cursor movement, text insertion, and text deletion operations.
    pub fn is_cursor_updated(&self) -> bool {
        matches!(
            self,
            UpdateReason::MoveCaret
                | UpdateReason::InsertedText
                | UpdateReason::DeletedTextAtCursor
        )
    }
}
