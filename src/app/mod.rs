use render::Renderer;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window};

use crate::config::AppConfig;
use std::time::Instant;

use crate::prelude::*;
use error::{Error, Result};
pub mod error;
pub mod render;

pub struct App<'window> {
    app_config: AppConfig,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer<'window>>,
    now: Instant,
    window_size: PhysicalSize<u32>,
    start_time: Instant,
    pub exit_state: Result<()>,
}

impl<'a> App<'a> {
    pub fn new(app_config: AppConfig) -> Self {
        Self {
            app_config,
            window_size: PhysicalSize::new(800, 800),
            now: Instant::now(),
            start_time: Instant::now(),
            window: None,
            renderer: None,

            exit_state: Ok(()),
        }
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        info!("resuming app");

        if self.renderer.is_some() {
            warn!("renderer is already initialised bur recieved resumed");
        }

        let window_attributes = Window::default_attributes().with_title("theed");

        let window = match event_loop.create_window(window_attributes) {
            Ok(window) => Arc::new(window),
            Err(e) => {
                error!("failed to create window");
                self.exit_state = Err(e.into());
                event_loop.exit();
                return;
            }
        };

        self.renderer = match pollster::block_on(Renderer::new(window.clone(), &self.app_config.renderer)) {
            Ok(renderer) => Some(renderer),
            Err(e) => {
                error!("failed to build wgpu renderer");
                self.exit_state = Err(e.into());
                event_loop.exit();
                return;
            }
        };

        self.window = Some(window);
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.exit_state = Ok(())
    }

    fn window_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, _: winit::window::WindowId, event: winit::event::WindowEvent) {
        trace!("event: {:?}", event);

        match event {
            // ensure we can always exit!
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event: KeyEvent {
                    logical_key: Key::Named(NamedKey::Escape),
                    ..
                },
                ..
            } => event_loop.exit(),
            WindowEvent::Resized(size) => {
                debug!("event: {:?}", event);
                if let (Some(renderer), Some(window)) = (self.renderer.as_mut(), self.window.as_ref()) {
                    self.window_size = size;
                    renderer.resize(size);
                    window.request_redraw();
                }
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let delta = self.start_time.elapsed().as_millis() as f32 / 1000.0;
        let frame_delta = self.now.elapsed().as_millis() as f32 / 1000.0;
        self.now = Instant::now();

        if let (Some(renderer), Some(window)) = (self.renderer.as_mut(), self.window.as_ref()) {
            match renderer.render(window.clone()) {
                Ok(_) => {}
                Err(e) => {
                    error!("render pass failed");
                    self.exit_state = Err(e.into());
                    event_loop.exit();
                    return;
                }
            }

        }

        trace!("delta: {}ms, frame_delta {}ms,", delta, frame_delta);
    }
}
