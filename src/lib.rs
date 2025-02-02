use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use winit::window::Window;

struct State<'a> {
    window: &'a Window,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'a>,
    size: winit::dpi::PhysicalSize<u32>,
}

impl<'a> State<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: wgpu::InstanceFlags::default(),
            ..Default::default() // easier to do than writing everything out...
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
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .unwrap();

        Self {
            window,
            queue,
            device,
            size,
            surface,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    event_loop.set_control_flow(ControlFlow::Wait);

    let state = State::new(&window).await;

    event_loop
        .run(move |event, control_flow| match event {
            Event::WindowEvent { event, window_id } if window_id == state.window().id() => {
                match event {
                    WindowEvent::CloseRequested => {
                        println!("adios");
                        control_flow.exit();
                    }
                    _ => {}
                }
            }
            _ => {}
        })
        .unwrap();
}
