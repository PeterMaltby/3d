use std::sync::Arc;
use wgpu::{BackendOptions, Backends, InstanceDescriptor, InstanceFlags, SurfaceTexture};
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::window::Window;

use config::Config;
use crate::prelude::*;
use error::{Error, Result};

pub mod config;
pub mod error;

pub struct Renderer<'window> {
    surface: Surface<'window>,
    device: Device,
    queue: Queue,
    surface_configuration: SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
}

impl<'window> Renderer<'window> {
    pub async fn new(window: Arc<Window>, config: &Config) -> Result<Renderer<'window>> {
        let backends = Backends::from_comma_list(&config.backends);

        info!("{:?}", backends);

        let gpu_instance_config = InstanceDescriptor {
            backends: Backends::from_comma_list(&config.backends),
            flags: InstanceFlags::from_build_config(),
            backend_options: BackendOptions::from_env_or_default(),
        };

        let gpu_instance = wgpu::Instance::new(&gpu_instance_config);

        let surface = gpu_instance.create_surface(window)?;

        let adapter_config = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: match config.force_fallback_adapter {
                Some(true) => true,
                _ => false,
            },
            compatible_surface: Some(&surface),
        };

        let adapter = match gpu_instance.request_adapter(&adapter_config).await {
            Some(adapter) => adapter,
            _ => return Err(error::Error::AdpaterRequestFailure),
        };

        let device_reqs = wgpu::DeviceDescriptor {
            required_limits: wgpu::Limits::default(),
            required_features: wgpu::Features::empty(),
            label: None,
            memory_hints: Default::default(),
        };

        let (device, queue) = adapter.request_device(&device_reqs, None).await?;

        let size = winit::dpi::PhysicalSize::new(800, 800);

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter().find(|f| f.is_srgb()).copied().unwrap_or(surface_caps.formats[0]);
        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        Ok(Self {
            surface,
            device,
            queue,
            surface_configuration,
            size,
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        info!("resized window");
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_configuration.width = new_size.width;
            self.surface_configuration.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_configuration);
        }
    }

    pub fn render(&mut self, window: Arc<Window>) -> Result<()> {
        let surface_texture: SurfaceTexture = match self.surface.get_current_texture() {
            Ok(st) => st,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                warn!("surface lost or outdated attempting recovery");
                self.resize(winit::dpi::PhysicalSize::new(800, 800));
                return Ok(());
            }
            Err(wgpu::SurfaceError::Timeout) => {
                warn!("surface timedout missed frame");
                return Ok(());
            }
            Err(e) => return Err(error::Error::SurfaceError(e)),
        };

        let texture_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let _r_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }
        self.queue.submit(Some(encoder.finish()));
        window.as_ref().request_redraw();
        surface_texture.present();

        return Ok(());
    }
}
