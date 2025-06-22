use crate::byte_cursor::ByteCursor;
use crate::math::{Point, Rect, Size};
use crate::state::{calculate_vertical_offset, TextState};
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

    /// Shapes the text buffer if it is not in the cache or if `reshape` is true.
    /// Returns the inner buffer dimensions if the buffer was created or updated
    pub fn shape_buffer_if_needed(
        &mut self,
        params: &TextParams,
        font_system: &mut FontSystem,
        reshape: bool,
        shape_till_cursor: Option<Cursor>,
    ) -> Option<Size> {
        let buffer_not_in_cache = self.buffer_no_retain(&params.buffer_id()).is_none();
        if buffer_not_in_cache || reshape {
            Some(self.create_or_update_and_shape_text_buffer(
                params,
                font_system,
                shape_till_cursor,
            ))
        } else {
            None
        }
    }

    /// Creates a new buffer. Returns inner buffer dimensions.
    fn create_buffer(
        &mut self,
        params: &TextParams,
        font_system: &mut FontSystem,
        cursor: Option<cosmic_text::Cursor>,
    ) -> Size {
        let mut buffer = Buffer::new(font_system, params.metrics());

        let size = BufferCache::update_buffer(params, &mut buffer, font_system, cursor);

        self.insert_buffer(params.buffer_id(), buffer);

        size
    }

    /// Returns inner buffer dimensions
    fn update_buffer(
        params: &TextParams,
        buffer: &mut Buffer,
        font_system: &mut FontSystem,
        cursor: Option<Cursor>,
    ) -> Size {
        let old_scroll = buffer.scroll();

        buffer.set_metrics(font_system, params.metrics());

        let text_style = &params.style();
        let font_color = text_style.font_color;

        let horizontal_alignment = params.style().horizontal_alignment;

        let text_area_size = params.size();

        buffer.set_wrap(font_system, text_style.wrap.unwrap_or_default().into());

        let font_family = &text_style.font_family;

        buffer.set_size(font_system, Some(text_area_size.x), Some(text_area_size.y));

        buffer.set_text(
            font_system,
            params.text(),
            &Attrs::new()
                .color(font_color.into())
                .family(font_family.to_fontdb_family())
                .metadata(params.buffer_id().0 as usize),
            Shaping::Advanced,
        );

        let mut buffer_measurement = Size::default();
        for line in buffer.lines.iter_mut() {
            for line in line
                .layout(
                    font_system,
                    text_style.font_size.value(),
                    Some(text_area_size.x),
                    text_style.wrap.unwrap_or_default().into(),
                    None,
                    // TODO: what is the default tab width? Make it configurable?
                    2,
                )
                .iter()
            {
                buffer_measurement.y += line.line_height_opt.unwrap_or(text_style.line_height_pt());
                buffer_measurement.x = buffer_measurement.x.max(line.w);
            }
            line.set_align(horizontal_alignment.into());
        }

        if let Some(cursor) = cursor {
            buffer.shape_until_cursor(font_system, cursor, false);
        } else {
            buffer.shape_until_scroll(font_system, false);
        }

        // Restore the scroll position, so adding text does not change the scroll position.
        buffer.set_scroll(old_scroll);
        buffer_measurement
    }

    /// Creates a new buffer or updates an existing one, and shapes it. Returns inner buffer
    /// dimensions.
    pub fn create_or_update_and_shape_text_buffer(
        &mut self,
        params: &TextParams,
        font_system: &mut FontSystem,
        cursor: Option<cosmic_text::Cursor>,
    ) -> Size {
        if let Some(buffer) = self.buffer_mut(&params.buffer_id()) {
            BufferCache::update_buffer(params, buffer, font_system, cursor)
        } else {
            self.create_buffer(params, font_system, cursor)
        }
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
    text_area_size: Size,
    style: &TextStyle,
    buffer_inner_dimensions: Size,
) -> Option<Point> {
    let mut editor = Editor::new(&mut *buffer);
    editor.set_cursor(current_char_byte_cursor.cursor);

    let caret_position = editor.cursor_position();

    match caret_position {
        Some(position) => {
            let mut point = Point::from(position);
            let mut scroll = buffer.scroll();

            // If the caret is not fully visible, we need to scroll it into view
            // TODO: maybe to that if the end of caret is larger than the text area size as well?
            if point.y < 0.0 {
                scroll.vertical += point.y;
                point.y = 0.0;
                buffer.set_scroll(scroll);
            }

            Some(point)
        }
        None => {
            // Caret is not visible, we need to shape the text and move the scroll
            // TODO: do this only if we're sure we need to shape the text
            editor.shape_as_needed(font_system, false);

            // If it's not visible, and the scroll is already at the top, that means that we're
            //  at the end of the text, and we need to scroll to the bottom to avoid jumping to
            //  the top of the text.
            if style.vertical_alignment == VerticalTextAlignment::End {
                editor.with_buffer_mut(|buffer| {
                    let mut scroll = buffer.scroll();
                    if scroll.vertical == 0.0 && buffer_inner_dimensions.y < text_area_size.y {
                        let vertical_scroll_to_align_text = calculate_vertical_offset(
                            style,
                            text_area_size,
                            buffer_inner_dimensions,
                        );
                        scroll.vertical = vertical_scroll_to_align_text;
                        buffer.set_scroll(scroll);
                    }
                });
            }
            editor.cursor_position().map(Point::from)
        }
    }
}
