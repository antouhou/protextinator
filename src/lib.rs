mod action;
mod buffer_utils;
mod byte_cursor;
mod id;
pub mod math;
mod state;
pub mod style;
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
pub use text_manager::{TextContext, TextManager};
