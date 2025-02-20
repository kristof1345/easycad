use crate::State;
use crate::GUI;
use egui_wgpu::wgpu;
use egui_wgpu::ScreenDescriptor;
use std::iter;

pub fn render(state: &mut State) -> Result<(), wgpu::SurfaceError> {
    let frame = state.surface.get_current_texture().unwrap();

    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Command Encoder"),
        });

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.3,
                        b: 0.4,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&state.render_pipeline);
        render_pass.set_vertex_buffer(0, state.vertex_buffer.slice(..));
        render_pass.set_bind_group(0, &state.zoom_bind_group, &[]);
        render_pass.draw(0..state.num_vertices, 0..1);
    }

    let screen_descriptor = ScreenDescriptor {
        size_in_pixels: [state.config.width, state.config.height],
        pixels_per_point: state.window().scale_factor() as f32,
    };

    state.egui.draw(
        &state.device,
        &state.queue,
        &mut encoder,
        &state.window,
        &view,
        screen_descriptor,
        |ui| GUI(ui),
    );

    state.queue.submit(iter::once(encoder.finish()));
    frame.present();

    Ok(())
}
