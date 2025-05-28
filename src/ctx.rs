use crate::state::{TextState};
use crate::{Id, Point, TextManager};
use ahash::HashMap;
use cosmic_text::{fontdb, FontSystem};
use std::sync::Arc;

pub struct TextContext {
    pub text_manager: TextManager,
    pub font_system: FontSystem,
}

impl Default for TextContext {
    fn default() -> Self {
        Self {
            text_manager: TextManager::default(),
            font_system: FontSystem::new(),
        }
    }
}

#[derive(Default)]
pub struct Kek {
    pub text_states: HashMap<Id, TextState>,
    pub text_context: TextContext,
}

impl Kek {
    pub fn handle_click(&mut self, text_id: Id, click_position_relative: impl Into<Point>) {
        if let Some(state) = self.text_states.get_mut(&text_id) {
            let text_manager = &mut self.text_context;
            state.handle_click(text_manager, click_position_relative.into());
            state.is_focused = true;
        } else {
            //TODO: print warning
        }
    }

    pub fn handle_drag(
        &mut self,
        text_id: Id,
        drag_position_relative: impl Into<Point>,
        drag_position_absolute: impl Into<Point>,
        is_dragging: bool,
    ) -> Option<()> {
        if let Some(state) = self.text_states.get_mut(&text_id) {
            let text_manager = &mut self.text_context;
            state.handle_drag(
                text_manager,
                is_dragging,
                drag_position_relative.into(),
                drag_position_absolute.into(),
            );
        } else {
            // TODO: print warning
        }
        
        None
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
