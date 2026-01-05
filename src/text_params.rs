use crate::math::Size;
use crate::state::SIZE_EPSILON;
use crate::style::TextStyle;
use cosmic_text::Metrics;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextParams {
    size: Size,
    style: TextStyle,
    text: String,
    metadata: usize,

    // Device scale factor; 1.0 == logical pixels
    scale_factor: f32,

    changed: bool,
    font_query_changed: bool,
    line_terminator_has_been_added: bool,
}

impl TextParams {
    #[inline(always)]
    pub fn new(size: Size, style: TextStyle, text: String, metadata: usize) -> Self {
        let mut params = Self {
            size,
            style,
            text: "".to_string(),
            metadata,
            scale_factor: 1.0,

            changed: true,
            font_query_changed: true,
            line_terminator_has_been_added: false,
        };

        params.set_text(&text);
        params
    }

    #[inline(always)]
    pub fn size(&self) -> Size {
        self.size
    }

    #[inline(always)]
    pub fn style(&self) -> &TextStyle {
        &self.style
    }

    #[inline(always)]
    pub fn original_text(&self) -> &str {
        if self.line_terminator_has_been_added {
            // If the line terminator was added by the set_text method, remove it to restore the
            // original text.
            &self.text[..self.text.len().saturating_sub(1)]
        } else {
            // Otherwise, return the text as is.
            &self.text
        }
    }

    #[inline(always)]
    pub fn text_for_internal_use(&self) -> &str {
        &self.text
    }

    #[inline(always)]
    pub fn insert_char(&mut self, index: usize, c: char) {
        if index <= self.text.len() {
            self.text.insert(index, c);
            self.changed = true;
        }
    }

    #[inline(always)]
    pub fn insert_str(&mut self, index: usize, s: &str) {
        if index <= self.text.len() {
            self.text.insert_str(index, s);
            self.changed = true;
        }
    }

    #[inline(always)]
    pub fn remove_char(&mut self, index: usize) -> Option<char> {
        if index < self.text.len() {
            let char = self.text.remove(index);
            self.changed = true;
            Some(char)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn remove_range(&mut self, start: usize, end: usize) {
        if start < end && end <= self.text.len() {
            self.text.drain(start..end);
            self.changed = true;
        }
    }

    #[inline(always)]
    pub fn metadata(&self) -> usize {
        self.metadata
    }

    #[inline(always)]
    pub fn changed_since_last_shape(&self) -> bool {
        self.changed
    }

    #[inline(always)]
    pub fn reset_changed(&mut self) {
        self.changed = false;
    }

    #[inline(always)]
    pub fn font_query_changed_since_last_shape(&self) -> bool {
        self.font_query_changed
    }

    #[inline(always)]
    pub fn reset_font_query_changed(&mut self) {
        self.font_query_changed = false;
    }

    #[inline(always)]
    pub fn set_text(&mut self, text: &str) {
        if self.original_text() != text {
            self.text = text.into();
            // TODO: move that to the insertion of the newline - this will cause shenanigans
            //  if text is one line right now, but you intend to add a newline later
            let is_one_line = !self.text.contains('\n');
            let ends_with_line_terminator = !self.text.ends_with('\n');
            if !is_one_line && !ends_with_line_terminator {
                // Ensure the text always ends with a line terminator. If the text does not end with
                // a newline, you'll need to add two newline characters to insert a new line at the
                // end of the text.
                self.text.push('\n');
                self.line_terminator_has_been_added = true;
            } else {
                self.line_terminator_has_been_added = false;
            }
            self.changed = true;
        }
    }

    #[inline(always)]
    pub fn set_size(&mut self, size: &Size) {
        if !self.size.approx_eq(size, SIZE_EPSILON) {
            self.size = *size;
            self.changed = true;
        }
    }

    #[inline(always)]
    pub fn set_style(&mut self, style: &TextStyle) {
        let font_query_changed =
            self.style.font_family != style.font_family || self.style.weight != style.weight;
        if font_query_changed {
            self.font_query_changed = true;
        }
        if &self.style != style {
            self.style = style.clone();
            self.changed = true;
        }
    }

    #[inline(always)]
    pub fn set_metadata(&mut self, metadata: usize) {
        if self.metadata != metadata {
            self.metadata = metadata;
            self.changed = true;
        }
    }

    #[inline(always)]
    pub fn metrics(&self) -> Metrics {
        let scale = self.scale_factor;
        let font_size = self.style().font_size.0 * scale;
        let line_height = self.style().line_height_pt() * scale;
        Metrics::new(font_size, line_height)
    }

    #[inline(always)]
    pub fn set_scale_factor(&mut self, scale: f32) {
        let scale = scale.max(0.01);
        if (self.scale_factor - scale).abs() > SIZE_EPSILON {
            self.scale_factor = scale;
            self.changed = true;
        }
    }

    #[inline(always)]
    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }
}
