mod vertex;
mod camera;
mod geometry;
mod material;

use wgpu::{
    RenderPipeline, Buffer, ShaderSource, VertexState, ColorTargetState, 
    BlendState, ShaderModuleDescriptor, PipelineLayoutDescriptor, 
    RenderPipelineDescriptor, FragmentState, ColorWrites, PrimitiveState, 
    PrimitiveTopology, FrontFace, Face, PolygonMode, MultisampleState, Device, 
    SurfaceConfiguration, CommandEncoder, TextureView, RenderPassDescriptor, 
    RenderPassColorAttachment, Operations, LoadOp, Color, ComputePipeline,
    ComputePipelineDescriptor, BindGroup, ComputePassDescriptor, Texture,
    util::{BufferInitDescriptor, DeviceExt},
};
use winit::dpi::PhysicalSize;
use vertex::Vertex;
use camera::Camera;
use geometry::GeometryStorage;
use material::MaterialStorage;
use rand::{thread_rng, Rng};

const RECTANGLE_VERTICES: &[Vertex] = &[
    Vertex::new([ 1.0,  1.0], [1.0, 0.0]),
    Vertex::new([-1.0,  1.0], [0.0, 0.0]),
    Vertex::new([ 1.0, -1.0], [1.0, 1.0]),
    Vertex::new([ 1.0, -1.0], [1.0, 1.0]),
    Vertex::new([-1.0,  1.0], [0.0, 0.0]),
    Vertex::new([-1.0, -1.0], [0.0, 1.0]),
];

const NUM_VERTICES: u32 = 6;

pub struct Pipeline {
    size: wgpu::Extent3d,
    camera: Camera,
    camera_buffer: Buffer,
    objects_buffer: Buffer,
    sample_count_buffer: Buffer,
    vertex_buffer: Buffer,
    random_texture: Texture,
    random_bind_group: BindGroup,
    camera_bind_group: BindGroup,
    compute1_bind_group1: BindGroup,
    compute2_bind_group1: BindGroup,
    compute1_bind_group2: BindGroup,
    compute2_bind_group2: BindGroup,
    compute_pipeline: ComputePipeline,
    render_bind_group1: BindGroup,
    render_bind_group2: BindGroup,
    render_pipeline: RenderPipeline,
    buffer_switch: bool,
    sample_count: u32,
}

impl Pipeline {
    pub fn new(device: &Device, config: &SurfaceConfiguration, size: PhysicalSize<u32>) -> Pipeline {
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(RECTANGLE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let size = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };

        let texture1 = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view1 = texture1.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler1 = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture2 = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view2 = texture2.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler2 = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let camera = Camera::default();

        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer Descriptor"),
            contents: bytemuck::cast_slice(&[camera.into_uniform()]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let objects = &[
            GeometryStorage::new_quad([3.5, -0.5, 1.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0], 
                                    MaterialStorage::new([2.0, 2.0, 2.0], 3, 1.0)),
            GeometryStorage::new_sphere([4.0, -0.5, -0.75], 0.25, MaterialStorage::new([1.0, 1.0, 1.0], 2, 0.0)),
            GeometryStorage::new_quad([4.0, 0.25, -1.0], [0.0, 0.0, 0.75], [0.5, -0.25, 0.0], 
                                    MaterialStorage::new([0.5, 0.5, 0.5], 0, 0.2)),
            GeometryStorage::new_quad([4.0, 0.25, -1.0], [0.0, 0.0, 0.75], [0.25, 0.5, 0.0], 
                                    MaterialStorage::new([0.5, 0.5, 0.5], 0, 0.2)),
            GeometryStorage::new_quad([4.0, 0.25, -0.25], [0.5, -0.25, 0.0], [0.25, 0.5, 0.0], 
                                    MaterialStorage::new([0.5, 0.5, 0.5], 0, 0.2)),
            GeometryStorage::new_quad([4.25, 0.75, -1.0], [0.0, 0.0, 0.75], [0.5, -0.25, 0.0], 
                                    MaterialStorage::new([0.5, 0.5, 0.5], 0, 0.2)),
            GeometryStorage::new_quad([4.5, 0.0, -1.0], [0.0, 0.0, 0.75], [0.25, 0.5, 0.0], 
                                    MaterialStorage::new([0.5, 0.5, 0.5], 0, 0.2)),
            //GeometryStorage::new_sphere([50.0, 2.0, 0.0], 1.0, MaterialStorage::new([0.8, 0.8, 0.8], 0, 0.0)),
            GeometryStorage::new_quad([3.0, -1.0, -1.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0], 
                                    MaterialStorage::new([1.0, 1.0, 1.0], 0, 1.0)),
            GeometryStorage::new_quad([3.0, -1.0, 1.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0], 
                                    MaterialStorage::new([1.0, 1.0, 1.0], 0, 1.0)),
            GeometryStorage::new_quad([5.0, -1.0, -1.0], [0.0, 0.0, 2.0], [0.0, 2.0, 0.0], 
                                    MaterialStorage::new([1.0, 1.0, 1.0], 0, 1.0)),
            GeometryStorage::new_quad([3.0, 1.0, -1.0], [2.0, 0.0, 0.0], [0.0, 0.0, 2.0], 
                                    MaterialStorage::new([1.0, 0.0, 0.0], 0, 1.0)),
            GeometryStorage::new_quad([3.0, -1.0, -1.0], [0.0, 0.0, 2.0], [2.0, 0.0, 0.0], 
                                    MaterialStorage::new([0.0, 1.0, 0.0], 0, 1.0)),
        ];

        let objects_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Objects Buffer Descriptor"),
            contents: bytemuck::cast_slice(objects),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });

        let sample_count_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Sample Count Buffer Descriptor"),
            contents: bytemuck::cast_slice(&[0u32]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });

        let random_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Random Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let random_view = random_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let compute_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Compute Shader"),
            source: ShaderSource::Wgsl(include_str!("ray_tracer.wgsl").into()),
        });

        let compute1_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
                label: Some("compute_bind_group_layout"),
            });

        let compute1_bind_group1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &compute1_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view1), // CHANGED!
                },
            ],
            label: Some("compute_bind_group"),
        });
        
        let compute1_bind_group2 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &compute1_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view2), // CHANGED!
                },
            ],
            label: Some("compute_bind_group"),
        });

        let compute2_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::ReadOnly,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
                label: Some("compute_bind_group_layout"),
            });

        let compute2_bind_group2 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &compute2_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view2), // CHANGED!
                },
            ],
            label: Some("compute_bind_group"),
        });

        let compute2_bind_group1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &compute2_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view1), // CHANGED!
                },
            ],
            label: Some("compute_bind_group"),
        });

        let camera_bind_group_layout = 
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: objects_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: sample_count_buffer.as_entire_binding(),
                },
            ],
            label: Some("camera_bind_group"),
        });

        let random_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::ReadWrite,
                            format: wgpu::TextureFormat::R32Uint,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
                label: Some("random_bind_group_layout"),
            });

        let random_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &random_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&random_view), // CHANGED!
                },
            ],
            label: Some("random_bind_group"),
        });

        let compute_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[
                &compute1_bind_group_layout, 
                &camera_bind_group_layout,
                &random_bind_group_layout,
                &compute2_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });
        
        let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "main",
        });

        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let render_bind_group1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view1), // CHANGED!
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler1), // CHANGED!
                },
            ],
            label: Some("render_bind_group"),
        });

        let render_bind_group2 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view2), // CHANGED!
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler2), // CHANGED!
                },
            ],
            label: Some("render_bind_group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&render_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline =
            device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Simple Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: "vs_main",     // 1.
                    buffers: &[Vertex::desc()], // 2.
                },
                fragment: Some(FragmentState {
                    // 3.
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(ColorTargetState {
                        // 4.
                        format: config.format,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList, // 1.
                    strip_index_format: None,
                    front_face: FrontFace::Ccw, // 2.
                    cull_mode: Some(Face::Back),
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None, // 1.
                multisample: MultisampleState {
                    count: 1,                         // 2.
                    mask: !0,                         // 3.
                    alpha_to_coverage_enabled: false, // 4.
                },
                multiview: None, // 5.
            });

        Pipeline {
            size,
            camera,
            vertex_buffer,
            camera_buffer,
            objects_buffer,
            sample_count_buffer,
            random_texture,
            random_bind_group,
            camera_bind_group,
            compute1_bind_group1,
            compute2_bind_group1,
            compute1_bind_group2,
            compute2_bind_group2,
            compute_pipeline,
            render_bind_group1,
            render_bind_group2,
            render_pipeline,
            buffer_switch: true,
            sample_count: 0,
        }
    }

    pub fn render<'a>(&'a self, encoder: &mut CommandEncoder, view: &TextureView) {
        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Compute Pass"),
            });
            if self.buffer_switch {
                compute_pass.set_bind_group(0, &self.compute1_bind_group1, &[]);
            } else {
                compute_pass.set_bind_group(0, &self.compute1_bind_group2, &[]);
            }
            compute_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            compute_pass.set_bind_group(2, &self.random_bind_group, &[]);
            if self.buffer_switch {
                compute_pass.set_bind_group(3, &self.compute2_bind_group2, &[]);
            } else {
                compute_pass.set_bind_group(3, &self.compute2_bind_group1, &[]);
            }
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.dispatch_workgroups(self.size.width, self.size.height, 1);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            if self.buffer_switch {
                render_pass.set_bind_group(0, &self.render_bind_group1, &[]);
            } else {
                render_pass.set_bind_group(0, &self.render_bind_group2, &[])
            }
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..NUM_VERTICES, 0..1);
        }
    }

    pub fn camera(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn update_camera(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera.into_uniform()]));
    }
    
    pub fn put_random_texture(&self, queue: &wgpu::Queue) {
        let mut data = vec![0u32; (self.size.width * self.size.height) as usize];
        thread_rng().fill(&mut data[..]);
        queue.write_texture(self.random_texture.as_image_copy(), bytemuck::cast_slice(&data[..]), wgpu::ImageDataLayout{
            offset: 0,
            bytes_per_row: Some(self.size.width * 4),
            rows_per_image: Some(self.size.height),
        }, self.size);
    }

    pub fn switch_buffer(&mut self) {
        self.buffer_switch = !self.buffer_switch;
    }
    
    pub fn reset_sample_count(&mut self, queue: &wgpu::Queue) {
        self.sample_count = 0;
        queue.write_buffer(&self.sample_count_buffer, 0, bytemuck::cast_slice(&[self.sample_count]));
    }

    pub fn increment_sample_count(&mut self, queue: &wgpu::Queue) {
        self.sample_count += 1;
        queue.write_buffer(&self.sample_count_buffer, 0, bytemuck::cast_slice(&[self.sample_count]));
        if self.sample_count % 100 == 0 {
            println!("{}", self.sample_count);
        }
    }
}
