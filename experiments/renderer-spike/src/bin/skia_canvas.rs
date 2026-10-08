use std::{num::NonZeroU32, sync::Arc};

use renderer_spike::{Camera, Scene, render_rgba};
use softbuffer::{Context, Surface};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition},
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

struct CanvasApp {
    window: Option<Arc<Window>>,
    context: Option<Context<Arc<Window>>>,
    surface: Option<Surface<Arc<Window>, Arc<Window>>>,
    scene: Scene,
    camera: Camera,
    cursor: (f32, f32),
    last_cursor: (f32, f32),
    dragging: bool,
    surface_size: (u32, u32),
}

impl Default for CanvasApp {
    fn default() -> Self {
        Self {
            window: None,
            context: None,
            surface: None,
            scene: Scene::new(10_000),
            camera: Camera::default(),
            cursor: (0.0, 0.0),
            last_cursor: (0.0, 0.0),
            dragging: false,
            surface_size: (0, 0),
        }
    }
}

impl ApplicationHandler for CanvasApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attributes = WindowAttributes::default()
            .with_title("Skia raster canvas spike — drag to pan, wheel to zoom")
            .with_inner_size(LogicalSize::new(1100.0, 760.0));
        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                eprintln!("could not create canvas window: {error}");
                event_loop.exit();
                return;
            }
        };

        let context = match Context::new(window.clone()) {
            Ok(context) => context,
            Err(error) => {
                eprintln!("could not initialize software presentation: {error}");
                event_loop.exit();
                return;
            }
        };
        let surface = match Surface::new(&context, window.clone()) {
            Ok(surface) => surface,
            Err(error) => {
                eprintln!("could not create software presentation surface: {error}");
                event_loop.exit();
                return;
            }
        };

        self.window = Some(window);
        self.context = Some(context);
        self.surface = Some(surface);
        self.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.window.as_ref().is_none_or(|window| window.id() != window_id) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                self.resize(size.width, size.height);
                self.request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => self.cursor_moved(position),
            WindowEvent::MouseInput { button: MouseButton::Left, state, .. } => {
                self.dragging = state == ElementState::Pressed;
                self.last_cursor = self.cursor;
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let amount = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(position) => bounded_coordinate(position.y / 48.0),
                };
                self.zoom_at(amount);
            }
            WindowEvent::RedrawRequested => self.draw_frame(),
            _ => {}
        }
    }
}

impl CanvasApp {
    fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 || self.surface.is_none() {
            return;
        }
        let Some(width) = NonZeroU32::new(width) else {
            return;
        };
        let Some(height) = NonZeroU32::new(height) else {
            return;
        };
        if let Some(surface) = &mut self.surface {
            if let Err(error) = surface.resize(width, height) {
                eprintln!("could not resize the software surface: {error}");
                return;
            }
            self.surface_size = (width.get(), height.get());
        }
    }

    fn cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        self.cursor = (bounded_coordinate(position.x), bounded_coordinate(position.y));
        if self.dragging {
            self.camera
                .pan_by(self.cursor.0 - self.last_cursor.0, self.cursor.1 - self.last_cursor.1);
            self.request_redraw();
        }
        self.last_cursor = self.cursor;
    }

    fn zoom_at(&mut self, wheel_delta: f32) {
        self.camera.zoom_at(self.cursor, wheel_delta);
        self.request_redraw();
    }

    fn draw_frame(&mut self) {
        let Some(window) = &self.window else {
            return;
        };
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }
        if self.surface_size != (size.width, size.height) {
            self.resize(size.width, size.height);
        }

        let pixels = match render_rgba(&self.scene, self.camera, size.width, size.height) {
            Ok(pixels) => pixels,
            Err(error) => {
                eprintln!("Skia frame failed: {error}");
                return;
            }
        };

        let Some(surface) = &mut self.surface else {
            return;
        };
        let mut buffer = match surface.buffer_mut() {
            Ok(buffer) => buffer,
            Err(error) => {
                eprintln!("could not acquire presentation buffer: {error}");
                return;
            }
        };
        for (destination, source) in buffer.iter_mut().zip(pixels.chunks_exact(4)) {
            *destination =
                (u32::from(source[0]) << 16) | (u32::from(source[1]) << 8) | u32::from(source[2]);
        }
        if let Err(error) = buffer.present() {
            eprintln!("could not present canvas frame: {error}");
        }
    }
}

// Window coordinates are clamped to a practical range before conversion; subpixel precision is
// retained for any viewport size this prototype can reasonably display.
#[allow(clippy::cast_possible_truncation)]
fn bounded_coordinate(value: f64) -> f32 {
    value.clamp(-1_000_000.0, 1_000_000.0) as f32
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
    event_loop.run_app(&mut CanvasApp::default())?;
    Ok(())
}
