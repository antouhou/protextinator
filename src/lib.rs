//! # Protextinator
//!
//! Text editing and rendering library built on top of [`cosmic_text`], that provides a simpler
//! API with additional features, such as:
//!
//! - Vertical text alignment
//! - Measuring text buffer size
//! - Managing scroll position with absolute coordinates
//! - A simple interface for loading and managing fonts
//! - A collection of text states that has optional track of usage for garbage collection
//! - Custom metadata for text states
//!   
//! and more.
//!
//! ## Basic Usage
//!
//! ```rust
//! use protextinator::{TextManager, TextState, math::Size};
//! use cosmic_text::{fontdb, Color};
//! use protextinator::style::TextStyle;
//!
//! // Create a text manager
//! let mut text_manager = TextManager::new();
//!
//! // Create a text state
//! let id = protextinator::Id::new("my_text");
//! let text = "Hello, world!";
//! text_manager.create_state(id, text, ());
//! // Add fonts
//! let font_sources: Vec<fontdb::Source> = vec![];
//! text_manager.load_fonts(font_sources.into_iter());
//! // Alternatively, you can load fonts from bytes if you want to embed them into the binary
//! // or download them at runtime as bytes
//! let byte_sources: Vec<&'static [u8]> = vec![];
//! text_manager.load_fonts_from_bytes(byte_sources.into_iter());
//!
//! // Optional: Marks the beginning of a frame so that you can track which text states are accessed
//! text_manager.start_frame();
//!
//! // Configure the text area size and style
//! if let Some(state) = text_manager.text_states.get_mut(&id) {
//!     state.set_outer_size(&Size::new(400.0, 200.0));
//!     
//!     let style = TextStyle::new(16.0, Color::rgb(255, 255, 255))
//!         .with_line_height(1.5);
//!     state.set_style(&style);
//!     
//!     // Enable editing
//!     state.is_editable = true;
//!     state.is_selectable = true;
//!     state.are_actions_enabled = true;
//!     
//!     // Recalculate layout
//!     state.recalculate(&mut text_manager.text_context);
//!
//!     // Get the inner size of the buffer - i.e., how much space the text needs to occupy
//!     let inner_size = state.inner_size();
//! }
//!
//! // Optional: going to remove all states that were not accessed during the current frame
//! text_manager.end_frame();
//! ```

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
pub use state::{RasterizedTexture, Selection, SelectionLine, TextState};
pub use text_manager::{TextContext, TextManager};
