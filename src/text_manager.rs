use crate::state::TextState;
use crate::Id;
use ahash::{HashMap, HashSet, HashSetExt};
use cosmic_text::{fontdb, FontSystem, SwashCache};
use std::sync::Arc;

pub struct TextContext {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub usage_tracker: TextUsageTracker,
}

impl Default for TextContext {
    fn default() -> Self {
        Self {
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
            usage_tracker: TextUsageTracker::new(),
        }
    }
}

#[derive(Default)]
pub struct TextManager<TMetadata = ()> {
    pub text_states: HashMap<Id, TextState<TMetadata>>,
    pub text_context: TextContext,
}

impl<TMetadata> TextManager<TMetadata> {
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

    pub fn create_state(&mut self, id: Id, text: impl Into<String>, metadata: TMetadata) {
        let state = TextState::new_with_text(text, &mut self.text_context.font_system, metadata);
        self.text_states.insert(id, state);
    }

    /// Utility to do some simple garbage collection of text states if you don't want
    /// to implement a usage tracker yourself. Call this at the start of each frame.
    pub fn start_frame(&mut self) {
        self.text_context.usage_tracker.clear();
    }

    /// Utility to do some simple garbage collection of text states if you don't want
    /// to implement a usage tracker yourself. Call this at the end of each frame, and this will
    /// remove any text states not marked as accessed since the last call to `start_frame`.
    pub fn end_frame(&mut self) {
        let accessed_states = self.text_context.usage_tracker.accessed_states();
        self.text_states
            .retain(|id, _| accessed_states.contains(id));
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

pub struct TextUsageTracker {
    accessed_states: HashSet<Id>,
}

impl Default for TextUsageTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl TextUsageTracker {
    pub fn new() -> Self {
        Self {
            accessed_states: HashSet::new(),
        }
    }

    pub fn mark_accessed(&mut self, id: Id) {
        self.accessed_states.insert(id);
    }

    pub fn clear(&mut self) {
        self.accessed_states.clear();
    }

    pub fn accessed_states(&self) -> &HashSet<Id> {
        &self.accessed_states
    }
}
