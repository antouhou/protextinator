//! The `TextState` type: text content, cursor, selection, scrolling, and editing.

use crate::action::{Action, ActionResult};
use crate::buffer_utils::{
    adjust_vertical_scroll_to_make_caret_visible, char_under_position,
    cursor_position_with_trailing_space_fallback, update_buffer, vertical_offset,
};
use crate::byte_cursor::ByteCursor;
use crate::math::Size;
use crate::style::{FontFamily, TextStyle, VerticalTextAlignment};
use crate::text_manager::TextContext;
use crate::text_params::TextParams;
use crate::utils::{linear_to_srgb_u8, srgb_to_linear_u8};
use crate::{Point, Rect};
#[cfg(test)]
use cosmic_text::LayoutGlyph;
use cosmic_text::{Buffer, Cursor, Edit, Editor, FontSystem, Motion};
use smol_str::SmolStr;
use std::time::{Duration, Instant};

/// Size comparison epsilon for floating-point calculations.
pub const SIZE_EPSILON: f32 = 0.0001;

/// CPU-side RGBA8 texture holding the rasterized contents of a text buffer.
#[derive(Debug, Clone)]
pub struct RasterizedTexture {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Start and end coordinates of a selection on a single line, for rendering
/// selection highlights.
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

/// The current text selection.
///
/// Defined by an origin (where the selection started) and an end cursor. A selection
/// can span multiple lines; each line's bounds are stored in the `lines` vector.
#[derive(Clone, Default, Debug)]
pub struct Selection {
    origin_character_byte_cursor: Option<ByteCursor>,
    ends_before_character_byte_cursor: Option<ByteCursor>,
    lines: Vec<SelectionLine>,
}

impl Selection {
    /// Returns `true` if there is no active selection.
    ///
    /// A selection is empty if either the origin or the end cursor is not set.
    ///
    /// # Examples
    /// ```
    /// use protextinator::Selection;
    ///
    /// let selection = Selection::default();
    /// assert!(selection.is_empty());
    /// ```
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.origin_character_byte_cursor.is_none()
            || self.ends_before_character_byte_cursor.is_none()
    }

    /// Returns the selection bounds for each line, for rendering highlights.
    #[inline(always)]
    pub fn lines(&self) -> &[SelectionLine] {
        &self.lines
    }
}

/// The core type for text editing: text buffer, cursor, selection, scroll position,
/// and styling.
///
/// # Type Parameters
/// * `T` - Custom metadata type that can be attached to the text state
#[derive(Debug)]
pub struct TextState<T> {
    params: TextParams,
    cursor: ByteCursor,
    // Caret position relative to the buffer viewport with scroll applied
    relative_caret_position: Option<Point>,
    caret_width: f32,
    selection: Selection,
    resolved_font_family: FontFamily,

    last_scroll_timestamp: Instant,

    inner_dimensions: Size,
    buffer: Buffer,

    // CPU-side cached rasterized texture of the current buffer (RGBA8, device pixels)
    rasterized_texture: RasterizedTexture,
    // Whether raster content needs to be regenerated
    raster_dirty: bool,

    // Settings
    /// Can text be selected?
    pub is_selectable: bool,
    /// Can text be edited?
    pub is_editable: bool,
    /// Whether an editing session is in progress. Not read by the library itself;
    /// it is for your own bookkeeping.
    pub is_editing: bool,
    /// Are actions enabled? If false, no actions will be performed.
    pub are_actions_enabled: bool,
    /// Interval between scroll updates when dragging the selection
    pub scroll_interval: Duration,

    /// Free-form metadata for your own use during rendering. The library ignores it.
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

            resolved_font_family: FontFamily::SansSerif,

            selection: Selection::default(),
            last_scroll_timestamp: Instant::now(),
            scroll_interval: Duration::from_millis(50),
            caret_width: 3.0,
            is_selectable: false,
            is_editable: false,

            inner_dimensions: Size::ZERO,
            buffer: Buffer::new(font_system, metrics),

            rasterized_texture: RasterizedTexture {
                pixels: Vec::new(),
                width: 0,
                height: 0,
            },
            raster_dirty: true,

            metadata,
        }
    }

    /// Sets the caret width in pixels.
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

    /// Returns the caret width in pixels.
    ///
    /// # Examples
    /// ```
    /// # use protextinator::TextState;
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let state = TextState::new_with_text("", &mut font_system, ());
    /// let width = state.caret_width();
    /// ```
    pub const fn caret_width(&self) -> f32 {
        self.caret_width
    }

    /// Caret position relative to the buffer viewport with scroll applied. Returns `None` if
    /// the caret is not visible or the buffer is not shaped yet.
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

    /// Returns the current selection.
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
    /// Does not reshape. Call [`Self::recalculate`] to update the layout.
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

        // TODO: should we just reset cursor on whole text update?
        if self.cursor.byte_character_start > self.params.text_for_internal_use().len() {
            if text.is_empty() {
                self.cursor = ByteCursor::default()
            } else {
                self.update_cursor_before_glyph_with_bytes_offset(
                    self.params.text_for_internal_use().len(),
                );
            }
        }
    }

    /// Returns the text in the buffer
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

    /// Returns the resolved font family after font matching.
    ///
    /// May differ from the style's font family if font substitution occurred.
    ///
    /// # Examples
    /// ```
    /// # use protextinator::TextState;
    /// # use cosmic_text::FontSystem;
    /// # let mut font_system = FontSystem::new();
    /// # let state = TextState::new_with_text("", &mut font_system, ());
    /// let resolved = state.resolved_font_family();
    /// ```
    pub fn resolved_font_family(&self) -> &FontFamily {
        &self.resolved_font_family
    }

    /// Sets the visible area of the text buffer, i.e. the viewport size. It determines
    /// how much text is visible.
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

    /// Returns the metadata set on the cosmic_text buffer
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

    /// Sets the metadata on the cosmic_text buffer. This is different from the `metadata`
    /// field on `TextState`, which holds your custom type.
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

    /// Returns the visible area size of the text buffer, as set by [`Self::set_outer_size`].
    /// This is not the actual text size; use [`Self::inner_size`] for that.
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

    /// Returns the actual size of the text content. May differ from [`Self::outer_size`]
    /// if the text is larger than the visible area.
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
    pub const fn inner_size(&self) -> Size {
        self.inner_dimensions
    }

    /// Returns the underlying cosmic-text [`Buffer`], for rendering
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

    /// Returns the last rasterized CPU texture.
    pub fn rasterized_texture(&self) -> &RasterizedTexture {
        &self.rasterized_texture
    }

    /// Returns the length of the text in characters, unlike `str::len` which counts bytes.
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

    /// Returns the character index of the cursor in the text. This is a character index,
    /// not a byte index.
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
        let text = self.params.text_for_internal_use();
        let at_end = self.cursor.byte_character_start >= text.len();
        let ends_with_newline = text.ends_with('\n');

        // cosmic_text quirk: when inserting a newline at the end of text that doesn't
        // already end with a newline, we need to insert two newlines so the caret can
        // be placed on the new line
        if character == '\n' && at_end && !ends_with_newline {
            self.params
                .insert_char(self.cursor.byte_character_start, '\n');
            self.params
                .insert_char(self.cursor.byte_character_start + 1, '\n');
        } else {
            self.params
                .insert_char(self.cursor.byte_character_start, character);
        }

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

    /// Returns `true` if any text is selected.
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

    /// Returns the selected text, or `None` if nothing is selected.
    ///
    /// To copy the selection to the clipboard instead, use [`TextState::apply_action`]
    /// with [`Action::CopySelectedText`].
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

    /// Returns the absolute scroll offset of the text content, both horizontal
    /// and vertical.
    ///
    /// Call [`TextState::recalculate`] first if the text changed, otherwise the
    /// result may be stale.
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
        let scale = self.params.scale_factor().max(0.01);
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
                    line_vertical_start +=
                        layout_line.line_height_opt.unwrap_or(line_height * scale);
                }
            }
        }
        // Convert to LOGICAL pixels
        Point {
            x: scroll_horizontal / scale,
            y: (scroll_vertical + line_vertical_start) / scale,
        }
    }

    /// Sets the absolute scroll position of the text buffer.
    ///
    /// Scroll only takes effect when vertical alignment is `VerticalTextAlignment::None`.
    /// With a fixed alignment (e.g. `VerticalTextAlignment::Top`), the offset comes from
    /// the layout and this method has no visible effect.
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
        let scale = self.params.scale_factor().max(0.01);

        let can_scroll_vertically =
            matches!(self.style().vertical_alignment, VerticalTextAlignment::None);

        // Horizontal scroll is stored in DEVICE pixels
        new_scroll.horizontal = scroll.x * scale;

        if can_scroll_vertically {
            let line_height = self.style().line_height_pt();
            let mut line_index = 0;
            let mut accumulated_height_device = 0.0;
            let target_y_device = scroll.y * scale;

            for (i, line) in self.buffer.lines.iter().enumerate() {
                let mut line_height_total_device = 0.0;

                if let Some(layout_lines) = line.layout_opt() {
                    for layout_line in layout_lines {
                        line_height_total_device +=
                            layout_line.line_height_opt.unwrap_or(line_height * scale);
                    }
                }

                if accumulated_height_device + line_height_total_device > target_y_device {
                    line_index = i;
                    break;
                }

                accumulated_height_device += line_height_total_device;
                line_index = i + 1; // In case we don't break, this will be the last line
            }

            // Set the line and calculate the remaining vertical offset (device px)
            new_scroll.line = line_index;
            new_scroll.vertical = target_y_device - accumulated_height_device;
        }

        // Apply only if changed
        let old = self.buffer.scroll();
        if (old.horizontal - new_scroll.horizontal).abs() > SIZE_EPSILON
            || (old.vertical - new_scroll.vertical).abs() > SIZE_EPSILON
            || old.line != new_scroll.line
        {
            self.buffer.set_scroll(new_scroll);
            // Any scroll change requires re-rasterization
            self.raster_dirty = true;
        }
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
            for (start_x, width) in run.highlight(start_cursor.cursor, end_cursor.cursor) {
                let scale = self.params.scale_factor().max(0.01);
                self.selection.lines.push(SelectionLine {
                    // Convert to LOGICAL pixels
                    start_x_pt: Some((start_x - self.buffer.scroll().horizontal) / scale),
                    end_x_pt: Some((start_x + width - self.buffer.scroll().horizontal) / scale),
                    start_y_pt: Some(run.line_top / scale),
                    end_y_pt: Some((run.line_top + run.line_height) / scale),
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
    /// Results are cached, so calling it repeatedly is cheap.
    ///
    /// Call this after changing text content, style, or size.
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
        // Return caret position in LOGICAL pixels relative to viewport
        let horizontal_scroll_device = self.buffer.scroll().horizontal;
        let scale = self.params.scale_factor().max(0.01);
        cursor_position_with_trailing_space_fallback(&mut self.buffer, self.cursor).map(
            |mut point_device| {
                // pos from cosmic_text is in DEVICE pixels
                // Adjust by horizontal scroll (device px)
                point_device.x -= horizontal_scroll_device;
                // Convert to logical
                Point::new(point_device.x / scale, point_device.y / scale)
            },
        )
    }

    fn align_vertically(&mut self) {
        if matches!(self.style().vertical_alignment, VerticalTextAlignment::None) {
            return;
        }

        let mut scroll = self.buffer.scroll();
        let text_area_size = self.params.size();
        let vertical_scroll_to_align_text_logical =
            calculate_vertical_offset(self.params.style(), text_area_size, self.inner_dimensions);
        let scale = self.params.scale_factor().max(0.01);
        let target_vertical_device = vertical_scroll_to_align_text_logical * scale;
        if (scroll.vertical - target_vertical_device).abs() > SIZE_EPSILON {
            scroll.vertical = target_vertical_device;
            self.buffer.set_scroll(scroll);
            // Vertical alignment scroll change affects raster
            self.raster_dirty = true;
        }
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
            let scale = self.params.scale_factor().max(0.01);
            let old_scroll = self.buffer.scroll();
            let old_relative_caret_x_logical = self.relative_caret_position.map_or(0.0, |p| p.x);
            // Convert old absolute caret to logical coords
            let old_absolute_caret_x_logical =
                old_relative_caret_x_logical + old_scroll.horizontal / scale;

            let caret_position_relative_to_buffer = adjust_vertical_scroll_to_make_caret_visible(
                &mut self.buffer,
                self.cursor,
                font_system,
                self.params.size(),
                self.params.style(),
                scale,
            )?;
            let mut new_scroll = self.buffer.scroll();
            let text_area_width = text_area_size.x;

            // TODO: there was some other implementation that took horizontal alignment into account,
            //  check if it is needed
            let new_absolute_caret_offset = caret_position_relative_to_buffer.x; // logical

            // TODO: A little hack to set horizontal scroll
            let current_absolute_visible_text_area = (
                old_scroll.horizontal / scale,
                old_scroll.horizontal / scale + text_area_width,
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
                    let text_shift_logical =
                        old_absolute_caret_x_logical - new_absolute_caret_offset;

                    // If a text was deleted (caret moved left), adjust the scroll to compensate
                    if text_shift_logical > 0.0 {
                        // Adjust scroll to keep the caret visually in the same position
                        new_scroll.horizontal =
                            (old_scroll.horizontal - text_shift_logical * scale).max(0.0);

                        // Ensure we don't scroll beyond the text boundaries
                        let inner_dimensions = self.inner_size();
                        let area_width = self.outer_size().x;

                        if inner_dimensions.x > area_width {
                            // Text is larger than viewport - clamp scroll to valid range
                            let max_scroll_device =
                                (inner_dimensions.x - area_width + self.caret_width) * scale;
                            new_scroll.horizontal = new_scroll.horizontal.min(max_scroll_device);
                        } else {
                            // Text fits within the viewport - no scroll needed
                            new_scroll.horizontal = 0.0;
                        }
                    }
                }
            } else if new_absolute_caret_offset > max {
                new_scroll.horizontal =
                    (new_absolute_caret_offset - text_area_width + self.caret_width) * scale;
            } else if new_absolute_caret_offset < min {
                new_scroll.horizontal = new_absolute_caret_offset * scale;
            } else if new_absolute_caret_offset < 0.0 {
                new_scroll.horizontal = 0.0;
            } else {
                // Do nothing?
            }

            // Apply only if changed
            let old = self.buffer.scroll();
            if (old.horizontal - new_scroll.horizontal).abs() > SIZE_EPSILON
                || (old.vertical - new_scroll.vertical).abs() > SIZE_EPSILON
                || old.line != new_scroll.line
            {
                self.buffer.set_scroll(new_scroll);
                // Scroll changes affect raster
                self.raster_dirty = true;
            }
        }

        None
    }

    /// Reshapes the text buffer, but only if the text, style, or size changed since
    /// the last reshape. Reshaping is expensive, so it is skipped when nothing changed.
    fn reshape_if_params_changed(&mut self, ctx: &mut TextContext) {
        let font_query_changed = self.params.font_query_changed_since_last_shape();
        if font_query_changed {
            let new_font_family = ctx.font_family_cache.resolve_font_family_query(
                self.params.style().font_family_query(),
                &mut ctx.font_system,
            );
            self.resolved_font_family = new_font_family;
            self.params.reset_font_query_changed();
        }
        let params_changed = self.params.changed_since_last_shape();
        if params_changed {
            let new_size = update_buffer(
                &self.params,
                &mut self.buffer,
                &mut ctx.font_system,
                &self.resolved_font_family,
            );
            self.inner_dimensions = new_size;
            self.params.reset_changed();
            // Any layout/text/style/size change requires re-rasterization
            self.raster_dirty = true;
        }
    }

    /// Rasterizes the current text buffer into an RGBA8 CPU texture using device-pixel dimensions.
    ///
    /// Returns `true` if the texture was updated, `false` if rasterization was skipped
    /// (e.g. zero-sized target).
    pub(crate) fn rasterize_into_texture(
        &mut self,
        ctx: &mut TextContext,
        alpha_mode: AlphaMode,
    ) -> bool {
        // Compute device-pixel texture size from the logical outer size and scale factor
        let size = self.outer_size();
        let scale = ctx.scale_factor.max(0.01);
        let width = (size.x * scale).ceil().max(0.0) as u32;
        let height = (size.y * scale).ceil().max(0.0) as u32;
        if width == 0 || height == 0 {
            // No room to rasterize; clear texture and mark clean
            self.rasterized_texture.width = 0;
            self.rasterized_texture.height = 0;
            self.rasterized_texture.pixels.clear();
            self.raster_dirty = false;
            return false;
        }

        let dims_changed =
            self.rasterized_texture.width != width || self.rasterized_texture.height != height;

        // Skip if nothing changed and dimensions match
        if !dims_changed && !self.raster_dirty {
            return false;
        }

        let required_len = width as usize * height as usize * 4;
        // Ensure capacity and set length; reuse allocation when possible
        if self.rasterized_texture.pixels.len() != required_len {
            self.rasterized_texture.pixels.resize(required_len, 0);
        }

        // Clear to transparent before drawing (fast fill)
        self.rasterized_texture.pixels.fill(0);

        let base_color = cosmic_text::Color::rgba(0, 0, 0, 0);
        let requested_font_alpha = self.params.style().font_color.0.a();
        let text_width = width;
        let text_height = height;
        let horizontal_scroll_device = self.buffer.scroll().horizontal.round() as i64;
        // TODO: make an atlas via an adapter trait or something that can be passed to here from the renderer
        self.buffer.draw(
            &mut ctx.font_system,
            &mut ctx.swash_cache,
            base_color,
            |x, y, mut w, mut h, color| {
                if w == 0 || h == 0 {
                    return;
                }

                // Use signed clipping first because scrolled glyphs can produce negative device
                // coordinates. Casting negatives to unsigned would incorrectly wrap and skip
                // visible glyph portions near the viewport edge.
                // Cosmic-text horizontal scroll is not reflected in draw callback coordinates,
                // so apply it explicitly here to keep rasterized output in sync with the buffer.
                let mut x_device = x as i64 - horizontal_scroll_device;
                let mut y_device = y as i64;
                let mut width_device = w as i64;
                let mut height_device = h as i64;

                if x_device < 0 {
                    let cut = -x_device;
                    if cut >= width_device {
                        return;
                    }
                    x_device = 0;
                    width_device -= cut;
                }
                if y_device < 0 {
                    let cut = -y_device;
                    if cut >= height_device {
                        return;
                    }
                    y_device = 0;
                    height_device -= cut;
                }

                if x_device >= text_width as i64 || y_device >= text_height as i64 {
                    return;
                }
                let max_width = text_width as i64 - x_device;
                let max_height = text_height as i64 - y_device;
                width_device = width_device.min(max_width);
                height_device = height_device.min(max_height);

                if width_device <= 0 || height_device <= 0 {
                    return;
                }

                let x0 = x_device as u32;
                let y0 = y_device as u32;
                w = width_device as u32;
                h = height_device as u32;

                // Precompute the 4-byte pixel once per rectangle and use row-wise fills
                let mut packed_px = [0u8; 4];
                // IMPORTANT: cosmic-text's mask glyph path replaces alpha with glyph coverage and does not
                // apply the requested font alpha. Reapply the style alpha here so semi-transparent
                // text survives rasterization.
                let effective_alpha =
                    ((u16::from(color.a()) * u16::from(requested_font_alpha) + 127) / 255) as u8;
                match alpha_mode {
                    AlphaMode::Premultiplied => {
                        let r_lin = srgb_to_linear_u8(color.r());
                        let g_lin = srgb_to_linear_u8(color.g());
                        let b_lin = srgb_to_linear_u8(color.b());
                        let a = effective_alpha as f32 / 255.0;
                        let r_pma = r_lin * a;
                        let g_pma = g_lin * a;
                        let b_pma = b_lin * a;
                        packed_px[0] = linear_to_srgb_u8(r_pma);
                        packed_px[1] = linear_to_srgb_u8(g_pma);
                        packed_px[2] = linear_to_srgb_u8(b_pma);
                        packed_px[3] = effective_alpha;
                    }
                    AlphaMode::Unmultiplied => {
                        packed_px[0] = color.r();
                        packed_px[1] = color.g();
                        packed_px[2] = color.b();
                        packed_px[3] = effective_alpha;
                    }
                }

                // Fill each destination row with the precomputed pixel
                for row in 0..h {
                    let dst_row_start = ((y0 + row) * text_width * 4 + x0 * 4) as usize;
                    let row_slice = &mut self.rasterized_texture.pixels
                        [dst_row_start..dst_row_start + (w as usize) * 4];

                    // Repeat-copy packed_px across the row
                    // Avoid per-pixel math; just copy the 4-byte pattern
                    let mut i = 0usize;
                    while i + 4 <= row_slice.len() {
                        row_slice[i..i + 4].copy_from_slice(&packed_px);
                        i += 4;
                    }
                }
            },
        );

        // Update texture dimensions and clear dirty flag
        self.rasterized_texture.width = width;
        self.rasterized_texture.height = height;
        self.raster_dirty = false;

        true
    }

    /// Updates the internal scale factor in params; will trigger reshape on next recalc if changed.
    pub fn set_scale_factor(&mut self, scale: f32) {
        self.params.set_scale_factor(scale);
    }

    fn copy_selected_text(&mut self) -> ActionResult {
        let selected_text = self.selected_text().unwrap_or("");
        ActionResult::TextCopied(selected_text.to_string())
    }

    fn paste_text_at_cursor(&mut self, ctx: &mut TextContext, text: &str) -> ActionResult {
        if self.is_text_selected() {
            self.move_cursor(ctx, Motion::Left);
            self.remove_selected_text();
        }
        if !text.is_empty() {
            let insert_byte_offset = self.cursor.byte_character_start;
            self.params.insert_str(insert_byte_offset, text);
            self.update_cursor_before_glyph_with_bytes_offset(insert_byte_offset + text.len());
        }
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
        ActionResult::TextCut(selected_text)
    }

    fn cut_text_range(&mut self, ctx: &mut TextContext, start: usize, end: usize) -> ActionResult {
        let (byte_start, byte_end) = {
            let text = self.params.original_text();
            let text_length = text.chars().count();
            let start = start.min(text_length);
            let end = end.min(text_length);
            if start >= end {
                return ActionResult::None;
            }
            let char_to_byte_offset = |char_index: usize| {
                text.char_indices()
                    .nth(char_index)
                    .map(|(byte_offset, _)| byte_offset)
                    .unwrap_or(text.len())
            };
            (char_to_byte_offset(start), char_to_byte_offset(end))
        };
        let cut_text = self.params.original_text()[byte_start..byte_end].to_owned();
        self.remove_characters(byte_start, byte_end);
        self.reset_selection();
        self.update_cursor_before_glyph_with_bytes_offset(byte_start);
        self.recalculate_with_update_reason(ctx, UpdateReason::DeletedTextAtCursor);
        ActionResult::TextCut(cut_text)
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
    /// Respects the state's `is_editable`, `is_selectable`, and `are_actions_enabled`
    /// flags.
    ///
    /// # Arguments
    /// * `ctx` - Mutable reference to the text context for processing
    /// * `action` - The action to apply
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

        if self.is_editable {
            if let Action::CutRange { start, end } = action {
                return self.cut_text_range(ctx, *start, *end);
            }
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
    /// Handles a mouse press on the text area: positions the cursor at the clicked
    /// character and starts a new selection there.
    ///
    /// # Arguments
    /// * `text_context` - Mutable reference to the text context
    /// * `click_position_relative_to_area` - The click position relative to the text area
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

            let byte_offset_cursor = char_under_position(
                &self.buffer,
                click_position_relative_to_area,
                self.params.scale_factor(),
            )?;
            self.update_cursor_before_glyph_with_cursor(byte_offset_cursor);

            // Reset selection to start at the press location
            self.selection.origin_character_byte_cursor = Some(self.cursor);
            self.selection.ends_before_character_byte_cursor = None;

            self.recalculate_with_update_reason(text_context, UpdateReason::MoveCaret);
        }

        None
    }

    /// Handles mouse drags to create and update the text selection. Scrolls
    /// automatically when the pointer moves beyond the visible text area.
    ///
    /// # Arguments
    /// * `ctx` - Mutable reference to the text context
    /// * `is_dragging` - Whether a drag operation is currently in progress
    /// * `pointer_relative_position` - The current pointer position relative to the text area
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
            let byte_cursor_under_position = char_under_position(
                &self.buffer,
                pointer_relative_position,
                self.params.scale_factor(),
            )?;

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

/// Returns the vertical offset that aligns the text within the text area, based on
/// the text area size, buffer dimensions, and vertical alignment.
///
/// # Arguments
/// * `text_style` - The text style containing alignment information
/// * `text_area_size` - The size of the text area container
/// * `buffer_inner_dimensions` - The actual dimensions of the text content
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

/// The reason for a text state update. Used internally to skip unnecessary work
/// during recalculation.
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

    /// Returns `true` if this update reason involves the cursor.
    ///
    /// Covers cursor movement, text insertion, and text deletion.
    pub fn is_cursor_updated(&self) -> bool {
        matches!(
            self,
            UpdateReason::MoveCaret
                | UpdateReason::InsertedText
                | UpdateReason::DeletedTextAtCursor
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub enum AlphaMode {
    /// Use premultiplied alpha for rendering. Preferred when blending with other
    /// premultiplied content.
    Premultiplied,
    /// Use unmultiplied alpha for rendering. Needed when compositing with
    /// non-premultiplied content, but can produce artifacts and is less efficient.
    Unmultiplied,
}
