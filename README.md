# Protextinator

[![Crates.io](https://img.shields.io/crates/v/protextinator.svg)](https://crates.io/crates/protextinator)
[![Docs.rs](https://img.shields.io/docsrs/protextinator)](https://docs.rs/protextinator/latest/protextinator/)
[![License](https://img.shields.io/crates/l/protextinator)](https://github.com/antouhou/protextinator/blob/main/LICENSE_MIT)

Protextinator is a text editing and rendering library for Rust, built on top of [cosmic_text](https://github.com/pop-os/cosmic-text). It provides a simpler API while adding features like vertical alignment, scroll position management, and text selection.

**Note:** This library is still a work in progress. APIs may change.

## Features

- Vertical text alignment
- Text buffer size measurement
- Scroll position management with absolute coordinates
- Simple font loading from files or embedded bytes
- Text state collection with optional usage tracking
- Custom metadata for text states
- Text selection and editing
- Efficient text buffer caching
- Word wrapping and text styling
- Optional serialization support

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
protextinator = "0.5.0"
```

With serialization:

```toml
[dependencies]
protextinator = { version = "0.1.0", features = ["serialization"] }
```

## Quick Start

```rust
use protextinator::{TextManager, TextState, math::Size};
use cosmic_text::{fontdb, Color};
use protextinator::style::TextStyle;

// Create a text manager
let mut text_manager = TextManager::new();

// Create a text state
let id = protextinator::Id::new("my_text");
let text = "Hello, world!";
text_manager.create_state(id, text, ());
// Add fonts
let font_sources: Vec<fontdb::Source> = vec![];
text_manager.load_fonts(font_sources.into_iter());
// Alternatively, you can load fonts from bytes if you want to embed them into the binary
// or download them at runtime as bytes
let byte_sources: Vec<&'static [u8]> = vec![];
text_manager.load_fonts_from_bytes(byte_sources.into_iter());

// Optional: Marks the beginning of a frame so that you can track which text states are accessed
text_manager.start_frame();

// Configure the text area size and style
if let Some(state) = text_manager.text_states.get_mut(&id) {
    state.set_outer_size(&Size::new(400.0, 200.0));
    
    let style = TextStyle::new(16.0, Color::rgb(255, 255, 255))
        .with_line_height(1.5);
    state.set_style(&style);
    
    // Enable editing
    state.is_editable = true;
    state.is_selectable = true;
    state.are_actions_enabled = true;
    
    // Recalculate layout
    state.recalculate(&mut text_manager.text_context);

    // Get the inner size of the buffer - i.e., how much space the text needs to occupy
    let inner_size = state.inner_size();
}

let mut remove_ids = vec![];
// Optional: going to remove all states that were not accessed during the current frame
text_manager.end_frame(&mut remove_ids);
```

For a complete example, see the [examples directory](https://github.com/antouhou/protextinator/tree/main/examples).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](http://www.apache.org/licenses/LICENSE-2.0))
- MIT license ([LICENSE-MIT](LICENSE_MIT) or http://opensource.org/licenses/MIT)

at your option.
