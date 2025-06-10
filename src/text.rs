use crate::byte_cursor::ByteCursor;
use crate::math::{Point, Rect};
use crate::state::TextState;
use crate::style::TextStyle;
use crate::style::TextWrap;
use crate::{Id, VerticalTextAlignment};
use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use cosmic_text::{
    Attrs, Buffer, Cursor, Edit, Editor, FontSystem, LayoutCursor, Metrics, Shaping,
};

#[derive(Default)]
pub struct TextManager {
    pub buffer_cache: HashMap<Id, Buffer>,
    pub buffers_accessed_last_frame: HashSet<Id>,
}

impl From<TextWrap> for cosmic_text::Wrap {
    fn from(value: TextWrap) -> Self {
        match value {
            TextWrap::NoWrap => cosmic_text::Wrap::None,
            TextWrap::Wrap => cosmic_text::Wrap::Word,
            TextWrap::BreakWord => cosmic_text::Wrap::Glyph,
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct GlyphPosition {
    pub x: f32,
    pub y: f32,
    pub width: f32,
}

impl TextManager {
    pub fn new() -> Self {
        Self {
            buffer_cache: HashMap::new(),
            buffers_accessed_last_frame: HashSet::new(),
        }
    }

    pub fn start_frame(&mut self) {
        self.buffers_accessed_last_frame.clear();
    }

    pub fn end_frame(&mut self) {
        self.buffer_cache
            .retain(|id, _| self.buffers_accessed_last_frame.contains(id));
    }

    pub fn buffer(&mut self, id: &Id) -> Option<&Buffer> {
        self.buffers_accessed_last_frame.insert(*id);
        self.buffer_cache.get(id)
    }

    pub fn buffer_no_retain(&self, id: &Id) -> Option<&Buffer> {
        self.buffer_cache.get(id)
    }

    pub fn buffer_no_retain_mut(&mut self, id: &Id) -> Option<&mut Buffer> {
        self.buffer_cache.get_mut(id)
    }

    pub fn buffer_mut(&mut self, id: &Id) -> Option<&mut Buffer> {
        self.buffer_cache.get_mut(id)
    }

    pub fn retain_buffer(&mut self, id: Id) {
        self.buffers_accessed_last_frame.insert(id);
    }

    pub fn insert_buffer(&mut self, id: Id, buffer: Buffer) {
        self.buffer_cache.insert(id, buffer);
        self.buffers_accessed_last_frame.insert(id);
    }

    pub fn remove_buffer(&mut self, id: Id) {
        self.buffer_cache.remove(&id);
    }

    pub fn char_under_position(
        &mut self,
        state: &TextState,
        font_system: &mut FontSystem,
        interaction_position_relative_to_element: Point,
    ) -> Option<Cursor> {
        let buffer = self.buffer_no_retain_mut(&state.text_buffer_id)?;
        let horizontal_scroll = buffer.scroll().horizontal;
        let byte_offset_cursor = buffer.hit(
            interaction_position_relative_to_element.x + horizontal_scroll,
            interaction_position_relative_to_element.y,
        )?;

        // TODO: cursor doesn't seem to correctly detect hit to the left of the line,
        //  so there's a little hack to detect if it is to the left of the line's first glyph
        // ======== Check that click isn't to the left of the line's first glyph =========
        let layout_cursor = buffer.layout_cursor(font_system, byte_offset_cursor)?;

        let first_glyph_on_line_cursor = LayoutCursor {
            line: layout_cursor.line,
            layout: layout_cursor.layout,
            glyph: 0,
        };
        let position_of_first_glyph = TextManager::get_position_of_a_glyph_with_buffer_and_cursor(
            buffer,
            first_glyph_on_line_cursor,
        );

        if let Some(position) = position_of_first_glyph {
            if interaction_position_relative_to_element.x < position.x {
                let first_glyph_cursor = Cursor {
                    line: byte_offset_cursor.line,
                    index: byte_offset_cursor.index.saturating_sub(layout_cursor.glyph),
                    affinity: Default::default(),
                };
                return Some(first_glyph_cursor);
            }
        }
        // ================================================================================

        Some(byte_offset_cursor)
    }

    pub fn get_position_of_a_glyph(
        &mut self,
        buffer_id: &Id,
        line_index: usize,
        glyph_index: usize,
    ) -> Option<GlyphPosition> {
        let buffer = self.buffer_no_retain(buffer_id)?;
        Self::get_position_of_a_glyph_with_buffer(buffer, line_index, glyph_index)
    }

    pub fn get_position_of_a_glyph_with_buffer(
        buffer: &Buffer,
        line_index: usize,
        glyph_index: usize,
    ) -> Option<GlyphPosition> {
        let line = buffer.lines.get(line_index)?;
        let layout = line.layout_opt().as_ref()?.get(line_index)?;
        let glyph = layout.glyphs.get(glyph_index)?;
        Some(GlyphPosition {
            x: glyph.x,
            y: glyph.y,
            width: glyph.w,
        })
    }

    pub fn get_position_of_a_glyph_with_buffer_and_cursor(
        buffer: &Buffer,
        cursor: LayoutCursor,
    ) -> Option<GlyphPosition> {
        let line = buffer.lines.get(cursor.line)?;
        let layout = line.layout_opt().as_ref()?.get(cursor.layout)?;
        let glyph = layout.glyphs.get(cursor.glyph)?;

        Some(GlyphPosition {
            x: glyph.x,
            y: glyph.y,
            width: glyph.w,
        })
    }

    pub fn get_position_of_last_glyph(&mut self, buffer_id: &Id) -> Option<GlyphPosition> {
        let buffer = self.buffer_no_retain(buffer_id)?;
        Self::get_position_of_last_glyph_buffer(buffer)
    }

    pub fn get_position_of_last_glyph_buffer(buffer: &Buffer) -> Option<GlyphPosition> {
        let line = buffer.lines.last()?;
        let line_index = buffer.lines.len().saturating_sub(1);
        let glyph_index = line
            .layout_opt()
            .as_ref()?
            .last()?
            .glyphs
            .len()
            .saturating_sub(1);
        Self::get_position_of_a_glyph_with_buffer(buffer, line_index, glyph_index)
    }

    pub fn create_and_shape_text_if_not_in_cache(
        &mut self,
        text: &str,
        text_style: &TextStyle,
        buffer_id: Id,
        element_area: Rect,
        font_system: &mut FontSystem,
        reshape: bool,
    ) {
        let buffer_not_in_cache = self.buffer_no_retain(&buffer_id).is_none();
        if buffer_not_in_cache || reshape {
            self.create_and_shape_text_buffer(
                text,
                text_style,
                buffer_id,
                element_area,
                font_system,
                None,
            );
        }
    }

    pub fn create_and_shape_text_buffer(
        &mut self,
        text: &str,
        text_style: &TextStyle,
        buffer_id: Id,
        element_area: Rect,
        font_system: &mut FontSystem,
        cursor: Option<cosmic_text::Cursor>,
    ) {
        let font_color = text_style.font_color;

        let horizontal_alignment = text_style.horizontal_alignment;

        let text_area_size = element_area.size();

        let metrics = Metrics::new(text_style.font_size.0, text_style.line_height_pt());
        let mut buffer = Buffer::new(font_system, metrics);
        buffer.set_wrap(font_system, text_style.wrap.unwrap_or_default().into());

        let font_family = &text_style.font_family;

        buffer.set_size(font_system, Some(text_area_size.0), Some(text_area_size.1));

        buffer.set_text(
            font_system,
            text,
            &Attrs::new()
                .color(font_color.into())
                .family(font_family.to_fontdb_family())
                .metadata(buffer_id.0 as usize),
            Shaping::Advanced,
        );

        for line in buffer.lines.iter_mut() {
            line.set_align(horizontal_alignment.into());
        }

        if let Some(cursor) = cursor {
            buffer.shape_until_cursor(font_system, cursor, false);
        } else {
            buffer.shape_until_scroll(font_system, false);
        }

        self.insert_buffer(buffer_id, buffer);
    }
}

pub(crate) fn buffer_height(buffer: &Buffer, style: &TextStyle, scale: f32) -> f32 {
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for layout_run in buffer.layout_runs() {
        for glyph in layout_run.glyphs.iter() {
            let physical_glyph = glyph.physical((0.0, 0.0), scale);
            min_y = min_y.min(physical_glyph.y as f32 + layout_run.line_y);
            max_y = max_y.max(physical_glyph.y as f32 + layout_run.line_y);
        }
    }

    if max_y > min_y {
        max_y - min_y + style.line_height_pt()
    } else {
        // For a single line, return the font size
        style.line_height_pt()
    }
}

pub(crate) fn vertical_offset(
    vertical_alignment: VerticalTextAlignment,
    area: Rect,
    buffer_height: f32,
) -> f32 {
    match vertical_alignment {
        VerticalTextAlignment::Start => area.min.y,
        VerticalTextAlignment::End => area.max.y - buffer_height,
        VerticalTextAlignment::Center => area.min.y + (area.height() - buffer_height) / 2.0,
    }
}

pub(crate) fn calculate_caret_position_pt(
    buffer: &mut Buffer,
    current_char_byte_cursor: ByteCursor,
) -> Option<Point> {
    let mut edit = Editor::new(&mut *buffer);
    edit.set_cursor(current_char_byte_cursor.cursor);

    edit.cursor_position().map(|(x, y)| Point {
        x: x as f32,
        y: y as f32,
    })
}
