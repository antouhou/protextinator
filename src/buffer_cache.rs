use crate::byte_cursor::ByteCursor;
use crate::math::{Point, Rect};
use crate::state::TextState;
use crate::style::TextStyle;
use crate::style::TextWrap;
use crate::{Id, TextParams, VerticalTextAlignment};
use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
#[cfg(test)]
use cosmic_text::LayoutGlyph;
use cosmic_text::{Attrs, Buffer, Cursor, Edit, Editor, FontSystem, Shaping};

#[derive(Default)]
pub struct BufferCache {
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

impl BufferCache {
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

    #[cfg(test)]
    pub fn first_glyph(&mut self, id: &Id) -> Option<&LayoutGlyph> {
        self.buffer(id).and_then(|buffer| {
            buffer
                .layout_runs()
                .next()
                .and_then(|run| run.glyphs.first())
        })
    }

    pub fn char_under_position(
        &mut self,
        state: &TextState,
        interaction_position_relative_to_element: Point,
    ) -> Option<Cursor> {
        let buffer = self.buffer_no_retain_mut(&state.params.buffer_id())?;
        let horizontal_scroll = buffer.scroll().horizontal;
        buffer.hit(
            interaction_position_relative_to_element.x + horizontal_scroll,
            interaction_position_relative_to_element.y,
        )
    }

    pub fn shape_buffer_if_needed(
        &mut self,
        params: &TextParams,
        font_system: &mut FontSystem,
        reshape: bool,
        shape_till_cursor: Option<Cursor>,
    ) {
        let buffer_not_in_cache = self.buffer_no_retain(&params.buffer_id()).is_none();
        if buffer_not_in_cache || reshape {
            self.create_or_update_and_shape_text_buffer(params, font_system, shape_till_cursor);
        }
    }

    fn create_buffer(
        &mut self,
        params: &TextParams,
        font_system: &mut FontSystem,
        cursor: Option<cosmic_text::Cursor>,
    ) {
        let mut buffer = Buffer::new(font_system, params.metrics());

        BufferCache::update_buffer(params, &mut buffer, font_system, cursor);

        self.insert_buffer(params.buffer_id(), buffer);
    }

    fn update_buffer(
        params: &TextParams,
        buffer: &mut Buffer,
        font_system: &mut FontSystem,
        cursor: Option<Cursor>,
    ) {
        let old_scroll = buffer.scroll();

        buffer.set_metrics(font_system, params.metrics());

        let text_style = &params.style();
        let font_color = text_style.font_color;

        let horizontal_alignment = params.style().horizontal_alignment;

        let text_area_size = params.size();

        buffer.set_wrap(font_system, text_style.wrap.unwrap_or_default().into());

        let font_family = &text_style.font_family;

        // TODO: do we actually need to preserve scroll here?
        buffer.set_size(font_system, Some(text_area_size.x), Some(text_area_size.y));
        buffer.set_scroll(old_scroll);

        buffer.set_text(
            font_system,
            params.text(),
            &Attrs::new()
                .color(font_color.into())
                .family(font_family.to_fontdb_family())
                .metadata(params.buffer_id().0 as usize),
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
    }

    pub fn create_or_update_and_shape_text_buffer(
        &mut self,
        params: &TextParams,
        font_system: &mut FontSystem,
        cursor: Option<cosmic_text::Cursor>,
    ) {
        if let Some(buffer) = self.buffer_mut(&params.buffer_id()) {
            BufferCache::update_buffer(params, buffer, font_system, cursor);
        } else {
            self.create_buffer(params, font_system, cursor);
        }
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

pub(crate) fn calculate_caret_position_pt_and_update_vertical_scroll(
    buffer: &mut Buffer,
    current_char_byte_cursor: ByteCursor,
    font_system: &mut FontSystem,
) -> Option<Point> {
    let mut edit = Editor::new(&mut *buffer);
    edit.set_cursor(current_char_byte_cursor.cursor);

    // TODO: do this only if something changed
    edit.shape_as_needed(font_system, false);

    

    edit.cursor_position().map(|(x, y)| Point {
        x: x as f32,
        y: y as f32,
    })
}
