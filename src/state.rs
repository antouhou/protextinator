use crate::action::{Action, ActionResult};
use crate::ctx::TextContext;
use crate::style::{TextAlignment, TextStyle, VerticalTextAlignment};
use crate::text::{
    buffer_height, calculate_caret_position_pt, char_index_to_layout_cursor, insert_character_at,
    insert_multiple_characters_at, remove_character_at, remove_multiple_characters_at,
    vertical_offset,
};
use crate::{Id, Point, Rect, TextManager};
use cosmic_text::{Buffer, Cursor, FontSystem, Scroll};
use smol_str::SmolStr;
use std::os::macos::raw::stat;
use std::time::{Duration, Instant};

#[derive(Clone, Default, Debug, Copy)]
pub struct SelectionLine {
    pub start_pt: Option<f32>,
    pub end_pt: Option<f32>,
    pub line_index: Option<usize>,
}

#[derive(Clone, Default, Debug)]
pub struct Selection {
    pub origin_character_index: Option<usize>,
    pub ends_before_character_index: Option<usize>,
    pub lines: Vec<SelectionLine>,
}

pub struct TextState {
    pub is_first_run: bool,
    text: String,
    pub is_focused: bool,

    pub cursor_before_glyph: usize,
    pub relative_caret_offset_horizontal: f32,
    pub relative_caret_offset_vertical: f32,
    /// The horizontal offset of the text inside the buffer. It is needed since horizontal scrolling
    ///  in cosmic_text does not seem to work.
    pub scroll: Scroll,
    /// The number of characters in the text.
    text_size: usize,

    pub selection: Selection,

    pub last_scroll_timestamp: Instant,
    pub scroll_interval: Duration,

    pub text_style: TextStyle,
    text_area: Rect,
    pub(crate) text_buffer_id: Id,
    pub caret_width: f32,

    pub is_selectable: bool,
    pub is_editable: bool,
}

impl TextState {
    pub fn new_with_text(text: impl Into<String>, text_buffer_id: Id) -> Self {
        let text = text.into();
        let char_count = text.chars().count();

        Self {
            is_first_run: true,
            text,
            is_focused: false,
            cursor_before_glyph: 0,
            relative_caret_offset_horizontal: 0.0,
            relative_caret_offset_vertical: 0.0,
            scroll: Scroll::new(0, 0.0, 0.0),
            text_size: char_count,
            selection: Selection::default(),
            last_scroll_timestamp: Instant::now(),
            scroll_interval: Duration::from_millis(50),
            text_area: Rect::default(),
            text_style: TextStyle::default(),
            text_buffer_id,
            caret_width: 3.0,
            is_selectable: false,
            is_editable: false,
        }
    }

    pub fn set_caret_width(&mut self, width: f32) {
        self.caret_width = width;
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.text_size = self.text.chars().count();

        if self.cursor_before_glyph > self.text_size {
            self.cursor_before_glyph = self.text_size;
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn text_size(&self) -> usize {
        self.text_size
    }

    pub fn insert_char_at_cursor(&mut self, character: char) -> ActionResult {
        self.text_size = insert_character_at(&mut self.text, self.cursor_before_glyph, character);
        self.cursor_before_glyph += 1;
        ActionResult::None
    }

    pub fn insert_text_at_cursor(&mut self, text: &str) -> usize {
        let old_text_size = self.text_size;
        self.text_size =
            insert_multiple_characters_at(&mut self.text, self.cursor_before_glyph, text);
        let move_cursor_by = self.text_size - old_text_size;
        self.cursor_before_glyph += move_cursor_by;
        self.text_size
    }

    pub fn remove_char_at_cursor(&mut self) {
        if !self.text.is_empty() && self.cursor_before_glyph > 0 {
            self.text_size = remove_character_at(&mut self.text, self.cursor_before_glyph - 1);
            self.cursor_before_glyph -= 1;
        }
    }

    pub fn remove_selected_text(&mut self) {
        if let (Some(origin), Some(end)) = (
            self.selection.origin_character_index,
            self.selection.ends_before_character_index,
        ) {
            if origin > end {
                let count = origin - end;
                self.text_size = remove_multiple_characters_at(&mut self.text, end, count);
                self.cursor_before_glyph = end;
            } else {
                let count = end - origin;
                self.text_size = remove_multiple_characters_at(&mut self.text, origin, count);
                self.cursor_before_glyph = origin;
            }
            self.reset_selection();
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_before_glyph > 0 {
            self.cursor_before_glyph -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_before_glyph < self.text_size {
            self.cursor_before_glyph += 1;
        }
    }

    pub fn move_cursor_to(&mut self, glyph_index: usize) {
        self.cursor_before_glyph = glyph_index;
    }

    pub fn move_cursor_to_selection_left(&mut self) {
        if let (Some(origin), Some(end)) = (
            self.selection.origin_character_index,
            self.selection.ends_before_character_index,
        ) {
            if origin > end {
                self.move_cursor_to(end);
            } else {
                self.move_cursor_to(origin);
            }
        }
    }

    pub fn move_cursor_to_selection_right(&mut self) {
        if let (Some(origin), Some(end)) = (
            self.selection.origin_character_index,
            self.selection.ends_before_character_index,
        ) {
            if origin < end {
                self.cursor_before_glyph = end;
            } else {
                self.cursor_before_glyph = origin;
            }
        }
    }

    pub fn is_text_selected(&self) -> bool {
        if let Some(origin) = self.selection.origin_character_index {
            if let Some(end) = self.selection.ends_before_character_index {
                origin != end
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn reset_selection_end(&mut self) {
        self.selection.ends_before_character_index = None;
        self.selection.lines.clear();
    }

    pub fn reset_selection(&mut self) {
        self.selection.origin_character_index = None;
        self.selection.ends_before_character_index = None;
        self.selection.lines.clear();
    }

    pub fn select_all(&mut self) {
        self.selection.origin_character_index = Some(0);
        self.selection.ends_before_character_index = Some(self.text_size);
    }

    pub fn substring(&self, start: usize, end: usize) -> String {
        self.text.chars().skip(start).take(end - start).collect()
    }

    pub fn selected_text(&self) -> Option<String> {
        if let (Some(mut origin), Some(mut end)) = (
            self.selection.origin_character_index,
            self.selection.ends_before_character_index,
        ) {
            if origin > end {
                std::mem::swap(&mut origin, &mut end);
            }
            Some(self.substring(origin, end))
        } else {
            None
        }
    }

    pub fn shape_if_not_shaped(
        &self,
        text_style: &TextStyle,
        text_id: Id,
        text_area: impl Into<Rect>,
        ctx: &mut TextContext,
        reshape: bool,
    ) {
        let text_area = text_area.into();
        let font_system = &mut ctx.font_system;
        ctx.text_manager.create_and_shape_text_if_not_in_cache(
            &self.text,
            text_style,
            text_id,
            text_area,
            font_system,
            reshape,
        );
    }

    /// Calculates physical selection area based on the selection start and end glyph indices
    fn recalculate_selection_area(
        &mut self,
        buffer: &mut Buffer,
        font_system: &mut FontSystem,
    ) -> Option<(f32, f32)> {
        let mut selection_starts_at_index = self.selection.origin_character_index?;
        let mut selection_ends_before_char_index = self.selection.ends_before_character_index?;
        if selection_starts_at_index > selection_ends_before_char_index {
            // Swap the values
            std::mem::swap(
                &mut selection_ends_before_char_index,
                &mut selection_starts_at_index,
            );
        }

        let selection_end_char_index = if selection_ends_before_char_index > 0 {
            if selection_starts_at_index == selection_ends_before_char_index {
                selection_starts_at_index -= 1;
            }
            selection_ends_before_char_index - 1
        } else {
            0
        };

        let start_cursor = char_index_to_layout_cursor(
            buffer,
            font_system,
            &self.text,
            selection_starts_at_index,
        )?;
        let end_cursor =
            char_index_to_layout_cursor(buffer, font_system, &self.text, selection_end_char_index)?;

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

            let layouts = line.layout_opt().as_ref()?;
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

    pub fn text_area(&self) -> Rect {
        self.text_area
    }

    /// DO NOT PASS VALUES FROM THE STATE TO THIS FUNCTION
    pub fn update_area_and_recalculate(
        &mut self,
        text_area: impl Into<Rect>,
        text_style: &TextStyle,
        ctx: &mut TextContext,
        reshape: bool,
    ) {
        // Update the text area
        self.text_area = text_area.into();
        self.text_style = *text_style;

        self.recalculate(ctx, reshape);
    }

    pub fn recalculate(&mut self, ctx: &mut TextContext, reshape: bool) {
        let text_area = self.text_area;
        let text_style = self.text_style;
        let text_buffer_id = self.text_buffer_id;

        self.shape_if_not_shaped(&text_style, text_buffer_id, text_area, ctx, reshape);

        let buffer = ctx.text_manager.buffer_no_retain_mut(&text_buffer_id).unwrap();

        self.recalculate_caret_position_and_scroll(
            &text_style,
            text_area,
            buffer,
            &mut ctx.font_system,
        );
        self.update_buffer_size_to_match_element(buffer, text_area, &mut ctx.font_system);
        self.recalculate_selection_area(buffer, &mut ctx.font_system);
    }

    fn update_buffer_size_to_match_element(
        &self,
        buffer: &mut Buffer,
        area: impl Into<Rect>,
        font_system: &mut FontSystem,
    ) {
        let area = area.into();
        let scroll = buffer.scroll();
        // TODO: since horizontal scrolling does not appear to work in cosmic_text right
        //  now, we use this hack to scroll the text horizontally
        let text_area = Rect::new(
            (area.min.x - scroll.horizontal, area.min.y).into(),
            area.max,
        );

        buffer.set_size(
            font_system,
            Some(text_area.width()),
            Some(text_area.height()),
        );

        // Setting size resets the scroll, so we need to set it back
        buffer.set_scroll(scroll);
    }

    pub fn recalculate_caret_position_and_scroll(
        &mut self,
        text_style: &TextStyle,
        text_area: Rect,
        buffer: &mut Buffer,
        font_system: &mut FontSystem,
    ) -> Option<()> {
        let caret_position =
            calculate_caret_position_pt(buffer, self.cursor_before_glyph, &self.text, font_system)?;

        let current_relative_caret_offset = self.relative_caret_offset_horizontal;
        let old_scroll = self.scroll;
        let line_height = text_style.line_height_pt();
        let text_area_width = text_area.width();
        let vertical_scroll_to_align_text =
            calculate_vertical_offset(text_style, text_area, buffer);

        let new_absolute_caret_offset = if let Some(absolute_caret_offset) = caret_position.x {
            absolute_caret_offset
        } else {
            let container_alignment = text_style.horizontal_alignment;
            // This means that this is an empty line, and the caret should be aligned to according
            //  to the horizontal text alignment
            match container_alignment {
                TextAlignment::Start => 0.0,
                TextAlignment::End => text_area_width,
                TextAlignment::Center => text_area_width / 2.0,
                // TODO: check that implementations after this are actually correct
                TextAlignment::Left => 0.0,
                TextAlignment::Right => text_area_width,
                TextAlignment::Justify => 0.0,
            }
        };

        // TODO: A little hack to set horizontal scroll
        let mut new_scroll = old_scroll;
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
            // Do not do anything with the scroll, convert caret to relative
            new_relative_caret_offset = new_absolute_caret_offset - old_scroll.horizontal;
            new_scroll.horizontal = 0.0;
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

        new_scroll.vertical = vertical_scroll_to_align_text;
        buffer.set_scroll(new_scroll);
        self.scroll = new_scroll;

        let mut vertical_offset = vertical_scroll_to_align_text * -1.0;
        vertical_offset += caret_position.line as f32 * line_height;

        self.relative_caret_offset_horizontal = new_relative_caret_offset;
        self.relative_caret_offset_vertical = vertical_offset;

        None
    }

    pub fn not_shaped(&self, ctx: &mut TextContext) -> bool {
        ctx.text_manager.buffer_no_retain(&self.text_buffer_id).is_none()
    }

    pub fn size_changed(&self, text_area: (f32, f32)) -> bool {
        self.text_area.size() != text_area
    }

    fn copy_selected_text(&mut self, ctx: &mut TextContext) -> ActionResult {
        let selected_text = self.selected_text().unwrap_or("".to_string());
        ActionResult::InsertToClipboard(selected_text)
    }

    fn paste_text_at_cursor(&mut self, ctx: &mut TextContext, text: &str) -> ActionResult {
        let old_text_size = self.text_size();
        let new_text_size = self.insert_text_at_cursor(text);
        if old_text_size != new_text_size {
            self.reset_selection_end();
        }

        self.recalculate(ctx, true);
        ActionResult::None
    }

    fn select_all_recalculate(&mut self, ctx: &mut TextContext) -> ActionResult {
        self.select_all();
        self.recalculate(ctx, false);
        ActionResult::None
    }

    fn cut_selected_text(&mut self, ctx: &mut TextContext) -> ActionResult {
        let selected_text = self.selected_text().unwrap_or("".to_string());
        self.remove_selected_text();
        self.recalculate(ctx, true);
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
        self.recalculate(ctx, true);
        ActionResult::None
    }

    fn insert_whitespace_at_cursor(&mut self, ctx: &mut TextContext) -> ActionResult {
        self.insert_char_at_cursor(' ');
        self.reset_selection();
        self.recalculate(ctx, true);
        ActionResult::None
    }

    fn move_cursor_right_recalculate(&mut self, ctx: &mut TextContext) -> ActionResult {
        if self.is_text_selected() {
            self.move_cursor_to_selection_right();
        } else {
            self.move_cursor_right();
        }
        self.reset_selection();
        self.recalculate(ctx, false);
        ActionResult::None
    }

    fn move_cursor_left_recalculate(&mut self, ctx: &mut TextContext) -> ActionResult {
        if self.is_text_selected() {
            self.move_cursor_to_selection_left();
        } else {
            self.move_cursor_left();
        }
        self.reset_selection();
        self.recalculate(ctx, false);
        ActionResult::None
    }

    fn insert_character_before_cursor(&mut self, character: &SmolStr, ctx: &mut TextContext) -> ActionResult {
        if self.is_text_selected() {
            self.move_cursor_left();
            self.remove_selected_text();
        }
        for character in character.chars() {
            self.insert_char_at_cursor(character);
            self.reset_selection_end();
        }

        self.recalculate(ctx, true);
        ActionResult::None
    }

    pub fn apply_action(
        &mut self,
        ctx: &mut TextContext,
        action: &Action,
    ) -> ActionResult {
        if self.is_editable && self.is_selectable {
            match action {
                Action::Paste(text) => self.paste_text_at_cursor(ctx, &text),
                Action::Cut => self.cut_selected_text(ctx),
                Action::DeleteBackward => self.delete_selected_text_or_text_before_cursor(ctx),
                Action::InsertWhitespace => self.insert_whitespace_at_cursor(ctx),
                Action::MoveCursorRight => self.move_cursor_right_recalculate(ctx),
                Action::MoveCursorLeft => self.move_cursor_left_recalculate(ctx),
                Action::InsertChar(character) => self.insert_character_before_cursor(&character, ctx),
                _ => ActionResult::None,
            }
        } else if self.is_selectable {
            match action {
                Action::Copy => self.copy_selected_text(ctx),
                Action::SelectAll => self.select_all_recalculate(ctx),
                _ => ActionResult::None,
            }
        } else {
            ActionResult::None
        }
    }
}

/// Takes element height, text buffer height and vertical alignment and returns the vertical offset
/// that is needed to align the text vertically.
pub fn calculate_vertical_offset(text_style: &TextStyle, text_area: Rect, buffer: &Buffer) -> f32 {
    let area = text_area;
    let normalized_area = Rect::new((0.0, 0.0).into(), (area.width(), area.height()).into());
    let style = text_style;

    let vertical_alignment = style.vertical_alignment;
    // TODO: fix scaling
    let buffer_height = buffer_height(buffer, style, 2.0);
    // TODO: FIX TOP.
    let vertical_offset = vertical_offset(vertical_alignment, normalized_area, buffer_height);

    0.0 - vertical_offset
}

pub fn cursor_to_char_index(string: &str, cursor: Cursor) -> Option<usize> {
    let mut char_index = 0;

    // Iterate through lines until we reach cursor.line
    for (line_number, line) in string.lines().enumerate() {
        if line_number == cursor.line {
            // Add the index within this line, but ensure it doesn't exceed the line length
            if cursor.index <= line.chars().count() {
                return Some(char_index + cursor.index);
            } else {
                // Cursor index is out of bounds for this line
                return None;
            }
        }

        // Add line length plus 1 for the newline character
        char_index += line.chars().count() + 1;
    }

    // If cursor.line is beyond the available lines
    None
}

pub fn handle_click(
    state: &mut TextState,
    text_context: &mut TextContext,
    click_position_relative_to_area: Point,
) -> Option<()> {
    let text_manager = &mut text_context.text_manager;
    let font_system = &mut text_context.font_system;
    if state.is_selectable || state.is_editable {
        state.reset_selection();

        let glyph_cursor = text_manager.glyph_under_position(
            state,
            font_system,
            click_position_relative_to_area,
        )?;
        let char_index = cursor_to_char_index(state.text(), glyph_cursor)?;
        state.move_cursor_to(char_index);
        state.recalculate(text_context, false);
    }

    None
}

pub fn handle_drag(
    text_state: &mut TextState,
    ctx: &mut TextContext,
    is_dragging: bool,
    pointer_relative_position: Point,
    pointer_absolute_position: Point,
) -> Option<()> {
    let text_manager = &mut ctx.text_manager;
    let font_system = &mut ctx.font_system;
    if text_state.is_selectable {
        let glyph_cursor = text_manager.glyph_under_position(
            text_state,
            font_system,
            pointer_relative_position,
        )?;

        let char_index_under_position = cursor_to_char_index(text_state.text(), glyph_cursor)?;

        if let Some(origin) = text_state.selection.origin_character_index {
            if char_index_under_position != origin {
                text_state.selection.ends_before_character_index =
                    Some(char_index_under_position + 1);
            }
        } else {
            text_state.selection.origin_character_index = Some(char_index_under_position);
        }

        // Simple debounce to make scroll speed consistent
        let now = std::time::Instant::now();
        if now > text_state.last_scroll_timestamp + text_state.scroll_interval && is_dragging {
                let element_area = text_state.text_area();
                let is_dragging_to_the_right = pointer_absolute_position.x > element_area.max.x;
                let is_dragging_to_the_left = pointer_absolute_position.x < element_area.min.x;

                if is_dragging_to_the_right || is_dragging_to_the_left {
                    text_state.move_cursor_to(char_index_under_position);
                    text_state.last_scroll_timestamp = now;
                }
        }

        text_state.recalculate(ctx, false);
    }

    None
}
