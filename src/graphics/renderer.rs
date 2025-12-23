use crate::graphics::gui_elements::UiAction;
use crate::DrawLineMode;
use crate::Mode;
use crate::State;
use egui_wgpu::wgpu;
use egui_wgpu::ScreenDescriptor;
use std::iter;

pub fn render(state: &mut State) -> Result<(), wgpu::SurfaceError> {
    let frame = match state.surface.get_current_texture() {
        Ok(frame) => frame,
        Err(wgpu::SurfaceError::Lost) => {
            state.resize(state.size);
            return Ok(());
        },
        Err(wgpu::SurfaceError::Outdated) => {
            return Ok(());
        }
        Err(wgpu::SurfaceError::Timeout) => {
            eprintln!("Surface timeout!");
            return Ok(());
        }
        Err(wgpu::SurfaceError::OutOfMemory) => {
            panic!("Out of memory!");
        }
    };

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
        
        state.update_axis_vertex_buffer();
        // println!("{:?}", state.num_vertices_indicators);

        // update the draw method to get the right amount of indicies once we get the logic of the indicators down
        render_pass.set_pipeline(&state.xy_axis_render_pipeline);
        render_pass.set_vertex_buffer(0, state.axis_vertex_buffer.slice(..));
        render_pass.set_bind_group(0, &state.camera_bind_group, &[]);
        render_pass.draw(0..state.num_vertices_indicators, 0..1);

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

    let State {ui, egui, device, queue, window, ..} = state;

    egui.draw(
        device,
        queue,
        &mut encoder,
        window,
        &view,
        screen_descriptor,
        |ui_ctx| {
            ui.gui(ui_ctx)
        },
    );

    if let Some(action) = ui.action.take() {
        match action {
            UiAction::DrawLine => {
                state.mode = Mode::DrawLine(DrawLineMode::Normal);
            }
            UiAction::DrawCircle => {
                state.mode = Mode::DrawCircle;
            }
            UiAction::OpenFilePath(file_path) => {
                let loaded = state.load_from_dxf(file_path);

                match loaded {
                    Ok(_) => println!("loaded file"),
                    Err(error) => eprintln!("i/o error while loading file: {}", error),
                };
            }
            UiAction::SaveFile => {
                let saved = state.save_to_dxf();
                match saved {
                    Ok(_) => println!("file saved"),
                    Err(error) => eprintln!("i/o error while saving file: {}", error),
                };
            }
            UiAction::Input(value) => {
                println!("value we got: {:?}", value);
            }
        }
    }

    state.queue.submit(iter::once(encoder.finish()));
    frame.present();

    Ok(())
}
