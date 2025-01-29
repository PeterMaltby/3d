use std::sync::Arc;
use wgpu::{BackendOptions, Backends, InstanceDescriptor, InstanceFlags, SurfaceTexture};
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::window::Window;

use crate::prelude::*;
use config::Config;
use error::{Error, Result};

pub mod config;
pub mod error;

pub struct Renderer<'window> {
    context: Surface<'window>,
    device: Device,
    queue: Queue,
    window: Arc<Window>,
    context_configuration: SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    size: winit::dpi::PhysicalSize<u32>,
}

impl<'window> Renderer<'window> {
    pub async fn new(window: Arc<Window>, config: &Config) -> Result<Renderer<'window>> {
        let backends = Backends::from_comma_list(&config.backends);

        info!("{:?}", backends);

        let graphics_instance_config = InstanceDescriptor {
            backends: Backends::from_comma_list(&config.backends),
            flags: InstanceFlags::from_build_config(),
            backend_options: BackendOptions::from_env_or_default(),
        };

        let graphics_instance = wgpu::Instance::new(&graphics_instance_config);

        // get our render context
        let context = graphics_instance.create_surface(window.clone())?;

        let adapter_config = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: match config.force_fallback_adapter {
                Some(true) => true,
                _ => false,
            },
            compatible_surface: Some(&context),
        };

        let adapter = match graphics_instance.request_adapter(&adapter_config).await {
            Some(adapter) => adapter,
            _ => {
                error!("failed to get graphics adapter");
                return Err(error::Error::AdpaterRequestFailure);
            }
        };

        // TODO link to config
        let device_reqs = wgpu::DeviceDescriptor {
            required_limits: wgpu::Limits::default(),
            required_features: wgpu::Features::empty(),
            label: None,
            memory_hints: Default::default(),
        };

        let (device, queue) = adapter.request_device(&device_reqs, None).await?;

        let size = winit::dpi::PhysicalSize::new(800, 800);

        let context_capabilities = context.get_capabilities(&adapter);

        // TODO set some nice defaults
        let surface_format = context_capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(context_capabilities.formats[0]);

        let context_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: context_capabilities.present_modes[0],
            alpha_mode: context_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Piepline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("renderPipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: context_configuration.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Ok(Self {
            render_pipeline,
            window,
            context,
            device,
            queue,
            context_configuration,
            size,
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        info!("resized window");
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.context_configuration.width = new_size.width;
            self.context_configuration.height = new_size.height;
            self.context.configure(&self.device, &self.context_configuration);
        }
    }

    pub fn render(&mut self) -> Result<()> {
        // WGPU calls the context a surface
        let surface_texture: SurfaceTexture = match self.context.get_current_texture() {
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

        // The context texture is our windows texture
        let context_texture = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    // output to texture
                    view: &context_texture,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 0.6,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));
        self.window.as_ref().request_redraw();
        surface_texture.present();

        return Ok(());
    }
}
