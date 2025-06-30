# 🚀 Protextinator

<div align="center">
  <a href="https://crates.io/crates/protextinator"><img src="https://img.shields.io/crates/v/protextinator.svg" alt="Crates.io"></a>
  <a href="https://docs.rs/protextinator/latest/protextinator/"><img src="https://img.shields.io/docsrs/protextinator" alt="Docs.rs"></a>
  <a href="https://github.com/antouhou/protextinator/blob/main/LICENSE_MIT"><img src="https://img.shields.io/crates/l/protextinator" alt="License"></a>
</div>

## ✨ Text Management, Made Simple!

**Protextinator** is a powerful text editing and rendering library built on top of [cosmic_text](https://github.com/pop-os/cosmic-text), providing a simpler API with advanced features for all your text handling needs!

> 💡 Perfect for game UIs, text editors, and any application that needs sophisticated text rendering with minimal hassle.

⚠️ **WARNING**:  This library is work in progress, use at your own risk! ⚠️

## 🔥 Features

- **Vertical text alignment** - Position your text exactly where you want it
- **Text buffer size measurement** - Know exactly how much space your text needs
- **Scroll position management** with absolute coordinates
- **Simple font loading interface** - Load fonts from files or embedded bytes
- **Text state collection** with optional usage tracking for garbage collection
- **Custom metadata** for text states
- **Text selection and editing** capabilities
- **Efficient text buffer caching**
- **Word wrapping and text styling**
- **Optional serialization** support via the `serialization` feature

## 📦 Installation

Add Protextinator to your `Cargo.toml`:

```toml
[dependencies]
protextinator = "0.1.0"
```

With serialization support:

```toml
[dependencies]
protextinator = { version = "0.1.0", features = ["serialization"] }
```

## 🚀 Quick Start

For code examples and detailed usage, check out:
- [Documentation on docs.rs](https://docs.rs/protextinator/)
- [Example code in the repository](https://github.com/antouhou/protextinator/tree/main/examples)

Protextinator makes it easy to:
1. Create and manage text states
2. Style text with various fonts, colors, and alignments
3. Handle text selection and editing
4. Efficiently render text in your application

## 🎮 Integration Example

Protextinator works great with rendering libraries like [Grafo](https://github.com/antouhou/grafo) and windowing libraries like [Winit](https://github.com/rust-windowing/winit). Check out the examples directory for a complete integration example.

## 📚 API Overview

- **TextManager**: The main entry point for managing text states and fonts
- **TextState**: Represents a text buffer with styling and layout information
- **Id**: A unique identifier for text states
- **TextStyle**: Configure font size, color, alignment, and more
- **Action**: Perform operations like copy, paste, and cursor movement

## 🔧 Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.

## 📄 License

Licensed under either of:

- Apache License, Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE_MIT) or http://opensource.org/licenses/MIT)

at your option.
