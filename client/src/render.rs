use wgpu::util::DeviceExt;
use wgpu::{Device, Instance, Queue, Surface, SurfaceConfiguration};
use winit::window::Window;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

// A simple grass block (cube)
const VERTICES: &[Vertex] = &[
    // Top face (Green)
    Vertex { position: [-0.5, 0.5, -0.5], color: [0.3, 0.7, 0.3] },
    Vertex { position: [0.5, 0.5, -0.5], color: [0.3, 0.7, 0.3] },
    Vertex { position: [0.5, 0.5, 0.5], color: [0.3, 0.7, 0.3] },
    Vertex { position: [-0.5, 0.5, 0.5], color: [0.3, 0.7, 0.3] },
    // Bottom face (Dirt color)
    Vertex { position: [-0.5, -0.5, 0.5], color: [0.4, 0.25, 0.15] },
    Vertex { position: [0.5, -0.5, 0.5], color: [0.4, 0.25, 0.15] },
    Vertex { position: [0.5, -0.5, -0.5], color: [0.4, 0.25, 0.15] },
    Vertex { position: [-0.5, -0.5, -0.5], color: [0.4, 0.25, 0.15] },
    // Front face (Dirt with grass top)
    Vertex { position: [-0.5, -0.5, 0.5], color: [0.4, 0.25, 0.15] },
    Vertex { position: [0.5, -0.5, 0.5], color: [0.4, 0.25, 0.15] },
    Vertex { position: [0.5, 0.5, 0.5], color: [0.3, 0.6, 0.2] },
    Vertex { position: [-0.5, 0.5, 0.5], color: [0.3, 0.6, 0.2] },
    // Back face (Dirt with grass top)
    Vertex { position: [0.5, -0.5, -0.5], color: [0.4, 0.25, 0.15] },
    Vertex { position: [-0.5, -0.5, -0.5], color: [0.4, 0.25, 0.15] },
    Vertex { position: [-0.5, 0.5, -0.5], color: [0.3, 0.6, 0.2] },
    Vertex { position: [0.5, 0.5, -0.5], color: [0.3, 0.6, 0.2] },
    // Left face (Dirt with grass top)
    Vertex { position: [-0.5, -0.5, -0.5], color: [0.4, 0.25, 0.15] },
    Vertex { position: [-0.5, -0.5, 0.5], color: [0.4, 0.25, 0.15] },
    Vertex { position: [-0.5, 0.5, 0.5], color: [0.3, 0.6, 0.2] },
    Vertex { position: [-0.5, 0.5, -0.5], color: [0.3, 0.6, 0.2] },
    // Right face (Dirt with grass top)
    Vertex { position: [0.5, -0.5, 0.5], color: [0.4, 0.25, 0.15] },
    Vertex { position: [0.5, -0.5, -0.5], color: [0.4, 0.25, 0.15] },
    Vertex { position: [0.5, 0.5, -0.5], color: [0.3, 0.6, 0.2] },
    Vertex { position: [0.5, 0.5, 0.5], color: [0.3, 0.6, 0.2] },
];

const INDICES: &[u16] = &[
    0, 1, 2, 0, 2, 3, // Top
    4, 5, 6, 4, 6, 7, // Bottom
    8, 9, 10, 8, 10, 11, // Front
    12, 13, 14, 12, 14, 15, // Back
    16, 17, 18, 16, 18, 19, // Left
    20, 21, 22, 20, 22, 23, // Right
];

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

pub struct Renderer {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    egui_renderer: egui_wgpu::Renderer,
    
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    depth_texture: wgpu::TextureView,
}

impl Renderer {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = unsafe {
            let s = instance.create_surface(window).unwrap();
            std::mem::transmute::<Surface<'_>, Surface<'static>>(s)
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();

        let format = surface.get_capabilities(&adapter).formats[0];
        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let egui_renderer = egui_wgpu::Renderer::new(&device, format, None, 1);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let camera_uniform = CameraUniform {
            view_proj: nalgebra::Matrix4::identity().into(),
        };

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let depth_texture = Self::create_depth_texture(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            egui_renderer,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            camera_buffer,
            camera_bind_group,
            depth_texture,
        }
    }

    pub fn render(&mut self, ctx: &egui::Context, full_output: &egui::FullOutput, show_3d: bool) {
        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(e) => {
                tracing::error!("Surface error: {:?}", e);
                return;
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ferrite"),
            });

        let bg_color = wgpu::Color { r: 0.53, g: 0.81, b: 0.92, a: 1.0 };

        // 3D pass (only in-game)
        if show_3d {
            let aspect = self.config.width as f32 / self.config.height as f32;
            let proj = nalgebra::Perspective3::new(aspect, 70.0_f32.to_radians(), 0.1, 100.0);

            let t = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f32();

            let eye = nalgebra::Point3::new(t.cos() * 3.0, 1.5, t.sin() * 3.0);
            let target = nalgebra::Point3::new(0.0, 0.0, 0.0);
            let view_matrix =
                nalgebra::Isometry3::look_at_rh(&eye, &target, &nalgebra::Vector3::y());

            let view_proj = proj.to_homogeneous() * view_matrix.to_homogeneous();

            let camera_uniform = CameraUniform {
                view_proj: view_proj.into(),
            };
            self.queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::cast_slice(&[camera_uniform]),
            );

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("3d_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(bg_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            pass.set_pipeline(&self.render_pipeline);
            pass.set_bind_group(0, &self.camera_bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(0..(INDICES.len() as u32), 0, 0..1);
        }

        // egui pass
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: 1.0,
        };

        let primitives = ctx
            .tessellate(full_output.shapes.clone(), full_output.pixels_per_point);

        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer
                .update_texture(&self.device, &self.queue, *id, image_delta);
        }

        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &primitives,
            &screen_descriptor,
        );

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: if show_3d {
                            wgpu::LoadOp::Load
                        } else {
                            wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.05,
                                g: 0.05,
                                b: 0.05,
                                a: 1.0,
                            })
                        },
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.egui_renderer
                .render(&mut pass, &primitives, &screen_descriptor);
        }

        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::TextureView {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("depth_texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.depth_texture = Self::create_depth_texture(&self.device, &self.config);
    }
}
