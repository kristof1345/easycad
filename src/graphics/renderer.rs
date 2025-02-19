use crate::State;
// use egui_wgpu::ScreenDescriptor;

pub fn render(state: &mut State) {
    let frame = state.surface.get_current_texture().unwrap();

    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Command Encoder"),
        });

    // // Start egui frame
    // let raw_input = state.ui.state.take_egui_input(&state.window);
    // state.ui.ctx.begin_frame(raw_input);

    // // Create your egui UI here
    // egui::Window::new("Controls").show(&state.ui.ctx, |ui| {
    //     ui.label("Drawing Tools");
    //     if ui.button("Clear").clicked() {
    //         state.vertices.clear();
    //     }
    //     ui.add(egui::Slider::new(&mut state.zoom, 0.1..=5.0).text("Zoom"));
    //     // Add more UI elements as needed
    // });

    // // End egui frame
    // let full_output = state.ui.ctx.end_frame();
    // let paint_jobs = state.ui.ctx.tessellate(full_output.shapes, 1.0);

    // // Create the screen descriptor for egui
    // let screen_descriptor = ScreenDescriptor {
    //     size_in_pixels: [state.config.width, state.config.height],
    //     pixels_per_point: state.window.scale_factor() as f32,
    // };

    // state.ui.renderer.update_buffers(
    //     &state.device,
    //     &state.queue,
    //     &mut encoder,
    //     &paint_jobs,
    //     &screen_descriptor,
    // );

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

    state.queue.submit(Some(encoder.finish()));
    frame.present();
}
