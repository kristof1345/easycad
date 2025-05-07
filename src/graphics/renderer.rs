use crate::graphics::gui_elements::UiAction;
use crate::Mode;
use crate::State;
use crate::GUI;
// use bytemuck::fill_zeroes;
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
                        r: 5.0 / 255.0,
                        g: 8.0 / 255.0,
                        b: 12.0 / 255.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&state.xy_axis_render_pipeline);
        render_pass.set_vertex_buffer(0, state.axis_vertex_buffer.slice(..));
        render_pass.set_bind_group(0, &state.camera_bind_group, &[]);
        render_pass.draw(0..4, 0..1);

        render_pass.set_pipeline(&state.render_pipeline);
        render_pass.set_vertex_buffer(0, state.vertex_buffer.slice(..));
        render_pass.set_bind_group(0, &state.camera_bind_group, &[]);
        render_pass.draw(0..state.num_vertices, 0..1);

        render_pass.set_pipeline(&state.render_pipeline2);
        render_pass.set_vertex_buffer(0, state.vertex_buffer_circle.slice(..));
        render_pass.set_index_buffer(
            state.index_buffer_circle.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.set_bind_group(0, &state.camera_bind_group, &[]);
        // render_pass.draw(0..state.num_vertices_circle, 0..1);
        render_pass.draw_indexed(0..state.circle_indices.len() as u32, 0, 0..1);
    }

    let screen_descriptor = ScreenDescriptor {
        size_in_pixels: [state.config.width, state.config.height],
        pixels_per_point: state.window().scale_factor() as f32,
    };

    let mode_flag = &mut state.mode;
    let mut load_flag = false;
    let mut load_path: String = String::new();

    state.egui.draw(
        &state.device,
        &state.queue,
        &mut encoder,
        &state.window,
        &view,
        screen_descriptor,
        |ui| {
            if let Some(action) = GUI(ui) {
                match action {
                    UiAction::DrawLine => {
                        *mode_flag = Mode::DrawLine;
                    }
                    UiAction::DrawCircle => {
                        *mode_flag = Mode::DrawCircle;
                    }
                    UiAction::OpenFilePath(file_path) => {
                        load_flag = true;
                        load_path = file_path;
                    }
                }
            }
        },
    );

    if load_flag {
        let _ = state.load_from_dxf(load_path);
    }

    // if let Some(action) = gui_action {
    //     match action {
    //         GuiAction::ToggleLine => {
    //             state.show_line = !state.show_line;
    //         }
    //     }
    // }

    state.queue.submit(iter::once(encoder.finish()));
    frame.present();

    Ok(())
}
