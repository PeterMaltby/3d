use std::error::Error;
use std::num::NonZeroU32;

//use gl::types::GLfloat;
use raw_window_handle::HasWindowHandle;
use renderer::{Renderer};
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{Key, NamedKey};
use winit::window::Window;

use glutin::config::{Config, ConfigTemplateBuilder, GetGlConfig};
use glutin::context::{ContextApi, ContextAttributesBuilder, NotCurrentContext, PossiblyCurrentContext, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, SwapInterval, WindowSurface};

use glutin_winit::{DisplayBuilder, GlWindow};

use std::time::Instant;
use log::{debug, info, trace, warn};

mod renderer;

pub struct ApplicationConfig {}

pub fn main(_: ApplicationConfig) -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new().unwrap();

    let gl_display_config = ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .with_float_pixels(false)
        .with_transparency(cfg!(cgl_backend));

    let window_attributes = Window::default_attributes().with_transparent(true).with_title("hello world!");
    let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attributes));

    let mut app = App::new(gl_display_config, display_builder);

    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop.run_app(&mut app)?;

    app.exit_state
}

struct App {
    gl_display_template: ConfigTemplateBuilder,
    gl_display: GlDisplayCreationState,
    gl_context: Option<PossiblyCurrentContext>,
    renderer: Option<Renderer>,
    state: Option<AppState>,
    now: Instant,
    start_time: Instant,
    exit_state: Result<(), Box<dyn Error>>,
}

enum GlDisplayCreationState {
    /// The display was not build yet.
    Builder(DisplayBuilder),
    /// The display was already created for the application.
    Created,
}

struct AppState {
    gl_surface: Surface<WindowSurface>,
    // NOTE: Window should be dropped after all resources created using its
    // raw-window-handle.
    window: Window,
}

impl App {
    fn new(gl_display_template: ConfigTemplateBuilder, display_builder: DisplayBuilder) -> Self {
        Self {
            gl_display_template,
            gl_display: GlDisplayCreationState::Builder(display_builder),
            renderer: None,
            state: None,
            gl_context: None,
            now: Instant::now(),
            start_time: Instant::now(),
            exit_state: Ok(()),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        trace!("resuming event loop {:?}", event_loop );

        let (window, gl_config) = match &self.gl_display {
            // We just created the event loop, so initialize the display, pick the config, and
            // create the context.
            GlDisplayCreationState::Builder(display_builder) => {
                debug!("Building new window in `resumed`");
                let (window, gl_config) = match display_builder.clone().build(event_loop, self.gl_display_template.clone(), gl_config_picker) {
                    Ok((window, gl_config)) => (window.unwrap(), gl_config),
                    Err(err) => {
                        self.exit_state = Err(err);
                        event_loop.exit();
                        return;
                    }
                };

                self.gl_display = GlDisplayCreationState::Created;

                self.gl_context = Some(create_gl_context(&window, &gl_config).treat_as_possibly_current());

                (window, gl_config)
            }
            GlDisplayCreationState::Created => {
                debug!("Recreating window in `resumed`");
                // Pick the config which we already use for the context.
                let gl_config = self.gl_context.as_ref().unwrap().config();
                let window_attributes = Window::default_attributes().with_transparent(true).with_title("hello world2");
                match glutin_winit::finalize_window(event_loop, window_attributes, &gl_config) {
                    Ok(window) => (window, gl_config),
                    Err(err) => {
                        self.exit_state = Err(err.into());
                        event_loop.exit();
                        return;
                    }
                }
            }
        };

        print_config(&gl_config);

        let surface_attributes = window.build_surface_attributes(Default::default()).expect("Failed to build surface attributes");
        let gl_surface = unsafe { gl_config.display().create_window_surface(&gl_config, &surface_attributes).unwrap() };
        let gl_context = self.gl_context.as_ref().unwrap();
        gl_context.make_current(&gl_surface).unwrap();

        self.renderer.get_or_insert_with(|| Renderer::new(&gl_config.display()).unwrap());

        // Try setting vsync.
        if let Err(res) = gl_surface.set_swap_interval(gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap())) {
            warn!("Error setting vsync: {res:?}");
        }

        // reset app timer to stop big jumps
        self.now = Instant::now();

        assert!(self.state.replace(AppState { gl_surface, window }).is_none())
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        debug!("exiting event loop");
        // NOTE: The handling below is only needed due to nvidia on Wayland to not crash
        // on exit due to nvidia driver touching the Wayland display from on
        // `exit` hook.
        let _gl_display = self.gl_context.take().unwrap().display();

        // Clear the window.
        self.state = None;
        #[cfg(egl_backend)]
        #[allow(irrefutable_let_patterns)]
        if let glutin::display::Display::Egl(display) = _gl_display {
            unsafe {
                display.terminate();
            }
        }
    }

    fn window_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, _: winit::window::WindowId, event: winit::event::WindowEvent) {
        match event {
            WindowEvent::Resized(size) if size.width != 0 && size.height != 0 => {
                if let Some(AppState { gl_surface, window: _ }) = self.state.as_ref() {
                    let gl_context = self.gl_context.as_ref().unwrap();
                    gl_surface.resize(gl_context, NonZeroU32::new(size.width).unwrap(), NonZeroU32::new(size.height).unwrap());

                    let renderer: &mut Renderer = self.renderer.as_mut().unwrap();
                    renderer.resize(size.width as i32, size.height as i32);

                    let delta = self.start_time.elapsed().as_millis() as f32 / 1000.0;
                    let frame_delta = self.now.elapsed().as_millis() as f32 / 1000.0;
                    self.now = Instant::now();

                    trace!("delta: {}, frame_delta {},", delta, frame_delta);

                    renderer.draw(delta, frame_delta);
                }
            }
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event: KeyEvent {
                    logical_key: Key::Named(NamedKey::Escape),
                    ..
                },
                ..
            } => event_loop.exit(),
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(AppState { gl_surface, window }) = self.state.as_ref() {
            let gl_context = self.gl_context.as_ref().unwrap();
            let renderer = self.renderer.as_mut().unwrap();

            let delta = self.start_time.elapsed().as_millis() as f32 / 1000.0;
            let frame_delta = self.now.elapsed().as_millis() as f32 / 1000.0;
            self.now = Instant::now();

            trace!("delta: {}, frame_delta {},", delta, frame_delta);

            renderer.draw(delta, frame_delta);
            window.request_redraw();

            gl_surface.swap_buffers(gl_context).unwrap();
        }
    }
}

// Find the config with the maximum number of samples
pub fn gl_config_picker(configs: Box<dyn Iterator<Item = Config> + '_>) -> Config {
    configs
        .reduce(|accum, config| {
            let transparency_check = config.supports_transparency().unwrap_or(false) & !accum.supports_transparency().unwrap_or(false);

            if transparency_check || config.num_samples() > accum.num_samples() {
                config
            } else {
                accum
            }
        })
        .unwrap()
}

fn print_config(config: &Config) {
    info!("config:");
    info!("color buffer type: {:?}", config.color_buffer_type());
    info!("float pixels: {:?}", config.float_pixels());
    info!("alpha size: {:?}", config.alpha_size());
    info!("depth size: {:?}", config.depth_size());
    info!("stencil size: {:?}", config.stencil_size());
    info!("num smaples: {:?}", config.num_samples());
    info!("srgb_capable: {:?}", config.srgb_capable());
    info!("hardware accelerated: {:?}", config.hardware_accelerated());
    info!("transparency: {:?}", config.supports_transparency());
    info!("API: {:?}\n", config.api());
}

fn create_gl_context(window: &Window, gl_config: &Config) -> NotCurrentContext {
    let raw_window_handle = window.window_handle().ok().map(|wh| wh.as_raw());

    // The context creation part.
    let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);

    // Since glutin by default tries to create OpenGL core context, which may not be
    // present we should try gles.
    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .build(raw_window_handle);

    // There are also some old devices that support neither modern OpenGL nor GLES.
    // To support these we can try and create a 2.1 context.
    let legacy_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(2, 1))))
        .build(raw_window_handle);

    // Reuse the uncurrented context from a suspended() call if it exists, otherwise
    // this is the first time resumed() is called, where the context still
    // has to be created.
    let gl_display = gl_config.display();

    unsafe {
        gl_display.create_context(gl_config, &context_attributes).unwrap_or_else(|_| {
            gl_display.create_context(gl_config, &fallback_context_attributes).unwrap_or_else(|_| {
                gl_display
                    .create_context(gl_config, &legacy_context_attributes)
                    .expect("failed to create context")
            })
        })
    }
}
