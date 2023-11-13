mod pipeline;

use wgpu::{
    Surface, Device, SurfaceConfiguration, Queue, SurfaceError, Instance, 
    InstanceDescriptor, Backends, RequestAdapterOptions, PowerPreference,
    DeviceDescriptor, Features, Limits, TextureUsages, TextureViewDescriptor,
    CommandEncoderDescriptor,
};
use winit::{
    dpi::{PhysicalSize, PhysicalPosition},
    event::{WindowEvent, VirtualKeyCode, ElementState, KeyboardInput},
    window::Window,
};
use pipeline::Pipeline;


pub struct GpuState {
    surface: Surface,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: PhysicalSize<u32>,
    prev_cursor: Option<PhysicalPosition<f64>>,
    window: Window,
    pipeline: Pipeline,
}

impl GpuState {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    features: Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        Limits::downlevel_webgl2_defaults()
                    } else {
                        Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let pipeline = Pipeline::new(&device, &config, size);

        pipeline.put_random_texture(&queue);

        GpuState {
            surface,
            device,
            queue,
            config,
            size,
            prev_cursor: None,
            window,
            pipeline,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
    
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        let event_captured = match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Up),
                    ..
                },
                ..
            } => {
                self.pipeline.camera().forwards();
                true
            },
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Down),
                    ..
                },
                ..
            } => {
                self.pipeline.camera().backwards();
                true
            },
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Right),
                    ..
                },
                ..
            } => {
                self.pipeline.camera().rightwards();
                true
            },
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Left),
                    ..
                },
                ..
            } => {
                self.pipeline.camera().leftwards();
                true
            },
            WindowEvent::CursorLeft { .. } => {
                self.prev_cursor = None;
                true
            },
            WindowEvent::CursorEntered { .. } => {
                self.prev_cursor = None;
                true
            },
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(s) = self.prev_cursor {
                    let pct_x = (position.x - s.x)/(self.size.width as f64);
                    self.pipeline.camera().rotate_rightwards(pct_x as f32);
                    let pct_y = (position.y - s.y)/(self.size.height as f64);
                    self.pipeline.camera().rotate_upwards(-pct_y as f32);
                }
                self.prev_cursor = Some(*position);
                true
            },
            _ => false,
        };

        if event_captured {
            self.pipeline.reset_sample_count(&self.queue);
        }

        event_captured
    }

    pub fn update(&mut self) {
        self.pipeline.update_camera(&self.queue);
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        
        self.pipeline.switch_buffer();
        self.pipeline.render(&mut encoder, &view);

        // submit will accept anything that implements IntoIter
        self.queue.submit([encoder.finish()]);
        output.present();

        self.pipeline.increment_sample_count(&self.queue);
        Ok(())
    }
}
