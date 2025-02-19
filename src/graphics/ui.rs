// use egui_winit::State as EguiWinitState;
// use winit::window::Window;

// pub struct Ui {
//     pub renderer: egui_wgpu::Renderer,
//     pub state: EguiWinitState,
//     pub ctx: egui::Context,
// }

// impl Ui {
//     pub fn new(
//         window: &Window,
//         device: &wgpu::Device,
//         surface_format: wgpu::TextureFormat,
//     ) -> Self {
//         let ctx = egui::Context::default();

//         // let renderer_format = match surface_format {
//         //     wgpu::TextureFormat::Bgra8UnormSrgb => egui_wgpu::wgpu::TextureFormat::Bgra8UnormSrgb,
//         //     wgpu::TextureFormat::Rgba8UnormSrgb => egui_wgpu::wgpu::TextureFormat::Rgba8UnormSrgb,
//         //     // Add other formats as needed
//         //     _ => egui_wgpu::wgpu::TextureFormat::Bgra8UnormSrgb, // fallback
//         // };

//         // let renderer_device =
//         //     unsafe { std::mem::transmute::<&wgpu::Device, &egui_wgpu::wgpu::Device>(device) };

//         let renderer = egui_wgpu::Renderer::new(device, surface_format, None, 1);

//         let state = EguiWinitState::new(&window);
//         // let state = EguiWinitState::new(
//         //     ctx.clone(),
//         //     ctx.viewport_id(),
//         //     window,
//         //     Some(window.scale_factor() as f32),
//         //     None,
//         // );

//         Self {
//             renderer,
//             state,
//             ctx,
//         }
//     }
// }
