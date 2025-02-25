use crate::model::line::LineOps;
use crate::DrawingState; // Import the enum if it's in another module
use crate::Mode;
use crate::State;
use winit::event::KeyEvent;
// Import your struct
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::KeyCode;
use winit::keyboard::PhysicalKey;

pub fn handle_input(state: &mut State, event: &WindowEvent) -> bool {
    match event {
        WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    state: ElementState::Pressed,
                    physical_key: PhysicalKey::Code(keycode),
                    ..
                },
            ..
        } => {
            match keycode {
                KeyCode::KeyL => {
                    if state.mode == Mode::Normal {
                        state.mode = Mode::DrawLine;
                        // state.window.set_cursor_icon(CursorIcon::Crosshair);
                    }
                }
                KeyCode::Escape => {
                    if !(state.mode == Mode::Normal) {
                        state.mode = Mode::Normal;
                        // state.window.set_cursor_icon(CursorIcon::Default);
                    }
                }
                _ => {}
            }

            true
        }

        WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Middle,
            ..
        } => {
            state.dragging = true; // Start panning
            true
        }
        WindowEvent::MouseInput {
            state: ElementState::Released,
            button: MouseButton::Middle,
            ..
        } => {
            state.dragging = false; // Stop panning
            true
        }

        // Pan when mouse moves while dragging
        WindowEvent::CursorMoved { position, .. } if state.dragging => {
            let dx = (position.x as f32 - state.last_mouse_x) / state.config.width as f32 * 2.0;
            let dy = (position.y as f32 - state.last_mouse_y) / state.config.height as f32 * 2.0;
            state.camera.pan(dx, -dy);

            state.queue.write_buffer(
                &state.camera_buffer,
                0,
                bytemuck::cast_slice(&state.camera.to_matrix()),
            );

            state.last_mouse_x = position.x as f32;
            state.last_mouse_y = position.y as f32;
            true
        }

        WindowEvent::CursorMoved { position, .. } => {
            let x = (2.0 * position.x as f32 / state.size.width as f32) - 1.0;
            let y = 1.0 - (2.0 * position.y as f32 / state.size.height as f32);
            state.cursor_position = Some([x, y]);

            if let DrawingState::WaitingForSecondPoint(_start_pos) = state.drawing_state {
                state.update_line([x, y]);
            }

            true
        }
        WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Left,
            ..
        } => {
            if let Some(position) = state.cursor_position {
                match state.drawing_state {
                    DrawingState::Idle => match state.mode {
                        Mode::DrawLine => {
                            state.drawing_state = DrawingState::WaitingForSecondPoint(position);
                            state.add_line(position, position);
                        }
                        _ => {}
                    },
                    DrawingState::WaitingForSecondPoint(_start_pos) => match state.mode {
                        Mode::DrawLine => {
                            state.update_line(position);
                            // println!("{:#?}", state.vertices);
                            state.drawing_state = DrawingState::Idle;
                        }
                        _ => {}
                    },
                }
            }
            true
        }
        WindowEvent::MouseWheel { delta, .. } => {
            let zoom_speed = 0.1;
            match delta {
                MouseScrollDelta::LineDelta(_, y) => {
                    state.camera.zoom(1.0 + zoom_speed * y.signum());
                }
                MouseScrollDelta::PixelDelta(pos) => {
                    state.camera.zoom(1.0 + zoom_speed * pos.y.signum() as f32);
                }
            }

            // state.zoom = state.zoom.clamp(0.1, 10.0);
            state.queue.write_buffer(
                &state.camera_buffer,
                0,
                bytemuck::cast_slice(&state.camera.to_matrix()),
            );

            true
        }
        // WindowEvent::PanGesture { delta, .. } => {
        //     println!("heyho");
        //     true
        // }
        _ => false,
    }
}
