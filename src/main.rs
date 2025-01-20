use glutin::dpi::LogicalSize;
use glutin::event::{Event, Event::WindowEvent};
use glutin::event_loop::ControlFlow;
use glutin::ContextBuilder;
use glutin::GlRequest;

fn main() {
    let event_loop = glutin::event_loop::EventLoop::new();
    let window = glutin::window::WindowBuilder::new()
        .with_title("EasyCAD")
        .with_inner_size(LogicalSize::new(800.0, 600.0));

    let gl_context = ContextBuilder::new()
        .with_gl(GlRequest::Latest)
        .build_windowed(window, &event_loop)
        .expect("Cannot create window context");

    let gl_context = unsafe {
        gl_context
            .make_current()
            .expect("failed to make context current");
    };

    gl::load_with(|symbol| gl_context.get_proc_address(symbol) as *const _);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::LoopDestroyed => {}
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            _ => (),
        }
    })
}
