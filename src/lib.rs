use egui_wgpu::wgpu;

mod graphics;
mod events;
mod model;

use graphics::pipeline::Pipeline;
use graphics::vertex::Vertex;
use graphics::gui;
use graphics::gui_elements;
use graphics::renderer;
use events::input;

use gui::EguiRenderer;
use gui_elements::GUI;
use egui_wgpu::wgpu::util::DeviceExt;
use egui_winit::winit::{
    event::*,
    event_loop::{EventLoop, ControlFlow},
    window::{Window, WindowBuilder},
};
use winit::window::CursorIcon;
use egui_winit::winit;

#[derive(Debug)]
enum DrawingState {
    Idle,
    WaitingForSecondPoint([f32; 2]),
}

#[derive(Debug,PartialEq, Eq)]
enum Mode {
    Normal,
    DrawLine,
}

struct State<'a> {
    window: &'a Window,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'a>,
    size: winit::dpi::PhysicalSize<u32>,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    zoom_buffer: wgpu::Buffer,
    vertices: Vec<Vertex>,
    num_vertices: u32,
    drawing_state: DrawingState,
    mode: Mode,
    cursor_position: Option<[f32; 2]>,
    zoom: f32,
    zoom_bind_group: wgpu::BindGroup,
    egui: EguiRenderer,
}

impl<'a> State<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: wgpu::InstanceFlags::default(),
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    // memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let zoom: f32 = 1.0;

        let zoom_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zoom buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&[zoom]),
        });

        let zoom_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zoom bind group layout"),
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
        });

        let zoom_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zoom bind group"),
            layout: &zoom_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: zoom_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("assets/line.wgsl").into()),
        });

        let render_pipeline = Pipeline::new(&device, &config, &shader, &zoom_bind_group_layout).render_pipeline;

        let vertices = Vec::new();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&vertices),
        });
        
        let mut egui = EguiRenderer::new(
            &device,       // wgpu Device
            config.format, // TextureFormat
            None,          // this can be None
            1,             // samples
            window,       // winit Window
        );

        Self {
            window,
            queue,
            device,
            size,
            surface,
            config,
            render_pipeline,
            vertex_buffer,
            vertices,
            num_vertices: 0,
            drawing_state: DrawingState::Idle,
            mode: Mode::Normal,
            cursor_position: None,
            zoom,
            zoom_buffer,
            zoom_bind_group,
            egui,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn update_vertex_buffer(&mut self) {
        self.vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vertex buffer"),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&self.vertices),
            });
        self.num_vertices = self.vertices.len() as u32;
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.window().request_redraw();
        input::handle_input(self, event)
    }


    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        renderer::render(self)
    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().with_title("cad").with_inner_size(winit::dpi::LogicalSize::new(800, 600)).build(&event_loop).unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut state = State::new(&window).await;

    event_loop
        .run(move |event, control_flow| {
            match event {
                Event::WindowEvent { event, window_id } if window_id == state.window().id() => {
                    state.egui.handle_input(&mut state.window, &event);
                    let egui_wants_pointer = state.egui.context.wants_pointer_input();
                    let egui_wants_keyboard = state.egui.context.wants_keyboard_input();

                    let should_skip = match event {
                        WindowEvent::CursorMoved { .. } 
                        | WindowEvent::MouseInput { .. } 
                        | WindowEvent::MouseWheel { .. } => egui_wants_pointer,
                        WindowEvent::KeyboardInput { .. } => egui_wants_keyboard,
                        _ => false, // Let other events through
                    };

                    if !should_skip {
                        if !state.input(&event) {
                            match event {
                                WindowEvent::CloseRequested => {
                                    println!("adios");
                                    control_flow.exit();
                                }
                                WindowEvent::Resized(new_size) => {
                                    state.resize(new_size);
                                }
                                WindowEvent::RedrawRequested => {
                                    match state.render() {
                                        Ok(_) => {}
                                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                                        Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                                        Err(e) => eprintln!("{:?}", e),
                                    }
                                }
                                _ => {}
                            };

                            match state.mode {
                                Mode::Normal => {
                                    state.window.set_cursor_icon(CursorIcon::Default);
                                }
                                Mode::DrawLine => {
                                    state.window.set_cursor_icon(CursorIcon::Crosshair);
                                }
                            }

                            // state.egui.handle_input(&mut state.window, &event);
                        }
                    }
                }
                _ => {}
            }
        })
        .unwrap();
}
