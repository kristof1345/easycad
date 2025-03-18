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
use model::line::Line;
// use model::circle::Circle;
use model::line::flatten_lines;
// use model::circle::flatten_circles;

use gui::EguiRenderer;
use gui_elements::GUI;
use egui_wgpu::wgpu::util::DeviceExt;
use egui_winit::winit::{
    event::*,
    event_loop::{EventLoop, ControlFlow},
    window::{Window, WindowBuilder},
};
use winit::keyboard::ModifiersState;
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
    DrawCircle,
}

// #[repr(C)]
// #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
// struct CircleUniform {
//     radius: f32,      // Circle radius
// }

struct State<'a> {
    window: &'a Window,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'a>,
    size: winit::dpi::PhysicalSize<u32>,
    config: wgpu::SurfaceConfiguration,

    render_pipeline: wgpu::RenderPipeline,
    // render_pipeline2: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,

    // circle_buffer: wgpu::Buffer,
    // circle_uniform: CircleUniform,
    // circle_bind_group: wgpu::BindGroup,
    // vertex_buffer_circle: wgpu::Buffer,

    lines: Vec<Line>,
    // circles: Vec<Circle>,

    num_vertices: u32,

    drawing_state: DrawingState,
    mode: Mode,
    cursor_position: Option<[f32; 2]>,
    last_position_for_pan: Option<[f32; 2]>,
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
            label: Some("shader module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("assets/line.wgsl").into()),
        });

        // let circle_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        //     label: Some("shader module"),
        //     source: wgpu::ShaderSource::Wgsl(include_str!("assets/circle.wgsl").into()),
        // });


        // let vertices = Vec::new();
        let lines = Vec::new();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            contents: &[],
        });

        // let vertex_buffer_circle = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("circle vertex buffer"),
        //     usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        //     contents: &[],
        // });

        // let circles = Vec::new();

        // let circle_uniform = CircleUniform {
        //     radius: 50.0,
        // };

        // let circle_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Circle uniform buffer"),
        //     contents: bytemuck::cast_slice(&[circle_uniform]),
        //     usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        // });

        // let circle_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //     label: Some("Circle Bind Group Layout"),
        //     entries: &[wgpu::BindGroupLayoutEntry {
        //         binding: 0,                           // Binding 0 within group 1
        //         visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        //         ty: wgpu::BindingType::Buffer {
        //             ty: wgpu::BufferBindingType::Uniform,
        //             has_dynamic_offset: false,
        //             min_binding_size: None,
        //         },
        //         count: None,
        //     }],
        // });

        // let circle_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     label: Some("Circle Bind Group"),
        //     layout: &circle_bind_group_layout,
        //     entries: &[wgpu::BindGroupEntry {
        //         binding: 0,
        //         resource: circle_buffer.as_entire_binding(),
        //     }],
        // }); 

        let render_pipeline = Pipeline::new(&device, &config, &shader, &camera_bind_group_layout).render_pipeline;
        // let render_pipeline2 = Pipeline::new_circle_pipeline(&device, &config, &circle_shader, &camera_bind_group_layout, &circle_bind_group_layout).render_pipeline;

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
            // render_pipeline2,

            vertex_buffer,
            // circle_uniform,
            // circle_buffer,
            // circle_bind_group,
            // vertex_buffer_circle,

            lines,
            // circles,

            num_vertices: 0,

            drawing_state: DrawingState::Idle,
            mode: Mode::Normal,
            cursor_position: None,
            last_position_for_pan: None,
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

    // pub fn update_circle_vertex_buffer(&mut self) {
    //     self.vertex_buffer_circle = self
    //         .device
    //         .create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //             label: Some("circle vertex buffer"),
    //             usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    //             contents: bytemuck::cast_slice(&flatten_circles(&self.circles)),
    //         });
    // }

    // pub fn update_vertex_buffer_circle(&mut self) {
    //     self.vertex_buffer_circle = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //         label: Some("vertex buffer circle"),
    //         usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    //         contents: bytemuck::cast_slice(&self.circle_vertices),
    //     });
    //     self.num_vertices_circle = self.circle_vertices.len() as u32;

    //     self.index_buffer_circle = self.device.create_buffer_init(
    //         &wgpu::util::BufferInitDescriptor {
    //         label: Some("circle index buffer"),
    //         contents: bytemuck::cast_slice(&self.circle_indices),
    //         usage: wgpu::BufferUsages::INDEX,
    //     });
    //     self.num_indices_circle = self.circle_indices.len() as u32;
    // }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.window().request_redraw();
        input::handle_input(self, event)
    }

    // pub fn draw_circle(&mut self, cx: f32, cy: f32, radius: f32, color: [f32; 3], n: u32) {
    //     for i in 0..n {
    //         let theta = 2.0 * std::f32::consts::PI * (i as f32) / (n as f32);
    //         let x = cx + radius * theta.cos();
    //         let y = cy + radius * theta.sin();
    //         self.circle_vertices.push(
    //             Vertex {
    //                 position: [x, y, 0.0],
    //                 color,
    //             }
    //         );
    //     }

    //     for i in 0..n {
    //         self.circle_indices.push(i as u16);
    //         self.circle_indices.push(((i + 1) % n) as u16);
    //     }

    //     self.update_vertex_buffer_circle();
    // }

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
    
    state.lines.push(Line {
        vertices: [
            Vertex {
                position: [50.0, 0.0, 0.0],
                color: [1.0, 1.0, 1.0],
            },
            Vertex {
                position: [0.0, 0.0, 0.0],
                color: [1.0, 1.0, 1.0],
            },
        ],
    });
    state.lines.push(Line {
        vertices: [
            Vertex {
                position: [0.0, 0.0, 0.0],
                color: [1.0, 1.0, 1.0],
            },
            Vertex {
                position: [0.0, 50.0, 0.0],
                color: [1.0, 1.0, 1.0],
            },
        ],
    });

    // state.circles.push(Circle { radius: 20.0, center: Vertex { position: [0.0, 0.0, 0.0], color: [1.0, 1.0, 1.0] } });

    flatten_lines(&state.lines);
    // flatten_circles(&state.circles);

    state.update_vertex_buffer();
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
                                Mode::DrawCircle => {
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
