use egui_wgpu::wgpu;

mod events;
mod graphics;
mod model;

use crate::model::circle::flatten_circles_to_instances;
use crate::model::line::flatten_lines_to_instances;
use events::input;
use graphics::camera;
use graphics::gui;
use graphics::pipeline::Pipeline;
use graphics::renderer;
use graphics::vertex::Vertex;
use model::circle::Circle;
use model::circle::CircleOps;
use model::line::flatten_lines;
use model::line::Line;

use egui_wgpu::wgpu::util::DeviceExt;
use egui_winit::winit;
use egui_winit::winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use gui::EguiRenderer;
use model::line::LineOps;
use winit::keyboard::ModifiersState;
use winit::window::CursorIcon;

use dxf::entities::EntityType;
use dxf::entities::*;
use dxf::Drawing;

use std::time::Instant as OtherInstant;

use crate::graphics::gui_elements::ColorScheme;
use crate::graphics::gui_elements::UiState;

const AXIS_COORDINATES: [Vertex; 4] = [
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

#[derive(Debug)]
enum DrawingState {
    Idle,
    WaitingForSecondPoint([f32; 2]),
    WaitingForRadius([f32; 2]),
}

#[derive(Debug, PartialEq)]
enum Mode {
    Normal,
    DrawCircle,
    Selection,
    Delete,
    // Measure(Option<[Vertex; 2]>),
    Measure(Option<[f32; 2]>),
    DrawLine(DrawLineMode),
    Move(FuncState),
    Copy(FuncState),
    CreateText,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
enum DrawLineMode {
    Normal,
    Ortho, // 0, 90, 180, 270 degrees
}

#[derive(PartialEq, PartialOrd, Debug)]
enum FuncState {
    Selection,
    Move([f32; 2]),
    Copy([f32; 2]),
    SelectPoint,
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

    ui: UiState,

    // line instance buffer
    instance_buffer: wgpu::Buffer,
    // circle instance buffer
    instance_buffer_circle: wgpu::Buffer,
    axis_vertex_buffer: wgpu::Buffer,

    lines: Vec<Line>,
    // next_line_id: u64,
    active_line_index: Option<usize>,
    active_circle_index: Option<usize>,

    circles: Vec<Circle>,
    indicators: Vec<Line>,
    num_vertices_indicators: u32,

    drawing_state: DrawingState,
    mode: Mode,
    snap: Option<[f32; 2]>,

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

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let lines = Vec::new();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("lines instance buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            contents: &[],
        });

        let circles = Vec::new();
        let instance_buffer_circle = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("circle instance buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            contents: &[],
        });

        let snap = None;

        let mut indicators = Vec::new();

        let axis_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&AXIS_COORDINATES),
        });

        // let index_buffer_circle = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Circle index buffer"),
        //     contents: bytemuck::cast_slice(&circle_indices),
        //     usage: wgpu::BufferUsages::INDEX,
        // });

        let render_pipeline =
            Pipeline::new(&device, &config, &shader, &camera_bind_group_layout).render_pipeline;
        let render_pipeline2 = Pipeline::new_circle_pipeline(
            &device,
            &config,
            &circle_shader,
            &camera_bind_group_layout,
        )
        .render_pipeline;
        let xy_axis_render_pipeline = Pipeline::new_xy_axis_pipeline(
            &device,
            &config,
            &xy_axis_shader,
            &camera_bind_group_layout,
        )
        .render_pipeline;

        let egui = EguiRenderer::new(
            &device,       // wgpu Device
            config.format, // TextureFormat
            None,          // this can be None
            1,             // samples
            window,        // winit Window
        );

        let dragging = false;

        for _i in 0..4 {
            indicators.push(Line {
                vertices: [
                    Vertex {
                        position: [0.0, 0.0, 0.0],
                        color: [1.0, 1.0, 1.0],
                    },
                    Vertex {
                        position: [0.0, 0.0, 0.0],
                        color: [1.0, 1.0, 1.0],
                    },
                ],
                // id: i,
                thickness: 1.0,
                selected: false,
                del: false,
                is_drawing: false,
            });
        }

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

            ui: UiState::new(),

            instance_buffer,
            instance_buffer_circle,
            axis_vertex_buffer,

            lines,
            // next_line_id: 0,
            active_line_index: None,
            active_circle_index: None,

            circles,
            indicators,

            num_vertices_indicators: 0,

            drawing_state: DrawingState::Idle,
            mode: Mode::Normal,
            snap,
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

    // update lines instance buffer
    pub fn update_instance_buffer(&mut self) {
        self.instance_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("instance buffer"),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&flatten_lines_to_instances(
                    &mut self.lines,
                    self.ui.theme.color_scheme,
                )),
            });
    }

    // update circle instance buffer
    pub fn update_circle_instance_buffer(&mut self) {
        self.instance_buffer_circle =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("circle instance buffer"),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    contents: bytemuck::cast_slice(&flatten_circles_to_instances(
                        &mut self.circles,
                        self.ui.theme.color_scheme,
                        self.camera.zoom,
                    )),
                });
    }

    pub fn update_axis_vertex_buffer(&mut self) {
        let flat_indicators = flatten_lines(&mut self.indicators, self.ui.theme.color_scheme);

        let mut out = Vec::with_capacity(4 + flat_indicators.len());
        out.extend_from_slice(&AXIS_COORDINATES);
        out.extend_from_slice(&flat_indicators);

        if self.ui.theme.color_scheme == ColorScheme::Light {
            out[2].color = [0.0, 0.0, 0.0];
            out[3].color = [0.0, 0.0, 0.0];
        }

        self.axis_vertex_buffer =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("axis vertex buffer"),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    contents: bytemuck::cast_slice(&out),
                });
        self.num_vertices_indicators = out.len() as u32;
    }

    pub fn move_indicators_to_cursor(&mut self, point: [f32; 2]) {
        let treshold: f32 = 10.0;

        for i in 0..4 {
            let base_x = point[0] * self.camera.zoom;
            let base_y = point[1] * self.camera.zoom;

            let (dx1, dy1, dx2, dy2) = match i {
                0 => (1.0, 1.0, -1.0, 1.0),
                1 => (-1.0, 1.0, -1.0, -1.0),
                2 => (1.0, -1.0, -1.0, -1.0),
                3 => (1.0, 1.0, 1.0, -1.0),
                _ => return,
            };

            self.indicators[i].vertices[0].position[0] = base_x + dx1 * treshold;
            self.indicators[i].vertices[0].position[1] = base_y + dy1 * treshold;
            self.indicators[i].vertices[1].position[0] = base_x + dx2 * treshold;
            self.indicators[i].vertices[1].position[1] = base_y + dy2 * treshold;
        }
    }

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
        let time_to_load_drawing = OtherInstant::now();
        println!("loading...");
        let drawing = Drawing::load_file(file_path)?;
        println!("drawing took: {:?}", time_to_load_drawing.elapsed());

        let now = OtherInstant::now();
        for e in drawing.entities() {
            println!("entity: {:?}", e);
            match e.specific {
                EntityType::Line(ref line) => {
                    self.add_line(
                        [line.p1.x as f32, line.p1.y as f32],
                        [line.p2.x as f32, line.p2.y as f32],
                        false,
                    );
                }
                EntityType::Circle(ref circle) => {
                    self.add_circle(
                        [circle.center.x as f32, circle.center.y as f32],
                        circle.radius as f32,
                        [1.0, 1.0, 1.0],
                        false,
                        false,
                        false,
                    );
                }
                _ => {}
            }
        }

        println!("{:?}", self.active_line_index);
        println!("⏱ now took: {:?}", now.elapsed());

        Ok(())
    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("easycad")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)
        .unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut state = State::new(&window).await;

    state.update_instance_buffer();
    state.update_circle_instance_buffer();
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
                                WindowEvent::RedrawRequested => match state.render() {
                                    Ok(_) => {}
                                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                                    Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                                    Err(e) => eprintln!("{:?}", e),
                                },
                                _ => {}
                            };

                            match state.mode {
                                Mode::Normal => {
                                    state.window.set_cursor_icon(CursorIcon::Default);
                                }
                                Mode::DrawLine(DrawLineMode::Normal)
                                | Mode::DrawLine(DrawLineMode::Ortho) => {
                                    state.window.set_cursor_icon(CursorIcon::Crosshair);
                                }
                                Mode::DrawCircle => {
                                    state.window.set_cursor_icon(CursorIcon::Crosshair);
                                }
                                Mode::Selection => {
                                    state.window.set_cursor_icon(CursorIcon::Pointer);
                                }
                                Mode::Move(_) => {
                                    state.window.set_cursor_icon(CursorIcon::Move);
                                }
                                Mode::Copy(_) => {
                                    state.window.set_cursor_icon(CursorIcon::Copy);
                                }
                                Mode::Delete => {
                                    state.window.set_cursor_icon(CursorIcon::Default);
                                }
                                Mode::Measure(_) => {
                                    state.window.set_cursor_icon(CursorIcon::Crosshair);
                                }
                                Mode::CreateText => {
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
