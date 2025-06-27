use crate::byte_cursor::ByteCursor;
use crate::math::{Point, Rect, Size};
use crate::state::calculate_vertical_offset;
use crate::style::{TextStyle, TextWrap, VerticalTextAlignment};
use crate::text_params::TextParams;
use cosmic_text::{Attrs, Buffer, Cursor, Edit, Editor, FontSystem, Shaping};

impl From<TextWrap> for cosmic_text::Wrap {
    fn from(value: TextWrap) -> Self {
        match value {
            TextWrap::NoWrap => cosmic_text::Wrap::None,
            TextWrap::Wrap => cosmic_text::Wrap::Word,
            TextWrap::BreakWord => cosmic_text::Wrap::Glyph,
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
        VerticalTextAlignment::None => 0.0,
    }
}

pub(crate) fn adjust_vertical_scroll_to_make_caret_visible(
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

pub fn char_under_position(
    buffer: &Buffer,
    interaction_position_relative_to_element: Point,
) -> Option<Cursor> {
    let horizontal_scroll = buffer.scroll().horizontal;
    buffer.hit(
        interaction_position_relative_to_element.x + horizontal_scroll,
        interaction_position_relative_to_element.y,
    )
}

/// Returns inner buffer dimensions
pub(crate) fn update_buffer(
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
        params.text_for_internal_use(),
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
