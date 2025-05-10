use egui_wgpu::wgpu;

mod graphics;
mod events;
mod model;

use graphics::pipeline::Pipeline;
use graphics::vertex::Vertex;
use graphics::gui;
use graphics::gui_elements;
use graphics::renderer;
use graphics::camera;
use events::input;
use model::circle::CircleOps;
use model::line::Line;
use model::circle::Circle;
use model::line::flatten_lines;
use model::circle::flatten_circles;

use gui::EguiRenderer;
use gui_elements::GUI;
use egui_wgpu::wgpu::util::DeviceExt;
use egui_winit::winit::{
    event::*,
    event_loop::{EventLoop, ControlFlow},
    window::{Window, WindowBuilder},
};
use model::line::LineOps;
use winit::keyboard::ModifiersState;
use winit::window::CursorIcon;
use egui_winit::winit;

use dxf::Drawing;
use dxf::entities::*;
use dxf::entities::EntityType;

#[derive(Debug)]
enum DrawingState {
    Idle,
    WaitingForSecondPoint([f32; 2]),
    WaitingForRadius([f32; 2]),
}

#[derive(Debug, PartialEq, Eq)]
enum Mode {
    Normal,
    DrawLine(DrawLineMode),
    DrawCircle,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
enum DrawLineMode {
    Normal,
    Ortho, // 0, 90, 180, 270 degrees
}

struct State<'a> {
    window: &'a Window,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'a>,
    size: winit::dpi::PhysicalSize<u32>,
    config: wgpu::SurfaceConfiguration,

    render_pipeline: wgpu::RenderPipeline,
    render_pipeline2: wgpu::RenderPipeline,
    xy_axis_render_pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    vertex_buffer_circle: wgpu::Buffer,
    index_buffer_circle: wgpu::Buffer,
    axis_vertex_buffer: wgpu::Buffer,

    lines: Vec<Line>,
    circles: Vec<Circle>,
    circle_indices: Vec<u32>,
    // xy_axis: Vec<Line>,

    num_vertices: u32,
    num_vertices_circle: u32,
    // num_vertices_xy_axis: u32,

    drawing_state: DrawingState,
    mode: Mode,

    // cursor position in world coordinates
    cursor_position: Option<[f32; 2]>,
    last_position_for_pan: Option<[f32; 2]>,
    last_screen_position_for_pan: Option<[f32; 2]>,
    egui: EguiRenderer,
    camera: camera::Camera,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    dragging: bool,
    modifiers: ModifiersState,
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

        let camera = camera::Camera::new(0.0, 0.0, 1.0);

        // let matrix = camera.to_matrix();
        let uniform = camera.to_uniform(config.width as f32, config.height as f32);
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&[uniform]),
        });

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera bind group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("line shader module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("assets/line.wgsl").into()),
        });

        let circle_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("circle shader module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("assets/circle.wgsl").into()),
        });

        let xy_axis_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("x y axis shader module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("assets/xy_axis.wgsl").into()),
        });

        // let vertices = Vec::new();
        let lines = Vec::new();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            contents: &[],
        });

        let circles = Vec::new();
        let vertex_buffer_circle = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("circle vertex buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            contents: &[],
        });

        let axis_coordinates = [
            Vertex {
                position: [50.1, 0.0, 0.0],
                color: [255.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.0, 0.0, 0.0],
                color: [255.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.0, 50.1, 0.0],
                color: [1.0, 1.0, 1.0],
            },
            Vertex {
                position: [0.0, 0.0, 0.0],
                color: [1.0, 1.0, 1.0],
            },
        ];

        // let xy_axis = Vec::new();
        let axis_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&axis_coordinates),
        });

        let circle_indices = Vec::new();
        let index_buffer_circle = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Circle index buffer"),
                contents: bytemuck::cast_slice(&circle_indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let render_pipeline = Pipeline::new(&device, &config, &shader, &camera_bind_group_layout).render_pipeline;
        let render_pipeline2 = Pipeline::new_circle_pipeline(&device, &config, &circle_shader, &camera_bind_group_layout).render_pipeline;
        let xy_axis_render_pipeline = Pipeline::new_xy_axis_pipeline(&device, &config, &xy_axis_shader, &camera_bind_group_layout).render_pipeline;

        let egui = EguiRenderer::new(
            &device,       // wgpu Device
            config.format, // TextureFormat
            None,          // this can be None
            1,             // samples
            window,       // winit Window
        );

        let dragging = false;
        
        Self {
            window,
            queue,
            device,
            size,
            surface,
            config,

            render_pipeline,
            render_pipeline2,
            xy_axis_render_pipeline,

            vertex_buffer,
            vertex_buffer_circle,
            circle_indices,
            axis_vertex_buffer,

            lines,
            circles,
            index_buffer_circle,
            // xy_axis,

            num_vertices: 0,
            num_vertices_circle: 0,
            // num_vertices_xy_axis: 0,

            drawing_state: DrawingState::Idle,
            mode: Mode::Normal,
            cursor_position: None,
            last_position_for_pan: None,
            last_screen_position_for_pan: None,
            egui,
            camera,
            camera_buffer,
            camera_bind_group,
            dragging,
            modifiers: ModifiersState::empty(),
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
                contents: bytemuck::cast_slice(&flatten_lines(&self.lines)),
            });
        self.num_vertices = (self.lines.len() as u32) * 2;
    }

    pub fn update_circle_vertex_buffer(&mut self) {
        self.vertex_buffer_circle = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("circle vertex buffer"),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&flatten_circles(&self.circles)),
            });
        self.num_vertices_circle = (self.circles.len() as u32) * 36; // 37 because of the last closing vertex

        self.index_buffer_circle = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Circle Index Buffer"),
            contents: bytemuck::cast_slice(&self.circle_indices),
            usage: wgpu::BufferUsages::INDEX,
        });
    }

    // pub fn update_axis_vertex_buffer(&mut self) {
    //     self.axis_vertex_buffer = self
    //         .device
    //         .create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //             label: Some("axis vertex buffer"),
    //             usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    //             contents: bytemuck::cast_slice(&flatten_lines(&self.xy_axis)),
    //         });
    //     self.num_vertices = (self.xy_axis.len() as u32) * 2;
    // }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.window().request_redraw();
        input::handle_input(self, event)
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        renderer::render(self)
    }

    pub fn save_to_dxf(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut drawing = Drawing::new();

        for line_entity in &self.lines {
            let start_position = line_entity.vertices[0].position;
            let end_position = line_entity.vertices[1].position;

            let line = dxf::entities::Line::new(
                dxf::Point::new(start_position[0] as f64, start_position[1] as f64, 0.0),
                dxf::Point::new(end_position[0] as f64, end_position[1] as f64, 0.0),
            );

            drawing.add_entity(Entity::new(EntityType::Line(line)));
        }

        drawing.save_file("C:/Users/krist/Desktop/test.dxf")?;

        Ok(())
    }

    pub fn load_from_dxf(&mut self, file_path: String) -> Result<(), Box<dyn std::error::Error>> {
        let drawing = Drawing::load_file(file_path)?;
        // let drawing = Drawing::load_file("C:/Users/krist/Documents/load_test.dxf")?;

        for e in drawing.entities() {
            match e.specific {
                EntityType::Line(ref line) => {
                    self.add_line([line.p1.x as f32, line.p1.y as f32], [line.p2.x as f32, line.p2.y as f32]);
                }
                EntityType::Circle(ref circle) => {
                    self.add_circle([circle.center.x as f32, circle.center.y as f32], circle.radius as f32, [1.0, 1.0, 1.0]);
                }
                _ => {}
            }
        }

        Ok(())
    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().with_title("cad").with_inner_size(winit::dpi::LogicalSize::new(800, 600)).build(&event_loop).unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut state = State::new(&window).await;
    
    // state.xy_axis.push(Line {
    //     vertices: [
    //         Vertex {
    //             position: [50.0, 0.0, 0.0],
    //             color: [1.0, 1.0, 1.0],
    //         },
    //         Vertex {
    //             position: [0.0, 0.0, 0.0],
    //             color: [1.0, 1.0, 1.0],
    //         },
    //     ],
    // });
    // state.xy_axis.push(Line {
    //     vertices: [
    //         Vertex {
    //             position: [0.0, 0.0, 0.0],
    //             color: [1.0, 1.0, 1.0],
    //         },
    //         Vertex {
    //             position: [0.0, 50.0, 0.0],
    //             color: [1.0, 1.0, 1.0],
    //         },
    //     ],
    // });

   // state.add_circle([0.0, 0.0], 60.0, [1.0, 1.0, 1.0]);
   // state.add_circle([200.0, 30.0], 50.0, [1.0, 1.0, 1.0]);

    state.update_vertex_buffer();
    state.update_circle_vertex_buffer();
    // state.update_axis_vertex_buffer();
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
                                Mode::DrawLine(DrawLineMode::Normal) | Mode::DrawLine(DrawLineMode::Ortho) => {
                                    state.window.set_cursor_icon(CursorIcon::Crosshair);
                                }
                                Mode::DrawCircle => {
                                    state.window.set_cursor_icon(CursorIcon::Crosshair);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        })
        .unwrap();
}
