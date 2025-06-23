mod action;
mod buffer_utils;
mod byte_cursor;
mod id;
pub mod math;
mod state;
mod style;
#[cfg(test)]
mod tests;
mod text_manager;
mod text_params;
pub mod utils;

pub use action::{Action, ActionResult};
pub use cosmic_text;
pub use id::Id;
pub use math::{Point, Rect};
pub use state::{Selection, SelectionLine, TextState};
pub use style::*;
pub use text_manager::{TextContext, TextManager};
pub use text_params::TextParams;
