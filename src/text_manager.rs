use crate::state::TextState;
use crate::{BufferCache, Id};
use ahash::HashMap;
use cosmic_text::{fontdb, FontSystem, SwashCache};
use std::sync::Arc;

pub struct TextContext {
    pub buffer_cache: BufferCache,
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
}

impl Default for TextContext {
    fn default() -> Self {
        Self {
            buffer_cache: BufferCache::default(),
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
        }
    }
}

#[derive(Default)]
pub struct TextManager {
    pub text_states: HashMap<Id, TextState>,
    pub text_context: TextContext,
}

impl TextManager {
    pub fn new() -> Self {
        Self {
            text_states: HashMap::default(),
            text_context: TextContext::default(),
        }
    }

    pub fn load_fonts(&mut self, fonts: impl Iterator<Item = fontdb::Source>) {
        self.text_context.load_fonts(fonts);
    }

    pub fn load_fonts_from_bytes<'a>(&mut self, fonts: impl Iterator<Item = &'a [u8]>) {
        self.text_context.load_fonts_from_bytes(fonts);
    }
}

impl TextContext {
    pub fn load_fonts(&mut self, fonts: impl Iterator<Item = fontdb::Source>) {
        let db = self.font_system.db_mut();

        for source in fonts {
            db.load_font_source(source);
        }
    }

    pub fn load_fonts_from_bytes<'a>(&mut self, fonts: impl Iterator<Item = &'a [u8]>) {
        let db = self.font_system.db_mut();

        for font_bytes in fonts {
            let source = fontdb::Source::Binary(Arc::new(font_bytes.to_vec()));
            db.load_font_source(source);
        }
    }
}
