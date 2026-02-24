//! Minimal WGPU test app for bishop crate.
//!
//! Run with: cargo run --example wgpu_test -p bishop --features wgpu --no-default-features

use std::sync::Arc;

use bishop::prelude::*;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

struct App {
    ctx: Option<WgpuContext>,
    window: Option<Arc<Window>>,
    frame_count: u32,
    typed_text: String,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = Window::default_attributes()
                .with_title("Bishop WGPU Test")
                .with_inner_size(LogicalSize::new(800, 600));
            let window = match event_loop.create_window(attrs) {
                Ok(w) => Arc::new(w),
                Err(e) => {
                    eprintln!("Failed to create window: {e}");
                    event_loop.exit();
                    return;
                }
            };
            match WgpuContext::new_sync(window.clone()) {
                Ok(c) => self.ctx = Some(c),
                Err(e) => {
                    eprintln!("Failed to create WgpuContext: {e}");
                    event_loop.exit();
                    return;
                }
            }
            self.window = Some(window);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(ctx) = &mut self.ctx {
            ctx.handle_window_event(&event);
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                self.render_frame();
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

impl App {
    fn render_frame(&mut self) {
        let Some(ctx) = &mut self.ctx else { return };

        // Read input BEFORE begin_frame clears it
        for c in ctx.chars_pressed() {
            self.typed_text.push(c);
        }
        if ctx.is_key_pressed(KeyCode::Backspace) && !self.typed_text.is_empty() {
            self.typed_text.pop();
        }
        if ctx.is_key_pressed(KeyCode::Escape) {
            self.typed_text.clear();
        }

        ctx.begin_frame();

        ctx.clear_background(Color::new(0.12, 0.12, 0.16, 1.0));

        // Primitives
        ctx.draw_rectangle(50.0, 50.0, 100.0, 80.0, Color::RED);
        ctx.draw_circle(200.0, 90.0, 40.0, Color::GREEN);
        ctx.draw_line(300.0, 50.0, 400.0, 130.0, 3.0, Color::BLUE);
        ctx.draw_triangle(
            Vec2::new(450.0, 130.0),
            Vec2::new(500.0, 50.0),
            Vec2::new(550.0, 130.0),
            Color::YELLOW,
        );

        // Text
        ctx.draw_text(
            "WGPU Test - Bishop Engine",
            50.0,
            180.0,
            24.0,
            Color::WHITE,
        );
        ctx.draw_text(
            &format!("Frame: {}", self.frame_count),
            50.0,
            220.0,
            16.0,
            Color::GREY,
        );

        // Input display
        let (mx, my) = ctx.mouse_position();
        ctx.draw_text(
            &format!("Mouse: ({:.0}, {:.0})", mx, my),
            50.0,
            250.0,
            16.0,
            Color::GREY,
        );

        // Key status display
        let mut y = 280.0;
        let keys_to_check = [
            (KeyCode::Space, "SPACE"),
            (KeyCode::W, "W"),
            (KeyCode::A, "A"),
            (KeyCode::S, "S"),
            (KeyCode::D, "D"),
            (KeyCode::Up, "UP"),
            (KeyCode::Down, "DOWN"),
            (KeyCode::Left, "LEFT"),
            (KeyCode::Right, "RIGHT"),
        ];

        let mut pressed_keys = Vec::new();
        for (key, name) in keys_to_check {
            if ctx.is_key_down(key) {
                pressed_keys.push(name);
            }
        }

        if pressed_keys.is_empty() {
            ctx.draw_text("Keys: (none)", 50.0, y, 16.0, Color::GREY);
        } else {
            ctx.draw_text(
                &format!("Keys: {}", pressed_keys.join(", ")),
                50.0,
                y,
                16.0,
                Color::GREEN,
            );
        }
        y += 30.0;

        // Typed text display (accumulates, backspace to delete, escape to clear)
        ctx.draw_text(
            &format!("Typed: {}_", self.typed_text),
            50.0,
            y,
            16.0,
            Color::SKYBLUE,
        );
        y += 20.0;
        ctx.draw_text(
            "(Backspace=delete, Escape=clear)",
            50.0,
            y,
            12.0,
            Color::GREY,
        );

        if let Err(e) = ctx.render_frame() {
            eprintln!("Render error: {e}");
        }
        ctx.update();

        self.frame_count += 1;
    }
}

fn main() {
    let event_loop = match EventLoop::new() {
        Ok(el) => el,
        Err(e) => {
            eprintln!("Failed to create event loop: {e}");
            return;
        }
    };
    let mut app = App {
        ctx: None,
        window: None,
        frame_count: 0,
        typed_text: String::new(),
    };
    if let Err(e) = event_loop.run_app(&mut app) {
        eprintln!("Event loop error: {e}");
    }
}
