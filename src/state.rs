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
use crate::{Id, Point, Rect};
#[cfg(test)]
use cosmic_text::LayoutGlyph;
use cosmic_text::{Buffer, Cursor, Edit, Editor, FontSystem, Motion};
use smol_str::SmolStr;
use std::time::{Duration, Instant};

pub const SIZE_EPSILON: f32 = 0.0001;

#[derive(Clone, Default, Debug, Copy)]
pub struct SelectionLine {
    pub start_x_pt: Option<f32>,
    pub start_y_pt: Option<f32>,
    pub end_x_pt: Option<f32>,
    pub end_y_pt: Option<f32>,
}

#[derive(Clone, Default, Debug)]
pub struct Selection {
    origin_character_byte_cursor: Option<ByteCursor>,
    ends_before_character_byte_cursor: Option<ByteCursor>,
    lines: Vec<SelectionLine>,
}

impl Selection {
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.origin_character_byte_cursor.is_none()
            || self.ends_before_character_byte_cursor.is_none()
    }

    #[inline(always)]
    pub fn lines(&self) -> &[SelectionLine] {
        &self.lines
    }
}

pub struct TextState {
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
}

impl TextState {
    pub fn new_with_text(
        text: impl Into<String>,
        text_buffer_id: Id,
        font_system: &mut FontSystem,
    ) -> Self {
        let text = text.into();
        let params = TextParams::new(Size::ZERO, TextStyle::default(), text, text_buffer_id);
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
        }
    }

    /// Sets the caret width, which is the width of the cursor when editing text.
    pub fn set_caret_width(&mut self, width: f32) {
        self.caret_width = width;
    }

    /// Returns the caret width, which is the width of the cursor when editing text.
    pub fn caret_width(&self) -> f32 {
        self.caret_width
    }

    /// Caret position relative to the buffer viewport with scroll applied. Returns `None` if
    /// the caret is not visible or the buffer is not shaped yet.
    pub fn caret_position_relative(&self) -> Option<Point> {
        self.relative_caret_position
    }

    /// Returns the position of the selection lines in the buffer viewport.
    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    /// Sets the text in the buffer and updates the cursor position if necessary. Also reshapes
    /// the buffer if the text parameters have changed.
    pub fn set_text_and_reshape(&mut self, text: &str, ctx: &mut TextContext) {
        self.params.set_text(text);

        self.reshape_if_params_changed(ctx, None);

        if self.cursor.byte_character_start > self.params.text_for_internal_use().len() {
            self.move_cursor(ctx, Motion::BufferEnd);
        }
    }

    /// Sets the text in the buffer and updates the cursor position if necessary.
    pub fn set_text(&mut self, text: &str) {
        self.params.set_text(text);

        if self.cursor.byte_character_start > self.params.text_for_internal_use().len() {
            self.update_cursor_before_glyph_with_bytes_offset(
                self.params.text_for_internal_use().len(),
            );
        }
    }

    /// Returns the text in the buffer
    pub fn text(&self) -> &str {
        self.params.original_text()
    }

    /// Sets the text style
    pub fn set_style(&mut self, style: &TextStyle) {
        self.params.set_style(style);
    }

    /// Returns the text style
    pub fn style(&self) -> &TextStyle {
        self.params.style()
    }

    /// Sets the visible are of the text buffer
    pub fn set_outer_size(&mut self, size: &Size) {
        self.params.set_size(size)
    }

    /// Returns the visible area size of the text buffer. Note that this is set directly by the
    /// `set_outer_size` method, and it does not represent the actual text dimensions. To get the
    /// inner dimensions of the text buffer, use `inner_size`.
    pub fn outer_size(&self) -> Size {
        self.params.size()
    }

    /// Returns the inner dimensions of the text buffer. This represents the actual size of the text
    /// content, which may differ from the outer size if the text is larger than the visible area.
    pub fn inner_size(&self) -> Size {
        self.inner_dimensions
    }

    // TODO: right now buffer id is used in grafo rendering to identify the depth - need to fix that
    pub fn set_buffer_id(&mut self, buffer_id: &Id) {
        self.params.set_buffer_id(buffer_id);
    }

    // TODO: right now buffer id is used in grafo rendering to identify the depth - need to fix that
    pub fn buffer_id(&self) -> Id {
        self.params.buffer_id()
    }

    /// Returns the text buffer that can be used for rendering
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Returns the length of the text in characters. Note that this is different from the
    /// string .len(), which returns the length in bytes.
    pub fn text_char_len(&self) -> usize {
        self.params.original_text().chars().count()
    }

    /// Returns the char index of the cursor in the text buffer. Note that this return the
    /// char index, not the char byte index.
    pub fn cursor_char_index(&self) -> Option<usize> {
        self.cursor.char_index(self.params.text_for_internal_use())
    }

    fn insert_char_at_cursor(&mut self, character: char, ctx: &mut TextContext) -> ActionResult {
        self.params
            .insert_char(self.cursor.byte_character_start, character);
        self.reshape_if_params_changed(ctx, None);
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

    // Buffer must be shaped and updated before calling this function
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

    pub fn set_absolute_scroll(&mut self, scroll: Point) {
        let mut new_scroll = self.buffer.scroll();

        new_scroll.horizontal = scroll.x;

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
        let _reshaped = self.params.changed_since_last_shape();
        // TODO: pass cursor if it's not currently visible

        self.reshape_if_params_changed(ctx, None);
        self.recalculate_caret_position_and_scroll(
            self.params.size(),
            update_reason,
            &mut ctx.font_system,
        );
        // TODO: do only if scroll/selection changed
        self.recalculate_selection_area();

        self.relative_caret_position = self.calculate_caret_position();
        self.align_vertically();
    }

    // TODO: Add a method to make other parameters same as `params`, and recalculate lazily
    //  on getters
    /// Recalculates and reshapes the text buffer, scroll, caret position and selection area.
    /// The results are cached, so don't be afraid to call this function multiple times.
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
    fn recalculate_caret_position_and_scroll(
        &mut self,
        text_area_size: Size,
        update_reason: UpdateReason,
        font_system: &mut FontSystem,
    ) -> Option<()> {
        let old_scroll = self.buffer.scroll();

        if update_reason.is_cursor_updated() {
            let caret_position_relative_to_buffer = adjust_vertical_scroll_to_make_caret_visible(
                &mut self.buffer,
                self.cursor,
                font_system,
                self.params.size(),
                self.params.style(),
                self.inner_dimensions,
            )?;
            let mut new_scroll = self.buffer.scroll();

            let current_relative_caret_offset = caret_position_relative_to_buffer.x;

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

            // If caret is within the visible text area, we don't need to scroll.
            //  In that case, we should return the old scroll and modify the caret offset
            if is_new_caret_visible {
                let should_update_horizontal_scroll = self.should_update_horizontal_scroll(
                    text_area_width,
                    current_relative_caret_offset,
                    new_absolute_caret_offset,
                    old_scroll.horizontal,
                );

                let is_moving_caret = matches!(update_reason, UpdateReason::MoveCaret);

                if should_update_horizontal_scroll && !is_moving_caret {
                    new_scroll.horizontal =
                        new_absolute_caret_offset - current_relative_caret_offset;
                } else {
                    new_scroll.horizontal = old_scroll.horizontal;
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

            // self.relative_caret_position = Some(
            //     Point::new(new_relative_caret_offset, caret_position_relative_to_buffer.y),
            // );
            self.buffer.set_scroll(new_scroll);
        }

        None
    }

    /// Determines if we should use improved scroll behavior where the caret stays visually
    /// fixed while deleting overflowing text, instead of moving the caret within the visible area.
    ///
    /// This behavior is used when:
    /// 1. Text overflows the visible area (text is longer than area width)
    /// 2. We're likely deleting from the end (caret moved to the left)
    /// 3. There's horizontal scroll present
    fn should_update_horizontal_scroll(
        &self,
        text_area_width: f32,
        old_relative_caret_x: f32,
        new_absolute_caret_x: f32,
        current_scroll_x: f32,
    ) -> bool {
        // Only apply improved behavior when there's existing scroll
        if current_scroll_x <= 0.0 {
            return false;
        }

        // Calculate approximate text width based on buffer content
        let text_overflows = self.estimate_text_overflows(text_area_width);
        if !text_overflows {
            return false;
        }

        // Check if caret moved to the left (likely deletion from end)
        let old_absolute_caret_x = old_relative_caret_x + current_scroll_x;

        // Use improved behavior when text overflows and caret moved left
        new_absolute_caret_x < old_absolute_caret_x
    }

    /// Estimates if a text overflows the given width by examining the buffer's layout
    fn estimate_text_overflows(&self, text_area_width: f32) -> bool {
        // TODO: check if it's better done with inner_dimensions instead of trying to figure out width
        // Look at the last glyph position to estimate if text overflows
        if let Some(line) = &self.buffer.lines.last() {
            if let Some(layouts) = line.layout_opt().as_ref() {
                if let Some(layout) = layouts.last() {
                    if let Some(last_glyph) = layout.glyphs.last() {
                        let text_width = last_glyph.x + last_glyph.w;
                        return text_width > text_area_width;
                    }
                }
            }
        }

        // Fallback: assume no overflow if we can't determine
        false
    }

    fn reshape_if_params_changed(
        &mut self,
        ctx: &mut TextContext,
        shape_till_cursor: Option<Cursor>,
    ) {
        let params_changed = self.params.changed_since_last_shape();
        if params_changed {
            let new_size = update_buffer(
                &self.params,
                &mut self.buffer,
                &mut ctx.font_system,
                shape_till_cursor,
            );
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

/// Takes element height, text buffer height and vertical alignment and returns the vertical offset
///  needed to align the text vertically.
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

pub enum UpdateReason {
    // Cursor changed
    InsertedText,
    MoveCaret,
    DeletedTextAtCursor,
    // Selection changed
    SelectionChanged,
    // Unknown reason, can be used for anything that doesn't fit the above
    Unknown,
}

impl UpdateReason {
    pub fn is_selection_changed(&self) -> bool {
        matches!(self, UpdateReason::SelectionChanged)
    }

    pub fn is_inserted_text(&self) -> bool {
        matches!(self, UpdateReason::InsertedText)
    }

    pub fn is_move_caret(&self) -> bool {
        matches!(self, UpdateReason::MoveCaret)
    }

    pub fn is_deleted_text_at_cursor(&self) -> bool {
        matches!(self, UpdateReason::DeletedTextAtCursor)
    }

    pub fn is_cursor_updated(&self) -> bool {
        matches!(
            self,
            UpdateReason::MoveCaret
                | UpdateReason::InsertedText
                | UpdateReason::DeletedTextAtCursor
        )
    }
}
