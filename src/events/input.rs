use crate::model::circle::CircleOps;
use crate::model::line::LineOps;
use crate::DrawLineMode;
use crate::DrawingState;
use crate::Mode;
use crate::State;
use winit::event::KeyEvent;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::KeyCode;
use winit::keyboard::PhysicalKey;

pub fn handle_input(state: &mut State, event: &WindowEvent) -> bool {
    match event {
        WindowEvent::ModifiersChanged(modifiers) => {
            state.modifiers = modifiers.state();
            false
        }

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
                        state.mode = Mode::DrawLine(DrawLineMode::Normal);
                    }
                }
                KeyCode::KeyC => {
                    if state.mode == Mode::Normal {
                        state.mode = Mode::DrawCircle;
                    }
                }
                KeyCode::KeyS => {
                    if state.modifiers.control_key() {
                        let saved = state.save_to_dxf();

                        match saved {
                            Ok(_) => println!("file saved"),
                            Err(error) => eprintln!("i/o error while saving file: {}", error),
                        };
                    }
                }
                KeyCode::KeyO => {
                    if state.modifiers.control_key() {
                        let mut path: String = String::new();

                        if let Some(file_path) = rfd::FileDialog::new().pick_file() {
                            path = file_path.display().to_string();
                        }

                        let loaded = state.load_from_dxf(path);

                        match loaded {
                            Ok(_) => println!("loaded file"),
                            Err(error) => eprintln!("i/o error while loading file: {}", error),
                        };
                    }
                    if state.mode == Mode::DrawLine(DrawLineMode::Normal) {
                        state.mode = Mode::DrawLine(DrawLineMode::Ortho);
                    } else if state.mode == Mode::DrawLine(DrawLineMode::Ortho) {
                        state.mode = Mode::DrawLine(DrawLineMode::Normal);
                    }
                }
                KeyCode::Escape => {
                    if state.mode == Mode::DrawLine(DrawLineMode::Normal)
                        || state.mode == Mode::DrawLine(DrawLineMode::Ortho)
                    {
                        if let DrawingState::WaitingForSecondPoint(_start_pos) = state.drawing_state
                        {
                            state.cancel_drawing_line();
                        }
                    }

                    if state.mode == Mode::DrawCircle {
                        if let DrawingState::WaitingForRadius(_start_pos) = state.drawing_state {
                            state.cancel_drawing_circle();
                        }
                    }
                    state.mode = Mode::Normal;
                    state.drawing_state = DrawingState::Idle;
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
            state.dragging = true;
            true
        }

        WindowEvent::MouseInput {
            state: ElementState::Released,
            button: MouseButton::Middle,
            ..
        } => {
            state.dragging = false;
            true
        }

        // Panning implementation
        WindowEvent::CursorMoved { position, .. }
            if state.dragging || state.modifiers.control_key() =>
        {
            let cen_x = position.x as f32 - (state.size.width as f32 / 2.0);
            let cen_y = (state.size.height as f32 / 2.0) - position.y as f32;

            let zoom = state.camera.zoom;
            let world_x = cen_x / zoom;
            let world_y = cen_y / zoom;

            if let Some(last_position) = state.last_position_for_pan {
                let dx = world_x - last_position[0];
                let dy = world_y - last_position[1];
                state.camera.pan(-dx, -dy);

                let uniform = state
                    .camera
                    .to_uniform(state.config.width as f32, state.config.height as f32);

                state
                    .queue
                    .write_buffer(&state.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));

                state.window.request_redraw();
            }
            state.last_position_for_pan = Some([world_x, world_y]);

            true
        }

        // converting cursor position into world space and storing it
        WindowEvent::CursorMoved { position, .. } => {
            state.last_screen_position_for_pan = Some([position.x as f32, position.y as f32]);

            let cen_x = position.x as f32 - (state.size.width as f32 / 2.0);
            let cen_y = (state.size.height as f32 / 2.0) - position.y as f32;

            let zoom = state.camera.zoom;
            let pan_x = state.camera.x_offset;
            let pan_y = state.camera.y_offset;
            let world_x = cen_x / zoom;
            let world_y = cen_y / zoom;
            let world_x_pan = cen_x / zoom + pan_x;
            let world_y_pan = cen_y / zoom + pan_y;

            state.cursor_position = Some([world_x_pan, world_y_pan]);
            state.last_position_for_pan = Some([world_x, world_y]);

            if let DrawingState::WaitingForSecondPoint(_start_pos) = state.drawing_state {
                state.update_line([world_x_pan, world_y_pan]);
            }
            if let DrawingState::WaitingForRadius(_start_pos) = state.drawing_state {
                state.update_circle([world_x_pan, world_y_pan]);
            }

            true
        }

        WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Left,
            ..
        } if state.mode != Mode::Normal => {
            if let Some(position) = state.cursor_position {
                match state.drawing_state {
                    DrawingState::Idle => match state.mode {
                        Mode::DrawLine(DrawLineMode::Normal)
                        | Mode::DrawLine(DrawLineMode::Ortho) => {
                            state.drawing_state = DrawingState::WaitingForSecondPoint(position);
                            state.add_line(position, position);
                        }
                        Mode::DrawCircle => {
                            state.drawing_state = DrawingState::WaitingForRadius(position);
                            state.add_circle(position, 0.0, [1.0, 1.0, 1.0]);
                        }
                        _ => {}
                    },
                    DrawingState::WaitingForSecondPoint(_start_pos) => match state.mode {
                        Mode::DrawLine(DrawLineMode::Normal)
                        | Mode::DrawLine(DrawLineMode::Ortho) => {
                            state.update_line(position);
                            state.drawing_state = DrawingState::Idle;
                        }
                        _ => {}
                    },
                    DrawingState::WaitingForRadius(_start_pos) => match state.mode {
                        Mode::DrawCircle => {
                            state.update_circle(position);
                            state.drawing_state = DrawingState::Idle;
                        }
                        _ => {}
                    },
                }
            }
            true
        }

        // Selection logic
        // TODO: Change vertices to lines
        WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Left,
            ..
        } if state.mode == Mode::Normal => {
            if let Some(position) = state.cursor_position {
                let mut update: bool = false;
                println!("{:?}", state.lines);
                for line in &mut state.lines {
                    let a = line.vertices[0].position;
                    let b = line.vertices[1].position;

                    // let product =
                        // (b[0] - a[0]) * (position[1] - a[1]) - (b[1] - a[1]) * (position[0] - a[0]);
                    let d = point_segment_distance(position[0], position[1], a[0], a[1], b[0], b[1]);

                    if d < 5.0 {
                        line.vertices[0].color = [150.0 / 255.0, 20.0 / 255.0, 10.0 / 255.0];
                        line.vertices[1].color = [150.0 / 255.0, 20.0 / 255.0, 10.0 / 255.0];
                        update = true;
                    } else {
                        line.vertices[0].color = [1.0, 1.0, 1.0];
                        line.vertices[1].color = [1.0, 1.0, 1.0];
                    }
                }

                if update {
                    // make_lines_white(state);
                    state.update_vertex_buffer();
                }
            }
            true
        }

        // Scrolling implementation
        WindowEvent::MouseWheel { delta, .. } => {
            let zoom_speed = 0.1;

            match delta {
                MouseScrollDelta::LineDelta(_, y) => {
                    let factor = 1.0 + zoom_speed * y.signum();
                    // state.camera.zoom(factor);
                    state.camera.zoom_at_cursor(
                        factor,
                        state.cursor_position.unwrap()[0],
                        state.cursor_position.unwrap()[1],
                    );
                }
                MouseScrollDelta::PixelDelta(pos) => {
                    state.camera.zoom(1.0 + zoom_speed * pos.y.signum() as f32);
                    println!("zoom two");
                }
            }

            let uniform = state
                .camera
                .to_uniform(state.config.width as f32, state.config.height as f32);

            state
                .queue
                .write_buffer(&state.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));

            true
        }
        _ => false,
    }
}


// helper functions

fn point_segment_distance(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let abx: f32 = bx - ax;
    let aby: f32 = by - ay;
    
    // handle zero length segment
    let ab_len_squared = abx * abx + aby * aby;
    if ab_len_squared == 0.0 {
        let dx: f32 = px - ax;
        let dy: f32 = py - ay;
        return (dx * dx + dy * dy).sqrt();
    }
    
    let t = ((px - ax) * abx + (py - ay) * aby) / ab_len_squared;

    let t = t.clamp(0.0, 1.0);

    let cx: f32 = ax + t * abx;
    let cy: f32 = ay + t * aby;

    let dx: f32 = px - cx;
    let dy: f32 = py - cy;
    (dx * dx + dy * dy).sqrt()
}