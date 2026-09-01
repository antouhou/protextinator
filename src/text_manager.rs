//! Text state management and font loading.
//!
//! Manages multiple text states, their fonts, and usage tracking.

use crate::font_family_query::FontFamilyCache;
use crate::state::{AlphaMode, TextState};
use crate::Id;
use ahash::{HashMap, HashSet, HashSetExt};
use cosmic_text::{fontdb, FontSystem, SwashCache};
use std::sync::Arc;

fn create_system_font_system() -> FontSystem {
    let locale = sys_locale::get_locale().unwrap_or_else(|| String::from("en-US"));
    let mut font_database = fontdb::Database::new();
    font_database.load_system_fonts();

    // FontSystem::new replaces generic families after loading the database.
    FontSystem::new_with_locale_and_db(locale, font_database)
}

/// Shared context for text rendering operations.
///
/// Holds the font system, glyph cache, and usage tracking shared by all text states.
pub struct TextContext {
    /// The cosmic-text font system for text layout and rendering.
    pub font_system: FontSystem,
    /// Cache for rendered glyphs.
    pub swash_cache: SwashCache,
    /// Current device scale factor. 1.0 means logical pixels; >1.0 means HiDPI.
    pub scale_factor: f32,
    /// Tracks which text states are being used for garbage collection.
    pub usage_tracker: TextUsageTracker,
    /// Cache for resolved font family queries.
    pub font_family_cache: FontFamilyCache,
}

impl Default for TextContext {
    /// Creates a default text context with initialized font system and caches.
    fn default() -> Self {
        Self {
            font_system: create_system_font_system(),
            swash_cache: SwashCache::new(),
            scale_factor: 1.0,
            usage_tracker: TextUsageTracker::new(),
            font_family_cache: FontFamilyCache::new(),
        }
    }
}

/// Manages multiple text states and the resources they share.
///
/// Handles text state creation, font loading, and optional garbage collection
/// of unused text states.
///
/// # Type Parameters
/// * `TMetadata` - Custom metadata type that can be attached to each text state
#[derive(Default)]
pub struct TextManager<TMetadata = ()> {
    /// Map of text state IDs to their corresponding text states.
    pub text_states: HashMap<Id, TextState<TMetadata>>,
    /// Shared context for text rendering operations.
    pub text_context: TextContext,
}

impl<TMetadata> TextManager<TMetadata> {
    /// Creates a new text manager with empty state.
    ///
    /// # Examples
    /// ```
    /// use protextinator::TextManager;
    ///
    /// let mut manager: TextManager<()> = TextManager::new();
    /// ```
    pub fn new() -> Self {
        Self {
            text_states: HashMap::default(),
            text_context: TextContext::default(),
        }
    }

    /// Loads fonts from the provided sources into the font system.
    ///
    /// # Arguments
    /// * `fonts` - Iterator of font sources to load
    ///
    /// # Examples
    /// ```
    /// use protextinator::TextManager;
    /// use cosmic_text::fontdb;
    ///
    /// let mut manager: TextManager<()> = TextManager::new();
    ///
    /// // Load system fonts (example)
    /// let sources = std::iter::empty(); // In practice, use actual font sources
    /// manager.load_fonts(sources);
    /// ```
    pub fn load_fonts(&mut self, fonts: impl Iterator<Item = fontdb::Source>) {
        self.text_context.load_fonts(fonts);
    }

    /// Loads fonts from byte slices into the font system.
    ///
    /// Use this to embed fonts in your application.
    ///
    /// # Arguments
    /// * `fonts` - Iterator of byte slices containing font data
    ///
    /// # Examples
    /// ```
    /// use protextinator::TextManager;
    ///
    /// let mut manager: TextManager<()> = TextManager::new();
    ///
    /// // Load embedded font data
    /// //let font_data = include_bytes!("../path/to/font.ttf");
    /// //manager.load_fonts_from_bytes(std::iter::once(font_data.as_slice()));
    /// ```
    pub fn load_fonts_from_bytes<'a>(&mut self, fonts: impl Iterator<Item = &'a [u8]>) {
        self.text_context.load_fonts_from_bytes(fonts);
    }

    /// Creates a new text state with the given ID, text content, and metadata.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the text state
    /// * `text` - Initial text content
    /// * `metadata` - Custom metadata to associate with the text state
    ///
    /// # Examples
    /// ```
    /// use protextinator::{TextManager, Id};
    ///
    /// let mut manager: TextManager<&str> = TextManager::new();
    /// let id = Id::new("my_text");
    ///
    /// manager.create_state(id, "Hello, world!", "label_text");
    /// ```
    pub fn create_state(&mut self, id: Id, text: impl Into<String>, metadata: TMetadata) {
        let state = TextState::new_with_text(text, &mut self.text_context.font_system, metadata);
        self.text_states.insert(id, state);
    }

    /// Call at the start of each frame to clear the usage tracker. Together with
    /// [`Self::end_frame`], this gives you simple garbage collection of text states
    /// if you don't want to implement usage tracking yourself.
    ///
    /// # Examples
    /// ```
    /// use protextinator::TextManager;
    ///
    /// let mut manager: TextManager<()> = TextManager::new();
    ///
    /// // At the start of each frame
    /// manager.start_frame();
    /// ```
    pub fn start_frame(&mut self) {
        self.text_context.usage_tracker.clear();
    }

    /// Call at the end of each frame to remove any text states not accessed since
    /// the last [`Self::start_frame`] call. Appends the IDs of removed states to
    /// `removed_ids`.
    ///
    /// # Examples
    /// ```
    /// use protextinator::TextManager;
    ///
    /// let mut manager: TextManager<()> = TextManager::new();
    ///
    /// let mut removed_ids = Vec::new();
    /// // At the end of each frame
    /// manager.end_frame(&mut removed_ids);
    /// ```
    pub fn end_frame(&mut self, removed_ids: &mut Vec<Id>) {
        let accessed_states = self.text_context.usage_tracker.accessed_states();
        self.text_states.retain(|id, _| {
            let accessed = accessed_states.contains(id);
            if !accessed {
                removed_ids.push(*id);
            }
            accessed
        });
    }

    /// Sets the global scale factor used for shaping and rasterization.
    /// This keeps `FontSize` and sizes in logical pixels while shaping in device pixels.
    /// Call this when the window scale factor changes.
    pub fn set_scale_factor(&mut self, scale: f32) {
        let scale = scale.max(0.01);
        if (self.text_context.scale_factor - scale).abs() < 0.0001 {
            return;
        }
        self.text_context.scale_factor = scale;
        // Update each state's params with new scale; they'll mark themselves changed.
        for state in self.text_states.values_mut() {
            // This will mark params changed if different and reshape on next recalc
            state.set_scale_factor(scale);
        }
    }

    /// Rasterizes all text states into CPU-side RGBA textures stored on the states.
    ///
    /// Recalculates shaping and layout first if needed. Runs on a single thread.
    pub fn rasterize_all_textures(&mut self, alpha_mode: AlphaMode) -> Vec<RasterizedTextureInfo> {
        // In the future this can be parallelized by splitting the states into chunks and
        // creating per-thread SwashCache/FontSystem references as needed.
        let mut changes = Vec::new();
        for (id, state) in self.text_states.iter_mut() {
            let old_w = state.rasterized_texture().width;
            let old_h = state.rasterized_texture().height;
            // Ensure the buffer is up to date
            state.recalculate(&mut self.text_context);
            // Rasterize into the state's texture storage
            let rerasterized = state.rasterize_into_texture(&mut self.text_context, alpha_mode);
            if rerasterized {
                let new_w = state.rasterized_texture().width;
                let new_h = state.rasterized_texture().height;
                let resized = new_w != old_w || new_h != old_h;
                changes.push(RasterizedTextureInfo {
                    id: *id,
                    width: new_w,
                    height: new_h,
                    resized,
                });
            }
        }
        changes
    }
}

/// Information about a text state's rasterized texture after `rasterize_all_textures`.
#[derive(Debug, Clone, Copy)]
pub struct RasterizedTextureInfo {
    pub id: Id,
    pub width: u32,
    pub height: u32,
    /// True if the texture dimensions changed compared to the previous rasterization.
    pub resized: bool,
}

impl TextContext {
    /// Loads fonts from the provided sources into the font database.
    ///
    /// # Arguments
    /// * `fonts` - Iterator of font sources to load
    pub fn load_fonts(&mut self, fonts: impl Iterator<Item = fontdb::Source>) {
        let db = self.font_system.db_mut();

        for source in fonts {
            db.load_font_source(source);
        }
    }

    /// Loads fonts from byte slices into the font database.
    ///
    /// This creates `fontdb::Source::Binary` sources from the provided byte data.
    ///
    /// # Arguments
    /// * `fonts` - Iterator of byte slices containing font data
    pub fn load_fonts_from_bytes<'a>(&mut self, fonts: impl Iterator<Item = &'a [u8]>) {
        let db = self.font_system.db_mut();

        for font_bytes in fonts {
            let source = fontdb::Source::Binary(Arc::new(font_bytes.to_vec()));
            db.load_font_source(source);
        }
    }
}

/// Tracks which text states have been accessed during the current frame.
///
/// [`TextManager`] uses this to drop text states that are no longer used.
pub struct TextUsageTracker {
    accessed_states: HashSet<Id>,
}

impl Default for TextUsageTracker {
    /// Creates a new empty usage tracker.
    fn default() -> Self {
        Self::new()
    }
}

impl TextUsageTracker {
    /// Creates a new usage tracker with no accessed states.
    pub fn new() -> Self {
        Self {
            accessed_states: HashSet::new(),
        }
    }

    /// Marks a text state as accessed during the current frame.
    ///
    /// # Arguments
    /// * `id` - The ID of the text state that was accessed
    pub fn mark_accessed(&mut self, id: Id) {
        self.accessed_states.insert(id);
    }

    /// Clears the set of accessed states. Call at the start of each frame.
    pub fn clear(&mut self) {
        self.accessed_states.clear();
    }

    /// Returns the set of text state IDs that have been accessed.
    pub fn accessed_states(&self) -> &HashSet<Id> {
        &self.accessed_states
    }
}
