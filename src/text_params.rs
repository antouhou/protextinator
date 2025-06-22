use crate::math::Size;
use crate::state::SIZE_EPSILON;
use crate::{Id, TextStyle};
use cosmic_text::Metrics;

#[derive(Clone, Debug, PartialEq)]
pub struct TextParams {
    size: Size,
    style: TextStyle,
    text: String,
    buffer_id: Id,

    changed: bool,
}

impl TextParams {
    #[inline(always)]
    pub fn new(size: Size, style: TextStyle, text: String, buffer_id: Id) -> Self {
        Self {
            size,
            style,
            text,
            buffer_id,

            changed: true,
        }
    }

    /// Updates the text parameters with new values if they differ from the current ones and
    /// marks the parameters as changed if any of the values changed.
    #[inline(always)]
    pub fn update(&mut self, size: &Size, style: &TextStyle, text: &str, buffer_id: &Id) {
        self.set_size(size);
        self.set_style(style);
        self.set_text(text);
        self.set_buffer_id(buffer_id);
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
    pub fn text(&self) -> &str {
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
    pub fn buffer_id(&self) -> Id {
        self.buffer_id
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
    pub fn set_text(&mut self, text: &str) {
        if self.text != text {
            self.text = text.into();
            if !self.text.ends_with('\n') {
                // Ensure the text always ends with a line terminator. If the text does not end with
                // a newline, you'll need to add two newline characters to insert a new line at the
                // end of the text.
                self.text.push('\n');
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
        if &self.style != style {
            self.style = style.clone();
            self.changed = true;
        }
    }

    #[inline(always)]
    pub fn set_buffer_id(&mut self, buffer_id: &Id) {
        if &self.buffer_id != buffer_id {
            self.buffer_id = *buffer_id;
            self.changed = true;
        }
    }

    pub fn metrics(&self) -> Metrics {
        Metrics::new(self.style().font_size.0, self.style().line_height_pt())
    }
}
