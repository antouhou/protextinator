//! # Protextinator
//!
//! A high-performance text editing and rendering library built on top of [`cosmic_text`].
//! Protextinator provides text state management, cursor handling, text selection, 
//! and various text editing operations with support for rich text styling.
//!
//! ## Features
//!
//! - **Text State Management**: Efficient text buffer management with cursor positioning
//! - **Text Selection**: Support for text selection with visual feedback
//! - **Text Editing**: Insert, delete, copy, paste, and other text editing operations
//! - **Rich Styling**: Comprehensive text styling with fonts, colors, alignment, and more
//! - **Scrolling**: Automatic scroll management to keep cursor visible
//! - **Font Management**: Easy font loading and management
//! - **Serialization**: Optional serialization support for text styles
//!
//! ## Basic Usage
//!
//! ```rust
//! use protextinator::{TextManager, TextState, TextStyle, math::Size};
//! use cosmic_text::Color;
//!
//! // Create a text manager
//! let mut text_manager = TextManager::new();
//!
//! // Create a text state
//! let id = protextinator::Id::new("my_text");
//! let text = "Hello, world!";
//! text_manager.create_state(id, text, ());
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
//! }
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
pub use state::{Selection, SelectionLine, TextState};
pub use text_manager::{TextContext, TextManager};
