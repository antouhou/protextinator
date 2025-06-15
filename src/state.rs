use crate::action::{Action, ActionResult};
use crate::byte_cursor::ByteCursor;
use crate::text_manager::TextContext;
use crate::math::Size;
use crate::style::TextStyle;
use crate::buffer_cache::{buffer_height, calculate_caret_position_pt, vertical_offset};
use crate::{Id, Point, Rect, TextParams};
use cosmic_text::{Buffer, Cursor, Edit, Editor, FontSystem, Motion, Scroll};
use smol_str::SmolStr;
use std::time::{Duration, Instant};

pub const SIZE_EPSILON: f32 = 0.0001;

#[derive(Clone, Default, Debug, Copy)]
pub struct SelectionLine {
    pub start_pt: Option<f32>,
    pub end_pt: Option<f32>,
    pub line_index: Option<usize>,
}

#[derive(Clone, Default, Debug)]
pub struct Selection {
    pub origin_character_byte_cursor: Option<ByteCursor>,
    pub ends_before_character_byte_cursor: Option<ByteCursor>,
    pub lines: Vec<SelectionLine>,
}

pub struct TextState {
    pub params: TextParams,

    pub cursor: ByteCursor,

    pub relative_caret_offset_horizontal: f32,
    pub relative_caret_offset_vertical: f32,

    pub caret_width: f32,
    /// The horizontal offset of the text inside the buffer. It is needed since horizontal scrolling
    ///  in cosmic_text does not seem to work.
    pub scroll: Scroll,
    pub selection: Selection,

    // Settings
    /// Can text be selected?
    pub is_selectable: bool,
    /// Can text be edited?
    pub is_editable: bool,
    /// Various actions, such as copy, paste, cut, etc., are going to be performed
    pub is_editing: bool,
    pub are_actions_enabled: bool,

    pub last_scroll_timestamp: Instant,
    pub scroll_interval: Duration,
}

impl TextState {
    pub fn new_with_text(text: impl Into<String>, text_buffer_id: Id) -> Self {
        let text = text.into();

        Self {
            params: TextParams::new(Size::default(), TextStyle::default(), text, text_buffer_id),

            is_editing: false,
            are_actions_enabled: false,

            cursor: ByteCursor::default(),

            relative_caret_offset_horizontal: 0.0,
            relative_caret_offset_vertical: 0.0,
            scroll: Scroll::new(0, 0.0, 0.0),
            selection: Selection::default(),
            last_scroll_timestamp: Instant::now(),
            scroll_interval: Duration::from_millis(50),
            caret_width: 3.0,
            is_selectable: false,
            is_editable: false,
        }
    }

    pub fn set_caret_width(&mut self, width: f32) {
        self.caret_width = width;
    }

    pub fn set_text(&mut self, text: &str, ctx: &mut TextContext) {
        self.params.set_text(text);

        self.reshape_if_params_changed(ctx);

        if self.cursor.byte_character_start > self.params.text().len() {
            self.move_cursor(ctx, Motion::BufferEnd);
        }
    }

    pub fn text(&self) -> &str {
        self.params.text()
    }

    pub fn text_size(&self) -> usize {
        self.params.text().chars().count()
    }

    pub fn insert_char_at_cursor(
        &mut self,
        character: char,
        ctx: &mut TextContext,
    ) -> ActionResult {
        self.params
            .insert_char(self.cursor.byte_character_start, character);
        self.reshape_if_params_changed(ctx);
        self.move_cursor(ctx, Motion::Next);

        ActionResult::TextChanged
    }

    pub fn insert_text_at_cursor(&mut self, text: &str) {
        self.params
            .insert_str(self.cursor.byte_character_start, text);
        self.update_cursor_before_glyph_with_bytes_offset(
            self.cursor.byte_character_start + text.len(),
        );
    }

    pub fn remove_char_at_cursor(&mut self) {
        if !self.params.text().is_empty() {
            if let Some(prev_char) = self.cursor.prev_char_byte_offset(self.params.text()) {
                self.remove_character(prev_char);
                self.cursor
                    .update_byte_offset(prev_char, self.params.text());
            }
        }
    }

    pub fn remove_characters(&mut self, byte_offset_start: usize, byte_offset_end: usize) {
        self.params.remove_range(byte_offset_start, byte_offset_end);
    }

    pub fn set_cursor_before_glyph(&mut self, cursor: ByteCursor) {
        self.cursor = cursor;
    }

    pub fn update_cursor_before_glyph_with_cursor(&mut self, cursor: Cursor) {
        self.cursor.update_cursor(cursor, self.params.text());
    }

    pub fn update_cursor_before_glyph_with_bytes_offset(&mut self, byte_offset: usize) {
        self.cursor
            .update_byte_offset(byte_offset, self.params.text());
    }

    pub fn remove_character(&mut self, byte_offset: usize) -> Option<char> {
        self.params.remove_char(byte_offset)
    }

    pub fn remove_selected_text(&mut self) -> Option<()> {
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

    pub fn move_cursor_to_selection_left(&mut self) {
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

    pub fn move_cursor_to_selection_right(&mut self) {
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

    pub fn reset_selection_end(&mut self) {
        self.selection.ends_before_character_byte_cursor = None;
        self.selection.lines.clear();
    }

    pub fn reset_selection(&mut self) {
        self.selection.origin_character_byte_cursor = None;
        self.selection.ends_before_character_byte_cursor = None;
        self.selection.lines.clear();
    }

    pub fn select_all(&mut self) {
        self.selection.origin_character_byte_cursor = Some(ByteCursor::string_start());
        if !self.params.text().is_empty() {
            self.selection.ends_before_character_byte_cursor =
                Some(ByteCursor::after_last_character(self.params.text()))
        } else {
            self.selection.ends_before_character_byte_cursor = None;
        }
    }

    pub fn substring_byte_offset(&self, start: usize, end: usize) -> String {
        // TODO: add bounds checking
        self.params.text()[start..end].to_string()
    }

    pub fn selected_text(&self) -> Option<String> {
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

    pub fn shape_if_not_shaped(&self, ctx: &mut TextContext, reshape: bool) {
        let font_system = &mut ctx.font_system;
        ctx.buffer_cache
            .create_and_shape_text_if_not_in_cache(&self.params, font_system, reshape);
    }

    /// Calculates physical selection area based on the selection start and end glyph indices
    fn recalculate_selection_area(
        &mut self,
        buffer: &mut Buffer,
        font_system: &mut FontSystem,
    ) -> Option<(f32, f32)> {
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

        // TODO: fix that
        // let selection_end_char_index = if selection_ends_before_char_index > 0 {
        //     if selection_starts_at_index == selection_ends_before_char_index {
        //         selection_starts_at_index -= 1;
        //     }
        //     selection_ends_before_char_index - 1
        // } else {
        //     0
        // };

        // let start_cursor = char_index_to_layout_cursor(
        //     buffer,
        //     font_system,
        //     &self.text,
        //     selection_starts_at_index,
        // )?;
        let start_cursor = selection_starts_at_index.layout_cursor(buffer, font_system)?;
        // let end_cursor =
        //     char_index_to_layout_cursor(buffer, font_system, &self.text, selection_end_char_index)?;
        let end_cursor = selection_ends_before_char_index.layout_cursor(buffer, font_system)?;

        self.selection.lines.clear();

        let horizontal_scroll = self.scroll.horizontal;
        let mut lines_counted: usize = 0;

        for (i, line) in buffer.lines.iter().enumerate() {
            if i < start_cursor.line {
                let layouts_count = line
                    .layout_opt()
                    .as_ref()
                    .map(|layouts| layouts.len())
                    .unwrap_or(0);
                lines_counted += layouts_count;
                continue;
            } else if i > end_cursor.line {
                break;
            }

            let starts_at_this_line = i == start_cursor.line;
            let ends_at_this_line = i == end_cursor.line;

            let layouts = line.layout_opt()?;
            for (j, layout) in layouts.iter().enumerate() {
                if starts_at_this_line && j < start_cursor.layout {
                    lines_counted += 1;
                    continue;
                }
                if ends_at_this_line && j > end_cursor.layout {
                    break;
                }

                let starts_at_this_layout = i == start_cursor.line && j == start_cursor.layout;
                let ends_at_this_layout = i == end_cursor.line && j == end_cursor.layout;

                let (first_glyph, last_glyph) = if starts_at_this_layout {
                    let first_glyph = layout.glyphs.get(start_cursor.glyph);
                    let last_glyph = if ends_at_this_layout {
                        layout.glyphs.get(end_cursor.glyph)
                    } else {
                        layout.glyphs.last()
                    };
                    (first_glyph, last_glyph)
                } else if ends_at_this_layout {
                    let first_glyph = layout.glyphs.first();
                    let last_glyph = layout.glyphs.get(end_cursor.glyph);
                    (first_glyph, last_glyph)
                    // If doesn't start nor doesn't end at this layout line, include the whole line
                } else {
                    let first_glyph = layout.glyphs.first();
                    let last_glyph = layout.glyphs.last();
                    (first_glyph, last_glyph)
                };

                self.selection.lines.push(SelectionLine {
                    start_pt: first_glyph.map(|glyph| glyph.x - horizontal_scroll),
                    end_pt: last_glyph.map(|glyph| glyph.x + glyph.w - horizontal_scroll),
                    line_index: Some(lines_counted),
                });

                lines_counted += 1;
            }
        }

        None
    }

    pub fn recalculate(&mut self, ctx: &mut TextContext, update_reason: UpdateReason) {
        let text_buffer_id = self.params.buffer_id();

        self.reshape_if_params_changed(ctx);

        let buffer = ctx
            .buffer_cache
            .buffer_no_retain_mut(&text_buffer_id)
            .unwrap();

        self.recalculate_caret_position_and_scroll(self.params.size(), buffer, update_reason);
        self.update_buffer_size_to_match_element(buffer, self.params.size(), &mut ctx.font_system);
        self.recalculate_selection_area(buffer, &mut ctx.font_system);
    }

    pub fn recalculate_and_reshape_if_needed(&mut self, ctx: &mut TextContext) {
        let params_changed = self.params.changed_since_last_shape();
        self.reshape_if_params_changed(ctx);
        if params_changed {
            self.recalculate(ctx, UpdateReason::Unknown);
        }
    }

    fn update_buffer_size_to_match_element(
        &self,
        buffer: &mut Buffer,
        size: impl Into<Size>,
        font_system: &mut FontSystem,
    ) {
        let size = size.into();
        let scroll = buffer.scroll();
        // TODO: since horizontal scrolling does not appear to work in cosmic_text right
        //  now, we use this hack to scroll the text horizontally
        // let text_area = Rect::new(
        //     (area.min.x - scroll.horizontal, area.min.y).into(),
        //     area.max,
        // );

        buffer.set_size(font_system, Some(size.x), Some(size.y));

        // Setting size resets the scroll, so we need to set it back
        buffer.set_scroll(scroll);
    }

    pub fn recalculate_caret_position_and_scroll(
        &mut self,
        text_area_size: Size,
        buffer: &mut Buffer,
        update_reason: UpdateReason,
    ) -> Option<()> {
        let old_scroll = self.scroll;
        let mut new_scroll = old_scroll;
        let vertical_scroll_to_align_text =
            calculate_vertical_offset(self.params.style(), text_area_size, buffer);

        if self.is_editing {
            let caret_position_relative_to_buffer =
                calculate_caret_position_pt(buffer, self.cursor)?;

            let current_relative_caret_offset = self.relative_caret_offset_horizontal;

            let text_area_width = text_area_size.x;

            // TODO: there was some other implementation that took horizontal alignment into account,
            //  check if it is needed
            let new_absolute_caret_offset = caret_position_relative_to_buffer.x;
            // if let Some(absolute_caret_offset) = caret_position_relative_to_buffer.x {
            //     // Not an empty line
            //     absolute_caret_offset
            // } else {
            //     let container_alignment = self.text_style.horizontal_alignment;
            //     // This means that this is an empty line, and the caret should be aligned to according
            //     //  to the horizontal text alignment
            //     match container_alignment {
            //         TextAlignment::Start => 0.0,
            //         TextAlignment::End => text_area_width,
            //         TextAlignment::Center => text_area_width / 2.0,
            //         // TODO: check that implementations after this are actually correct
            //         TextAlignment::Left => 0.0,
            //         TextAlignment::Right => text_area_width,
            //         TextAlignment::Justify => 0.0,
            //     }
            // };

            // TODO: A little hack to set horizontal scroll
            let mut new_relative_caret_offset = current_relative_caret_offset;

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
                // Check if we should implement improved scroll behavior for overflowing text
                let should_update_horizontal_scroll = self.should_update_horizontal_scroll(
                    buffer,
                    text_area_width,
                    current_relative_caret_offset,
                    new_absolute_caret_offset,
                    old_scroll.horizontal,
                );

                let is_moving_caret = matches!(update_reason, UpdateReason::MoveCaret);

                if should_update_horizontal_scroll && !is_moving_caret {
                    // Improved behavior: keep caret visually fixed, adjust scroll accordingly
                    // We want: new_absolute_caret_offset = new_scroll.horizontal + new_relative_caret_offset
                    // Since we want to keep new_relative_caret_offset = current_relative_caret_offset
                    // We get: new_scroll.horizontal = new_absolute_caret_offset - current_relative_caret_offset
                    new_scroll.horizontal =
                        new_absolute_caret_offset - current_relative_caret_offset;
                    new_relative_caret_offset = current_relative_caret_offset; // Keep caret visually fixed
                } else {
                    // Standard behavior: Do not do anything with the scroll, convert caret to relative
                    new_relative_caret_offset = new_absolute_caret_offset - old_scroll.horizontal;
                    new_scroll.horizontal = old_scroll.horizontal;
                }
            } else if new_absolute_caret_offset > max {
                new_scroll = Scroll::new(
                    0,
                    0.0,
                    new_absolute_caret_offset - text_area_width + self.caret_width,
                );
                // Adjust caret offset to be relative to the new scroll
                new_relative_caret_offset = text_area_width - self.caret_width;
            } else if new_absolute_caret_offset < min {
                new_scroll = Scroll::new(0, 0.0, new_absolute_caret_offset);
                new_relative_caret_offset = 0.0;
            } else if new_absolute_caret_offset < 0.0 {
                new_scroll = Scroll::new(0, 0.0, 0.0);
                new_relative_caret_offset = 0.0;
            } else {
                // Do nothing?
            }

            // let mut vertical_offset = vertical_scroll_to_align_text * -1.0;
            // vertical_offset += caret_position_relative_to_buffer.y;

            self.relative_caret_offset_horizontal = new_relative_caret_offset;
            self.relative_caret_offset_vertical = caret_position_relative_to_buffer.y;
        }

        new_scroll.vertical = vertical_scroll_to_align_text;
        buffer.set_scroll(new_scroll);
        self.scroll = new_scroll;

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
        buffer: &Buffer,
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
        let text_overflows = self.estimate_text_overflows(buffer, text_area_width);
        if !text_overflows {
            return false;
        }

        // Check if caret moved to the left (likely deletion from end)
        let old_absolute_caret_x = old_relative_caret_x + current_scroll_x;

        // Use improved behavior when text overflows and caret moved left
        new_absolute_caret_x < old_absolute_caret_x
    }

    /// Estimates if text overflows the given width by examining the buffer's layout
    fn estimate_text_overflows(&self, buffer: &Buffer, text_area_width: f32) -> bool {
        // Look at the last glyph position to estimate if text overflows
        if let Some(line) = buffer.lines.last() {
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

    pub fn not_shaped(&self, ctx: &mut TextContext) -> bool {
        ctx.buffer_cache
            .buffer_no_retain(&self.params.buffer_id())
            .is_none()
    }

    pub fn size_changed(&self, text_area: Size) -> bool {
        self.params.size() != text_area
    }

    pub fn reshape_if_params_changed(&mut self, ctx: &mut TextContext) {
        self.shape_if_not_shaped(ctx, self.params.changed_since_last_shape());
        self.params.reset_changed();
    }

    fn copy_selected_text(&mut self) -> ActionResult {
        let selected_text = self.selected_text().unwrap_or("".to_string());
        ActionResult::InsertToClipboard(selected_text)
    }

    fn paste_text_at_cursor(&mut self, ctx: &mut TextContext, text: &str) -> ActionResult {
        if !text.is_empty() {
            self.reset_selection_end();
        }

        self.recalculate(ctx, UpdateReason::InsertedText);
        ActionResult::TextChanged
    }

    fn select_all_recalculate(&mut self, ctx: &mut TextContext) -> ActionResult {
        self.select_all();
        self.recalculate(ctx, UpdateReason::SelectionChanged);
        ActionResult::CursorUpdated
    }

    fn cut_selected_text(&mut self, ctx: &mut TextContext) -> ActionResult {
        let selected_text = self.selected_text().unwrap_or("".to_string());
        self.remove_selected_text();
        self.recalculate(ctx, UpdateReason::DeletedTextAtCursor);
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
        self.recalculate(ctx, UpdateReason::DeletedTextAtCursor);
        ActionResult::TextChanged
    }

    fn move_cursor_right_recalculate(&mut self, ctx: &mut TextContext) -> ActionResult {
        if self.is_text_selected() {
            self.move_cursor_to_selection_right();
        } else {
            self.move_cursor(ctx, Motion::Right);
        }
        self.reset_selection();
        self.recalculate(ctx, UpdateReason::MoveCaret);
        ActionResult::CursorUpdated
    }

    fn move_cursor_left_recalculate(&mut self, ctx: &mut TextContext) -> ActionResult {
        if self.is_text_selected() {
            self.move_cursor_to_selection_left();
        } else {
            self.move_cursor(ctx, Motion::Left);
        }
        self.reset_selection();
        self.recalculate(ctx, UpdateReason::MoveCaret);
        ActionResult::CursorUpdated
    }

    fn move_cursor(&mut self, ctx: &mut TextContext, motion: Motion) -> ActionResult {
        let Some(buffer) = ctx
            .buffer_cache
            .buffer_no_retain_mut(&self.params.buffer_id())
        else {
            return ActionResult::None;
        };

        let mut edit = Editor::new(buffer);
        edit.set_cursor(self.cursor.cursor);
        edit.action(&mut ctx.font_system, cosmic_text::Action::Motion(motion));
        self.update_cursor_before_glyph_with_cursor(edit.cursor());

        ActionResult::CursorUpdated
    }

    fn move_cursor_recalculate(&mut self, ctx: &mut TextContext, motion: Motion) -> ActionResult {
        let res = self.move_cursor(ctx, motion);
        self.reset_selection();
        self.recalculate(ctx, UpdateReason::MoveCaret);
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

        self.recalculate(ctx, UpdateReason::InsertedText);
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
        let text_manager = &mut text_context.buffer_cache;
        let font_system = &mut text_context.font_system;
        if self.is_selectable || self.is_editable {
            self.reset_selection();

            let byte_offset_cursor = text_manager.char_under_position(
                self,
                font_system,
                click_position_relative_to_area,
            )?;
            self.update_cursor_before_glyph_with_cursor(byte_offset_cursor);

            // Reset selection to start at the press location
            self.selection.origin_character_byte_cursor = Some(self.cursor);
            self.selection.ends_before_character_byte_cursor = None;

            self.recalculate(text_context, UpdateReason::MoveCaret);
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
        let text_manager = &mut ctx.buffer_cache;
        let font_system = &mut ctx.font_system;
        if self.is_selectable {
            let byte_cursor_under_position =
                text_manager.char_under_position(self, font_system, pointer_relative_position)?;

            // let byte_cursor_char_index =
            //     byte_offset_cursor_to_char_index(self.text(), byte_cursor_under_position)?;

            if let Some(origin) = self.selection.origin_character_byte_cursor {
                if byte_cursor_under_position != origin.cursor {
                    // TODO: probably need to do something with this
                    // self.selection.ends_before_character_byte_cursor =
                    //     Some(byte_cursor_char_index + 1);

                    if let Some(selection) =
                        self.selection.ends_before_character_byte_cursor.as_mut()
                    {
                        selection.update_cursor(byte_cursor_under_position, self.params.text());
                    } else {
                        self.selection.ends_before_character_byte_cursor =
                            ByteCursor::from_cursor(byte_cursor_under_position, self.params.text());
                    }
                }
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

            self.recalculate(ctx, UpdateReason::MoveCaret);
        }

        None
    }
}

/// Takes element height, text buffer height and vertical alignment and returns the vertical offset
///  needed to align the text vertically.
fn calculate_vertical_offset(text_style: &TextStyle, text_area_size: Size, buffer: &Buffer) -> f32 {
    let normalized_area = Rect::new((0.0, 0.0).into(), text_area_size);
    let style = text_style;

    let vertical_alignment = style.vertical_alignment;
    // TODO: fix scaling
    let buffer_height = buffer_height(buffer, style, 2.0);
    // TODO: FIX TOP.
    let vertical_offset = vertical_offset(vertical_alignment, normalized_area, buffer_height);

    0.0 - vertical_offset
}

pub enum UpdateReason {
    SelectionChanged,
    InsertedText,
    MoveCaret,
    DeletedTextAtCursor,
    Unknown,
}
