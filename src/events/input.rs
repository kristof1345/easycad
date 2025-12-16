use crate::model::circle::CircleOps;
use crate::model::line::LineOps;
use crate::model::line::Line;
use crate::model::circle::Circle;
use crate::graphics::camera::Camera;
use crate::DrawLineMode;
use crate::MoveMode;
use crate::CopyMode;
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
                KeyCode::KeyK => {
                    if state.mode == Mode::Normal {
                        state.mode = Mode::Copy(CopyMode::Selection);
                    }
                }
                KeyCode::KeyM => {
                    if state.mode == Mode::Normal {
                        state.mode = Mode::Move(MoveMode::Selection);
                    }
                }
                KeyCode::KeyS => {
                    if state.modifiers.control_key() {
                        let saved = state.save_to_dxf();

                        match saved {
                            Ok(_) => println!("file saved"),
                            Err(error) => eprintln!("i/o error while saving file: {}", error),
                        };
                    } else {
                        // test
                        if state.mode == Mode::Normal {
                            state.mode = Mode::Selection;
                        }
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

                    if matches!(state.mode, Mode::Selection | Mode::Move(MoveMode::Selection)) {
                        // if lines are selected
                        // unselec_lines
                        if state.lines.iter().any(|line| line.selected) {
                            state.unselect_lines();
                        }

                        // if circles are selected
                        // unselect circles
                        if state.circles.iter().any(|circle| circle.selected) {
                            state.unselect_circles();
                        }
                    }

                    state.mode = Mode::Normal;
                    state.drawing_state = DrawingState::Idle;
                }
                KeyCode::Enter => {
                    if matches!(state.mode, Mode::Move(MoveMode::Selection)) {
                        state.mode = Mode::Move(MoveMode::SelectPoint);
                    }
                    if matches!(state.mode, Mode::Copy(CopyMode::Selection)) {
                        state.mode = Mode::Copy(CopyMode::SelectPoint);
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
            let snap_treshold: f32 = 5.0 / state.camera.zoom;
            let screen = [position.x as f32, position.y as f32];
            state.last_screen_position_for_pan = Some(screen);

            let world = screen_to_world(screen[0], screen[1], state.size, &state.camera);

            state.cursor_position = Some(world);

            let cen_x = screen[0] - state.size.width as f32 / 2.0;
            let cen_y = state.size.height as f32 / 2.0 - screen[1];
            state.last_position_for_pan = Some([
                cen_x / state.camera.zoom,
                cen_y / state.camera.zoom,
            ]);

            state.snap = None;

            for line in &mut state.lines {
                if !line.is_drawing {
                    for vertex in &line.vertices {
                        let x = vertex.position[0];
                        let y = vertex.position[1];
    
                        let diffx = x - world[0];
                        let diffy = y - world[1];
    
                        if diffx.abs() < snap_treshold && diffy.abs() < snap_treshold {
                            state.snap = Some(*vertex);
                            break;
                        }
                    }
                }
            }

            match state.snap {
                Some(_vertex) => {
                    // there's a bug if you zoom out while there's snap turned on, should dissapear once i figure how to reset snap
                    state.move_indicators_to_cursor([world[0], world[1]]);
                    state.update_axis_vertex_buffer();
                }
                None => {
                    for indicator in &mut state.indicators {
                        indicator.vertices[0].position[0] = 0.0;
                        indicator.vertices[0].position[1] = 0.0;

                        indicator.vertices[1].position[0] = 0.0;
                        indicator.vertices[1].position[1] = 0.0;
                    }
                }
            }

            if let DrawingState::WaitingForSecondPoint(_start_pos) = state.drawing_state {
                state.update_line([world[0], world[1]], true);
            }
            if let DrawingState::WaitingForRadius(_start_pos) = state.drawing_state {
                state.update_circle([world[0], world[1]]);
            }
            if let Mode::Move(MoveMode::Move(starting_position)) | Mode::Copy(CopyMode::Copy(starting_position)) = state.mode {
                let diff1 = starting_position[0] - world[0];
                let diff2 = starting_position[1] - world[1];

                for line in &mut state.lines {
                    if line.selected {
                        line.move_line(diff1, diff2);
                    }
                }
                for circle in &mut state.circles {
                    if circle.selected {
                        circle.move_circle(diff1, diff2);
                    }
                }

                state.update_vertex_buffer();
                state.update_circle_vertex_buffer();
                state.mode = Mode::Move(MoveMode::Move([world[0], world[1]]));
            }

            true
        }

        // handles drawing state
        WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Left,
            ..
        } if matches!(state.mode, Mode::DrawCircle | Mode::DrawLine(_) | Mode::Move(MoveMode::SelectPoint) | Mode::Move(MoveMode::Move(_)) | Mode::Copy(CopyMode::SelectPoint) | Mode::Copy(CopyMode::Copy(_))) => {
            if let Some(position) = state.cursor_position {
                match state.drawing_state {
                    DrawingState::Idle => match state.mode {
                        Mode::DrawLine(DrawLineMode::Normal)
                        | Mode::DrawLine(DrawLineMode::Ortho) => {
                            state.drawing_state = DrawingState::WaitingForSecondPoint(position);
                            state.add_line(position, position, true);
                        }
                        Mode::DrawCircle => {
                            state.drawing_state = DrawingState::WaitingForRadius(position);
                            state.add_circle(position, 0.0, [1.0, 1.0, 1.0], false, false);
                        }
                        // move lines from this selection point
                        Mode::Move(MoveMode::SelectPoint) | Mode::Copy(CopyMode::SelectPoint) => {
                            let mut new_lines = Vec::new();
                            let mut new_circles = Vec::new();

                            if matches!(state.mode, Mode::Move(_)) {
                                state.mode = Mode::Move(MoveMode::Move(position));
                            } else {
                                state.mode = Mode::Copy(CopyMode::Copy(position));
                            }

                            for line in &mut state.lines {
                                if line.selected {
                                    let new_line = line.clone();
                                    line.selected = false;
                                    line.del = matches!(state.mode, Mode::Move(_));

                                    new_lines.push(new_line);
                                }
                            }
                            for circle in &mut state.circles {
                                if circle.selected {
                                    let new_circle = circle.clone();
                                    circle.selected = false;
                                    circle.del = matches!(state.mode, Mode::Move(_));

                                    new_circles.push(new_circle);
                                }
                            }


                            for new_line in new_lines {
                                state.lines.push(new_line);
                            }
                            for new_circle in new_circles {
                                state.add_circle([new_circle.center.position[0], new_circle.center.position[1]], new_circle.radius, new_circle.center.color, new_circle.selected, new_circle.del);
                            }
                        }
                        // second click: move the selected objects "HERE"
                        Mode::Move(MoveMode::Move(starting_position)) | Mode::Copy(CopyMode::Copy(starting_position)) => {
                            let diff1 = starting_position[0] - position[0];
                            let diff2 = starting_position[1] - position[1];

                            for line in &mut state.lines {
                                if line.selected {
                                    line.move_line(diff1, diff2);

                                    line.selected = false;
                                }
                            }
                            for circle in &mut state.circles {
                                if circle.selected {
                                    circle.move_circle(diff1, diff2);

                                    circle.selected = false;
                                }
                            }

                            // todo
                            state.lines.retain(|line: &Line| line.del != true);
                            state.circles.retain(|circle: &Circle| circle.del != true);

                            state.update_vertex_buffer();
                            state.update_circle_vertex_buffer();
                            state.mode = Mode::Normal;
                        }
                        _ => {}
                    },
                    DrawingState::WaitingForSecondPoint(_start_pos) => match state.mode {
                        Mode::DrawLine(DrawLineMode::Normal)
                        | Mode::DrawLine(DrawLineMode::Ortho) => {
                            state.update_line(position, false);
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
        } if matches!(state.mode, Mode::Normal | Mode::Selection | Mode::Move(MoveMode::Selection) | Mode::Copy(CopyMode::Selection)) => {
            if let Some(position) = state.cursor_position {
                let mut update: bool = false;

                let treshold = 5.0 / state.camera.zoom;
  
                for line in &mut state.lines {
                    let a = line.vertices[0].position;
                    let b = line.vertices[1].position;

                    let d = point_segment_distance(position[0], position[1], a[0], a[1], b[0], b[1]);

                    if d < treshold && !line.selected {
                        if !matches!(state.mode, Mode::Move(_) | Mode::Copy(_)) {
                            state.mode = Mode::Selection;
                        }
                        line.selected = true;
                        update = true;
                    } 
                }

                for circle in &mut state.circles {
                    let [cx, cy, ..] = circle.center.position;
                    let rad = circle.radius;

                    let d = circle_hit(position[0], position[1], cx, cy, rad);

                    if d < treshold && !circle.selected {
                        if !matches!(state.mode, Mode::Move(_) | Mode::Copy(_)) {
                            state.mode = Mode::Selection;
                        }
                        circle.selected = true;
                        update = true;
                    } 
                }

                if update {
                    state.update_vertex_buffer();
                    state.update_circle_vertex_buffer();
                }
            }
            true
        }

        // Scrolling implementation
        WindowEvent::MouseWheel { delta, .. } => {
            let zoom_speed = 0.1;

            let factor = match delta {
                MouseScrollDelta::LineDelta(_, y) => {
                    1.0 + zoom_speed * y.signum()
                }
                MouseScrollDelta::PixelDelta(pos) => {
                    1.0 + zoom_speed * pos.y.signum() as f32
                }
            };

            if let Some([sx, sy]) = state.last_screen_position_for_pan {
                let before = screen_to_world(sx, sy, state.size, &state.camera);

                state.camera.zoom(factor);

                let after = screen_to_world(sx, sy, state.size, &state.camera);

                state.camera.pan(before[0] - after[0], before[1] - after[1]);

                state.cursor_position = Some(before);
                state.last_position_for_pan = Some([
                    (sx - state.size.width as f32 / 2.0) / state.camera.zoom,
                    (state.size.height as f32 / 2.0 - sy) / state.camera.zoom,
                ]);
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

fn circle_hit(px: f32, py: f32, cx: f32, cy: f32, r: f32) -> f32 {
    let dx = px - cx;
    let dy = py - cy;
    let dist = (dx*dx + dy*dy).sqrt();

    (dist - r).abs()
}

fn screen_to_world(
    screen_x: f32,
    screen_y: f32,
    size: winit::dpi::PhysicalSize<u32>,
    camera: &Camera,
) -> [f32; 2] {
    let cen_x = screen_x - size.width as f32 / 2.0;
    let cen_y = size.height as f32 / 2.0 - screen_y;

    [
        cen_x / camera.zoom + camera.x_offset,
        cen_y / camera.zoom + camera.y_offset,
    ]
}
