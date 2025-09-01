use futures::executor::block_on;
use grafo::{Color, MathRect, Renderer, Shape, Stroke};
use protextinator::style::{
    FontColor, FontFamily, FontSize, HorizontalTextAlignment, LineHeight, TextStyle, TextWrap,
    VerticalTextAlignment,
};
use protextinator::{cosmic_text::FontSystem, Id, Point, Rect, TextManager};
use std::sync::Arc;
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

/// Main application state
struct App<'a> {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer<'a>>,
    font_system: Option<FontSystem>,
    text_content: String,
    cursor_position: usize,
    text_manager: TextManager,
    // Texture id for the rendered text for the renderer
    text_texture_id: u64,
    // Track allocated texture size to avoid reallocating each frame
    text_texture_dimenstions: Option<(u32, u32)>,
}

impl<'a> Default for App<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> App<'a> {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            font_system: None,
            text_content: "Welcome to Protextinator!\n\nThis example demonstrates the integration of:\n• Protextinator - for advanced text management and caching\n• Grafo 0.6 - for GPU-accelerated rendering\n• Winit 0.30 - for cross-platform windowing\n\nKey features being showcased:\n✓ Text shaping and layout via cosmic-text\n✓ Efficient text buffer caching\n✓ Direct buffer rendering with add_text_buffer()\n✓ Real-time text editing and reshaping\n✓ Word wrapping and text styling\n\nTry typing to see the text management in action!\nNotice how protextinator efficiently caches and manages the text buffers.".to_string(),
            cursor_position: 0,
            text_manager: TextManager::new(),
            text_texture_id: 123,
            text_texture_dimenstions: None,
        }
    }
    fn setup_renderer(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Protextinator + Winit 0.30 + Grafo 0.6 Example")
                        .with_inner_size(winit::dpi::LogicalSize::new(900, 700)),
                )
                .unwrap(),
        );

        let window_size = window.inner_size();
        let scale_factor = window.scale_factor();
        let physical_size = (window_size.width, window_size.height);

        // Initialize the renderer
        let renderer = block_on(Renderer::new(
            window.clone(),
            physical_size,
            scale_factor,
            true,  // vsync
            false, // transparent
        ));

        // Renderer receives the initial scale factor at creation time.

        // Initialize text systems
        let font_system = FontSystem::new();

        self.window = Some(window);
        self.renderer = Some(renderer);
        self.font_system = Some(font_system);
        // Ensure text manager uses the same scale factor for shaping
        if let Some(r) = self.renderer.as_ref() {
            self.text_manager.set_scale_factor(r.scale_factor() as f32);
        }
    }

    fn handle_text_input(&mut self, text: &str) {
        // Insert text at cursor position
        self.text_content.insert_str(self.cursor_position, text);
        self.cursor_position += text.len();
    }

    fn handle_backspace(&mut self) {
        if self.cursor_position > 0 {
            // Find the previous character boundary
            let mut new_pos = self.cursor_position;
            while new_pos > 0 {
                new_pos -= 1;
                if self.text_content.is_char_boundary(new_pos) {
                    break;
                }
            }
            self.text_content.remove(new_pos);
            self.cursor_position = new_pos;
        }
    }

    fn render_frame(&mut self) {
        if let Some(renderer) = self.renderer.as_mut() {
            // Clear previous frame
            renderer.clear_draw_queue();

            // Background rectangle
            let background = Shape::rect(
                [(0.0, 0.0), (900.0, 700.0)],
                Color::rgb(30, 34, 42), // Dark background
                Stroke::new(0.0, Color::TRANSPARENT),
            );
            renderer.add_shape(background, None, (0.0, 0.0), None);

            // Text area background
            let text_bg = Shape::rect(
                [(40.0, 40.0), (860.0, 650.0)],
                Color::rgb(40, 44, 52), // Slightly lighter background
                Stroke::new(2.0, Color::rgb(97, 175, 239)),
            );
            renderer.add_shape(text_bg, None, (0.0, 0.0), None);

            // Main text content using protextinator for text management
            let text_id = Id::new("main_text");
            let text_rect = Rect::new(Point { x: 60.0, y: 60.0 }, Point { x: 840.0, y: 630.0 });

            let text_style = TextStyle {
                font_size: FontSize(18.0),
                line_height: LineHeight(1.5),
                font_color: FontColor(protextinator::cosmic_text::Color::rgb(0xE5, 0xE5, 0xE5)), // Light gray
                horizontal_alignment: HorizontalTextAlignment::Start,
                vertical_alignment: VerticalTextAlignment::Start,
                wrap: Some(TextWrap::Wrap),
                font_family: FontFamily::SansSerif,
            };

            // Create or update the text state
            if !self.text_manager.text_states.contains_key(&text_id) {
                self.text_manager
                    .create_state(text_id, self.text_content.clone(), ());
            }

            // Get the text state and reshape if needed
            if let Some(text_state) = self.text_manager.text_states.get_mut(&text_id) {
                text_state.set_text(&self.text_content);

                // Keep font sizes and outer sizes in logical pixels. We pass scale to the manager instead.
                text_state.set_outer_size(&text_rect.size().into());
                text_state.set_style(&text_style);
                text_state.set_buffer_metadata(text_id.0 as usize);
                text_state.recalculate(&mut self.text_manager.text_context);

                // Now here's the key part: use protextinator's buffer with grafo's add_text_buffer!
                let _buffer = &text_state.buffer();
                // Define the area where the text should be rendered
                let text_area = MathRect {
                    min: (text_rect.min.x, text_rect.min.y).into(),
                    max: (text_rect.max.x, text_rect.max.y).into(),
                };

                let text_area_size = text_area.size();
                let scale_factor = renderer.scale_factor() as f32;
                // Use device-pixel dimensions for the texture to avoid blurriness on HiDPI
                let texture_dimensions = (
                    (text_area_size.width * scale_factor).ceil() as u32,
                    (text_area_size.height * scale_factor).ceil() as u32,
                );

                // Allocate or reallocate the texture only when size changes
                if self.text_texture_dimenstions != Some(texture_dimensions) {
                    renderer
                        .texture_manager()
                        .allocate_texture(self.text_texture_id, texture_dimensions);
                    self.text_texture_dimenstions = Some(texture_dimensions);
                }

                // Rasterize into a CPU buffer every frame (to measure rasterization cost)
                let mut texture =
                    vec![0u8; (texture_dimensions.0 * texture_dimensions.1 * 4) as usize];

                let t_raster_start = Instant::now();
                rasterize_text_into_pixels(
                    self.font_system.as_mut().unwrap(),
                    &mut self.text_manager.text_context.swash_cache,
                    // Buffer is already shaped in device pixels; no draw-time scaling
                    1.0,
                    (text_area_size.width, text_area_size.height),
                    &mut texture,
                    texture_dimensions,
                    &text_state.buffer(),
                );
                let raster_time = t_raster_start.elapsed();

                let t_upload_start = Instant::now();
                match renderer.texture_manager().load_data_into_texture(
                    self.text_texture_id,
                    texture_dimensions,
                    &texture,
                ) {
                    Ok(_) => {}
                    Err(err) => eprintln!("Failed to load text texture data: {err:?}"),
                }
                let upload_time = t_upload_start.elapsed();

                println!(
                    "rasterize: {} µs, load_texture: {} µs",
                    raster_time.as_micros(),
                    upload_time.as_micros()
                );

                // TODO: cache shapes
                let text_shape_id = renderer.add_shape(
                    Shape::rect(
                        [(0.0, 0.0), (text_area_size.width, text_area_size.height)],
                        Color::TRANSPARENT,
                        Stroke::new(0.0, Color::TRANSPARENT),
                    ),
                    None,
                    (text_rect.min.x, text_rect.min.y),
                    // TODO: that's not an actual cache key, but it's fine for now
                    Some(self.text_texture_id),
                );

                renderer.set_shape_texture(text_shape_id, Some(self.text_texture_id));
            }

            // Add a simple cursor indicator
            let cursor_line_estimate = (self.cursor_position as f32 / 60.0) as usize; // Rough estimate
            let cursor_y = 60.0 + (cursor_line_estimate as f32 * text_style.line_height_pt());
            let cursor_x = 60.0 + ((self.cursor_position % 60) as f32 * 12.0); // Very rough approximation

            let cursor = Shape::rect(
                [
                    (cursor_x, cursor_y),
                    (cursor_x + 2.0, cursor_y + text_style.font_size.0),
                ],
                Color::rgb(97, 175, 239), // Blue cursor
                Stroke::new(0.0, Color::TRANSPARENT),
            );
            renderer.add_shape(cursor, None, (0.0, 0.0), None);

            // Show statistics about protextinator's text management
            if let Some(text_state) = self.text_manager.text_states.get(&text_id) {
                // Create a separate buffer for stats display
                let stats_id = Id::new("stats_text");
                let stats_text = format!(
                    "Protextinator Stats:\n• Text lines in buffer: {}\n• Total characters: {}\n• Cached buffers: {}\n• Buffer metadata ID: {}", 
                    text_state.buffer().lines.len(),
                    self.text_content.len(),
                    self.text_manager.text_states.len(),
                    text_id.0
                );

                let stats_rect =
                    Rect::new(Point { x: 60.0, y: 660.0 }, Point { x: 840.0, y: 700.0 });

                let stats_style = TextStyle {
                    font_size: FontSize(14.0),
                    line_height: LineHeight(1.2),
                    font_color: FontColor(protextinator::cosmic_text::Color::rgb(0x61, 0xAF, 0xEF)), // Light blue
                    horizontal_alignment: HorizontalTextAlignment::Start,
                    vertical_alignment: VerticalTextAlignment::Start,
                    wrap: Some(TextWrap::Wrap),
                    font_family: FontFamily::Serif,
                };

                // Create or update the stats text state
                if !self.text_manager.text_states.contains_key(&stats_id) {
                    self.text_manager.create_state(stats_id, &stats_text, ());
                }

                // Get the stats text state and reshape if needed
                if let Some(stats_text_state) = self.text_manager.text_states.get_mut(&stats_id) {
                    stats_text_state.set_text(&stats_text);
                    stats_text_state.set_outer_size(&stats_rect.size().into());
                    stats_text_state.set_style(&stats_style);
                    stats_text_state.set_buffer_metadata(stats_id.0 as usize);
                    stats_text_state.recalculate(&mut self.text_manager.text_context);

                    // Render stats using add_text_buffer as well
                    let _stats_buffer = &stats_text_state.buffer();
                    let _stats_area = MathRect {
                        min: (stats_rect.min.x, stats_rect.min.y).into(),
                        max: (stats_rect.max.x, stats_rect.max.y).into(),
                    };

                    // TODO: fix dis, load text the same way as main text using a texture
                    // renderer.add_text_buffer(
                    //     stats_buffer,
                    //     stats_area,
                    //     Color::rgb(97, 175, 239),
                    //     0.0,
                    //     stats_id.0 as usize,
                    //     None,
                    // );
                }
            }

            // Render the frame
            match renderer.render() {
                Ok(_) => {}
                Err(grafo::wgpu::SurfaceError::Lost) => {
                    renderer.resize(renderer.size());
                }
                Err(grafo::wgpu::SurfaceError::OutOfMemory) => {
                    eprintln!("Out of memory!");
                }
                Err(e) => eprintln!("Render error: {e:?}"),
            }
        }
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            self.setup_renderer(event_loop);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Close requested, exiting...");
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                println!("Resized to {:?}", physical_size);
                if let Some(renderer) = &mut self.renderer {
                    let new_size = (physical_size.width, physical_size.height);
                    renderer.resize(new_size);
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                println!("Scale factor changed: {}", scale_factor);
                if let Some(window) = &self.window {
                    let size = window.inner_size();
                    let physical_size = (size.width, size.height);
                    // Recreate renderer with the new scale factor
                    let new_renderer = block_on(Renderer::new(
                        window.clone(),
                        physical_size,
                        scale_factor,
                        true,
                        false,
                    ));
                    self.renderer = Some(new_renderer);
                    // Propagate scale to TextManager so buffers reshape in device pixels
                    self.text_manager.set_scale_factor(scale_factor as f32);
                    // Force texture reallocation next frame if needed
                    self.text_texture_dimenstions = None;
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                self.render_frame();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key,
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match logical_key {
                Key::Named(NamedKey::Escape) => {
                    event_loop.exit();
                }
                Key::Named(NamedKey::Backspace) => {
                    self.handle_backspace();
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                Key::Named(NamedKey::Enter) => {
                    self.handle_text_input("\n");
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                Key::Character(text) => {
                    self.handle_text_input(&text);
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Request a redraw periodically for smooth experience
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn rasterize_text_into_pixels(
    font_system: &mut cosmic_text::FontSystem,
    swash_cache: &mut cosmic_text::SwashCache,
    _scale_factor: f32,
    _logical_size: (f32, f32),
    pixels: &mut [u8], // RGBA8 sRGB
    dimensions: (u32, u32),
    buffer: &cosmic_text::Buffer,
) {
    let (tex_w, tex_h) = dimensions;
    debug_assert_eq!(pixels.len(), (tex_w * tex_h * 4) as usize);

    // Clear to transparent
    for px in pixels.chunks_exact_mut(4) {
        px[0] = 0;
        px[1] = 0;
        px[2] = 0;
        px[3] = 0;
    }

    // Note: We do not modify, clone, or create buffers here; we only draw using the provided one.
    // For best sharpness, ensure the provided buffer is shaped for device pixels upstream.

    // Use Buffer::draw to iterate painted rects and alpha-blend into our pixel buffer
    // Base color is white; spans can still carry their own colors if set
    let base_color = cosmic_text::Color::rgb(255, 255, 255);
    buffer.draw(font_system, swash_cache, base_color, |x, y, w, h, color| {
        // Clip to buffer bounds
        let (x0, y0) = ((x as u32).min(tex_w), (y as u32).min(tex_h));
        let mut w = w as u32;
        let mut h = h as u32;
        if x0 >= tex_w || y0 >= tex_h || w == 0 || h == 0 {
            return;
        }
        if x0 + w > tex_w {
            w = tex_w - x0;
        }
        if y0 + h > tex_h {
            h = tex_h - y0;
        }

        let [r, g, b, a] = color.as_rgba();
        let src_a = a as f32 / 255.0;
        for row in 0..h {
            let dst_row_start = ((y0 + row) * tex_w * 4 + x0 * 4) as usize;
            let row_slice = &mut pixels[dst_row_start..dst_row_start + (w as usize) * 4];
            for px in row_slice.chunks_exact_mut(4) {
                let dst_r = px[0] as f32 / 255.0;
                let dst_g = px[1] as f32 / 255.0;
                let dst_b = px[2] as f32 / 255.0;
                let dst_a = px[3] as f32 / 255.0;

                // Straight alpha blend in sRGB space (good enough for example)
                let out_a = src_a + dst_a * (1.0 - src_a);
                let inv = if out_a > 0.0 { 1.0 - src_a } else { 0.0 };
                let out_r = (r as f32 / 255.0) * src_a + dst_r * inv;
                let out_g = (g as f32 / 255.0) * src_a + dst_g * inv;
                let out_b = (b as f32 / 255.0) * src_a + dst_b * inv;

                px[0] = (out_r.clamp(0.0, 1.0) * 255.0 + 0.5).floor() as u8;
                px[1] = (out_g.clamp(0.0, 1.0) * 255.0 + 0.5).floor() as u8;
                px[2] = (out_b.clamp(0.0, 1.0) * 255.0 + 0.5).floor() as u8;
                px[3] = (out_a.clamp(0.0, 1.0) * 255.0 + 0.5).floor() as u8;
            }
        }
    });
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    if let Err(e) = event_loop.run_app(&mut app) {
        eprintln!("Event loop error: {e:?}");
    }
}
