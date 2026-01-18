use crate::byte_cursor::ByteCursor;
use crate::math::{Point, Rect, Size};
use crate::style::{FontFamily, TextStyle, TextWrap, VerticalTextAlignment};
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

/// Ensures the caret is vertically visible by adjusting buffer scroll using DEVICE pixels.
/// Returns caret top-left in LOGICAL pixels relative to the viewport.
pub(crate) fn adjust_vertical_scroll_to_make_caret_visible(
    buffer: &mut Buffer,
    current_char_byte_cursor: ByteCursor,
    font_system: &mut FontSystem,
    text_area_size: Size,
    style: &TextStyle,
    scale_factor: f32,
) -> Option<Point> {
    let mut editor = Editor::new(&mut *buffer);
    editor.set_cursor(current_char_byte_cursor.cursor);

    let caret_position = editor.cursor_position();

    match caret_position {
        Some(position) => {
            // caret position from cosmic_text is in DEVICE pixels
            let mut caret_top_left_corner = Point::from(position);
            let mut scroll = buffer.scroll();
            let scale = scale_factor.max(0.01);
            let line_height_device = style.line_height_pt() * scale;
            let text_area_height_device = text_area_size.y * scale;

            // If the caret is not fully visible, we need to scroll it into view
            if caret_top_left_corner.y < 0.0 {
                scroll.vertical += caret_top_left_corner.y;
                caret_top_left_corner.y = 0.0;
                buffer.set_scroll(scroll);
            } else if caret_top_left_corner.y + line_height_device > text_area_height_device {
                scroll.vertical +=
                    caret_top_left_corner.y + line_height_device - text_area_height_device;
                caret_top_left_corner.y = text_area_height_device - line_height_device;
                buffer.set_scroll(scroll);
            }
            // Convert caret position back to LOGICAL pixels for the API
            Some(Point::new(
                caret_top_left_corner.x / scale,
                caret_top_left_corner.y / scale,
            ))
        }
        None => {
            // Caret is not visible, we need to shape the text and move the scroll
            editor.shape_as_needed(font_system, false);

            // TODO: Let's keep it the code below for a little while, it might be useful in the
            //  future.

            // If it's not visible, and the scroll is already at the top, that means that we're
            //  at the end of the text, and we need to scroll to the bottom to avoid jumping to
            //  the top of the text.
            // if style.vertical_alignment == VerticalTextAlignment::End {
            //     editor.with_buffer_mut(|buffer| {
            //         let mut scroll = buffer.scroll();
            //         if scroll.vertical == 0.0 && buffer_inner_dimensions.y < text_area_size.y {
            //             let vertical_scroll_to_align_text = calculate_vertical_offset(
            //                 style,
            //                 text_area_size,
            //                 buffer_inner_dimensions,
            //             );
            //             scroll.vertical = vertical_scroll_to_align_text;
            //             buffer.set_scroll(scroll);
            //         }
            //     });
            // }
            // Return caret position in LOGICAL pixels
            editor.cursor_position().map(|p| {
                let p = Point::from(p);
                let scale = scale_factor.max(0.01);
                Point::new(p.x / scale, p.y / scale)
            })
        }
    }
}

/// Hit-test a character under a LOGICAL pixel coordinate, accounting for scroll and scale.
pub fn char_under_position(
    buffer: &Buffer,
    interaction_position_relative_to_element: Point,
    scale_factor: f32,
) -> Option<Cursor> {
    let horizontal_scroll_device = buffer.scroll().horizontal;
    let scale = scale_factor.max(0.01);
    let x_device = interaction_position_relative_to_element.x * scale + horizontal_scroll_device;
    let y_device = interaction_position_relative_to_element.y * scale;
    buffer.hit(x_device, y_device)
}

/// Returns inner buffer dimensions
pub(crate) fn update_buffer(
    params: &TextParams,
    buffer: &mut Buffer,
    font_system: &mut FontSystem,
    font_family: &FontFamily,
) -> Size {
    let text_style = &params.style();
    let font_color = text_style.font_color;
    let horizontal_alignment = text_style.horizontal_alignment;
    let wrap = text_style.wrap.unwrap_or_default();
    let text_area_size = params.size();
    let weight = text_style.weight;
    let letter_spacing = text_style.letter_spacing;
    let metadata = params.metadata();
    let old_scroll = buffer.scroll();

    let scale_factor = params.scale_factor();
    buffer.set_metrics(font_system, params.metrics());
    buffer.set_wrap(font_system, wrap.into());

    // Setting vertical size to None means that the buffer will use the height of the text.
    // This is needed to ensue that glyphs can be scrolled vertically by smaller amounts than
    // the line height.
    // Apply scale for shaping to device pixels
    buffer.set_size(font_system, Some(text_area_size.x * scale_factor), None);

    let mut attrs = Attrs::new()
        .color(font_color.into())
        .family(font_family.to_fontdb_family())
        .weight(weight.into())
        .metadata(metadata);

    if let Some(letter_spacing) = letter_spacing {
        attrs = attrs.letter_spacing(letter_spacing.0 * scale_factor);
    }

    buffer.set_text(
        font_system,
        params.text_for_internal_use(),
        &attrs,
        Shaping::Advanced,
        None,
    );

    let mut buffer_measurement = Size::default();
    for line in buffer.lines.iter_mut() {
        line.set_align(horizontal_alignment.into());
        for layout_line in line
            .layout(
                font_system,
                text_style.font_size.value() * scale_factor,
                Some(text_area_size.x * scale_factor),
                text_style.wrap.unwrap_or_default().into(),
                None,
                // TODO: what is the default tab width? Make it configurable?
                2,
                cosmic_text::Hinting::Enabled,
            )
            .iter()
        {
            let line_height = layout_line
                .line_height_opt
                .unwrap_or(text_style.line_height_pt() * scale_factor);
            buffer_measurement.y += line_height;
            buffer_measurement.x = buffer_measurement.x.max(layout_line.w);
        }
    }

    if buffer_measurement.x > text_area_size.x * scale_factor {
        #[cfg(test)]
        eprintln!(
            "RELAYOUT: buffer_measurement.x={}, text_area_size.x * scale_factor={}",
            buffer_measurement.x,
            text_area_size.x * scale_factor
        );
        // If the buffer is smaller than the text area, we need to set the width to the text area
        // size to ensure that the text is centered.
        // After we've measured the buffer, we need to run layout() again to realign the lines
        for line in buffer.lines.iter_mut() {
            line.reset_layout();
            line.set_align(horizontal_alignment.into());
            line.layout(
                font_system,
                text_style.font_size.value() * scale_factor,
                Some(buffer_measurement.x),
                wrap.into(),
                None,
                // TODO: what is the default tab width? Make it configurable?
                2,
                cosmic_text::Hinting::Enabled,
            );
        }
    }

    buffer.set_scroll(old_scroll);
    // We shaped at device pixels; convert inner_dimensions back to logical for API
    Size::from((
        buffer_measurement.x / scale_factor,
        buffer_measurement.y / scale_factor,
    ))
}
