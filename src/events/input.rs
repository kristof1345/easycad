use crate::graphics::camera::Camera;
use crate::graphics::gui_elements::{Text, UiMode};
use crate::model::circle::flatten_circles_for_snap;
use crate::model::circle::Circle;
use crate::model::circle::CircleOps;
use crate::model::line::Line;
use crate::model::line::LineOps;
use crate::DrawLineMode;
use crate::DrawingState;
use crate::FuncState;
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
                    // text: usr_string,
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
                        state.mode = Mode::Copy(FuncState::Selection);
                    }
                }
                KeyCode::KeyM => {
                    if state.mode == Mode::Normal {
                        state.mode = Mode::Move(FuncState::Selection);
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
                        if state.mode == Mode::Normal {
                            state.mode = Mode::Selection;
                        }
                    }
                }
                KeyCode::KeyO => {
                    if state.modifiers.control_key() {
                        let mut path: String = String::new();

                        if let Some(file_path) = rfd::FileDialog::new()
                            .add_filter(".dxf, .cad", &["dxf", "cad"])
                            .pick_file()
                        {
                            path = file_path.display().to_string();
                            println!("{path}");
                        }

                        let len = path.len();
                        let extension: &str = &path[len - 3..len];
                        println!("{extension}");

                        let loaded: Result<(), Box<dyn std::error::Error>> = if extension == "dxf" {
                            state.load_from_dxf(path)
                        } else if extension == "cad" {
                            state.load_from_cad(path)
                        } else {
                            Err("Unsupported file extension".into())
                        };

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
                KeyCode::KeyA => {
                    state.mode = Mode::Measure(None);
                }
                KeyCode::Delete => {
                    if state.mode == Mode::Normal {
                        state.mode = Mode::Delete;
                    } else if state.mode == Mode::Selection {
                        state.lines.retain(|line| line.selected != true);
                        state.circles.retain(|circle| circle.selected != true);

                        state.update_instance_buffer();
                        state.update_circle_instance_buffer();
                    }
                }
                KeyCode::KeyT => {
                    if state.mode == Mode::Normal {
                        state.mode = Mode::CreateText;
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

                    if matches!(
                        state.mode,
                        Mode::Selection | Mode::Move(FuncState::Selection)
                    ) {
                        if state.lines.iter().any(|line| line.selected) {
                            state.unselect_lines();
                        }
                        if state.circles.iter().any(|circle| circle.selected) {
                            state.unselect_circles();
                        }
                    }

                    state.mode = Mode::Normal;
                    state.snap = None;
                    state.drawing_state = DrawingState::Idle;

                    for indicator in &mut state.indicators {
                        indicator.vertices[0].position = [0.0, 0.0, 0.0];
                        indicator.vertices[1].position = [0.0, 0.0, 0.0];
                    }
                }
                KeyCode::Enter => {
                    if matches!(state.mode, Mode::Move(FuncState::Selection)) {
                        state.mode = Mode::Move(FuncState::SelectPoint);
                    }
                    if matches!(state.mode, Mode::Copy(FuncState::Selection)) {
                        state.mode = Mode::Copy(FuncState::SelectPoint);
                    }
                }
                KeyCode::Digit0 => {
                    if matches!(
                        state.drawing_state,
                        DrawingState::WaitingForSecondPoint(_) | DrawingState::WaitingForRadius(_)
                    ) {
                        state.ui.push_digit('0')
                    }
                }
                KeyCode::Digit1 => {
                    if matches!(
                        state.drawing_state,
                        DrawingState::WaitingForSecondPoint(_) | DrawingState::WaitingForRadius(_)
                    ) {
                        state.ui.push_digit('1')
                    }
                }
                KeyCode::Digit2 => {
                    if matches!(
                        state.drawing_state,
                        DrawingState::WaitingForSecondPoint(_) | DrawingState::WaitingForRadius(_)
                    ) {
                        state.ui.push_digit('2')
                    }
                }
                KeyCode::Digit3 => {
                    if matches!(
                        state.drawing_state,
                        DrawingState::WaitingForSecondPoint(_) | DrawingState::WaitingForRadius(_)
                    ) {
                        state.ui.push_digit('3')
                    }
                }
                KeyCode::Digit4 => {
                    if matches!(
                        state.drawing_state,
                        DrawingState::WaitingForSecondPoint(_) | DrawingState::WaitingForRadius(_)
                    ) {
                        state.ui.push_digit('4')
                    }
                }
                KeyCode::Digit5 => {
                    if matches!(
                        state.drawing_state,
                        DrawingState::WaitingForSecondPoint(_) | DrawingState::WaitingForRadius(_)
                    ) {
                        state.ui.push_digit('5')
                    }
                }
                KeyCode::Digit6 => {
                    if matches!(
                        state.drawing_state,
                        DrawingState::WaitingForSecondPoint(_) | DrawingState::WaitingForRadius(_)
                    ) {
                        state.ui.push_digit('6')
                    }
                }
                KeyCode::Digit7 => {
                    if matches!(
                        state.drawing_state,
                        DrawingState::WaitingForSecondPoint(_) | DrawingState::WaitingForRadius(_)
                    ) {
                        state.ui.push_digit('7')
                    }
                }
                KeyCode::Digit8 => {
                    if matches!(
                        state.drawing_state,
                        DrawingState::WaitingForSecondPoint(_) | DrawingState::WaitingForRadius(_)
                    ) {
                        state.ui.push_digit('8')
                    }
                }
                KeyCode::Digit9 => {
                    if matches!(
                        state.drawing_state,
                        DrawingState::WaitingForSecondPoint(_) | DrawingState::WaitingForRadius(_)
                    ) {
                        state.ui.push_digit('9')
                    }
                }
                KeyCode::Period => {
                    if matches!(
                        state.drawing_state,
                        DrawingState::WaitingForSecondPoint(_) | DrawingState::WaitingForRadius(_)
                    ) {
                        state.ui.push_digit('.')
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
            state.ui.cursor_position = Some(world);

            let cen_x = screen[0] - state.size.width as f32 / 2.0;
            let cen_y = state.size.height as f32 / 2.0 - screen[1];
            state.last_position_for_pan =
                Some([cen_x / state.camera.zoom, cen_y / state.camera.zoom]);

            if matches!(
                state.mode,
                Mode::DrawLine(_)
                    | Mode::DrawCircle
                    | Mode::Copy(_)
                    | Mode::Move(_)
                    | Mode::Measure(_)
            ) {
                state.snap = None;

                for line in &mut state.lines {
                    if !line.is_drawing {
                        for vertex in line.vertices {
                            let x = vertex.position[0];
                            let y = vertex.position[1];

                            let diffx = x - world[0];
                            let diffy = y - world[1];

                            if diffx.abs() < snap_treshold && diffy.abs() < snap_treshold {
                                state.snap = Some([x, y]);
                                break;
                            }
                        }
                    }
                }

                let circle_snap_vertexes = flatten_circles_for_snap(&mut state.circles);
                for vertex in circle_snap_vertexes {
                    let x = vertex.position[0];
                    let y = vertex.position[1];

                    let diffx = x - world[0];
                    let diffy = y - world[1];

                    if diffx.abs() < snap_treshold && diffy.abs() < snap_treshold {
                        state.snap = Some([x, y]);
                        break;
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
                            indicator.vertices[0].position = [0.0, 0.0, 0.0];
                            indicator.vertices[1].position = [0.0, 0.0, 0.0];
                        }
                    }
                }
            }

            if let DrawingState::WaitingForSecondPoint(_start_pos) = state.drawing_state {
                state.update_line([world[0], world[1]], true);
            }
            if let DrawingState::WaitingForRadius(_start_pos) = state.drawing_state {
                state.update_circle([world[0], world[1]], true);
            }
            if let Mode::Move(FuncState::Move(starting_position))
            | Mode::Copy(FuncState::Copy(starting_position)) = state.mode
            {
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

                state.update_instance_buffer();
                state.update_circle_instance_buffer();

                state.mode = Mode::Move(FuncState::Move([world[0], world[1]]));
            }

            true
        }

        // handles drawing state
        WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Left,
            ..
        } if matches!(
            state.mode,
            Mode::DrawCircle
                | Mode::DrawLine(_)
                | Mode::Move(FuncState::SelectPoint)
                | Mode::Move(FuncState::Move(_))
                | Mode::Copy(FuncState::SelectPoint)
                | Mode::Copy(FuncState::Copy(_))
                | Mode::Measure(_)
                | Mode::CreateText
        ) =>
        {
            if let Some(position) = state.cursor_position {
                match state.drawing_state {
                    DrawingState::Idle => match state.mode {
                        Mode::DrawLine(DrawLineMode::Normal)
                        | Mode::DrawLine(DrawLineMode::Ortho) => {
                            let mut snap_or_position: [f32; 2] = position;

                            match state.snap {
                                Some(snap_positions) => {
                                    snap_or_position = snap_positions;
                                }
                                None => {}
                            }

                            state.drawing_state =
                                DrawingState::WaitingForSecondPoint(snap_or_position);
                            state.add_line(snap_or_position, snap_or_position, true);
                        }
                        Mode::DrawCircle => {
                            let mut snap_or_position: [f32; 2] = position;

                            match state.snap {
                                Some(snap_positions) => {
                                    snap_or_position = snap_positions;
                                }
                                None => {}
                            }

                            state.drawing_state = DrawingState::WaitingForRadius(snap_or_position);
                            state.add_circle(
                                snap_or_position,
                                0.0,
                                [1.0, 1.0, 1.0],
                                false,
                                false,
                                true,
                            );
                        }
                        Mode::Measure(first_pos) => {
                            let snap_or_pos = state.snap.unwrap_or_else(|| position);
                            match first_pos {
                                Some(pos) => {
                                    // println!("{:?}, {:?}", pos, position);
                                    let dx = pos[0] - snap_or_pos[0];
                                    let dy = pos[1] - snap_or_pos[1];
                                    let sum = (dx * dx + dy * dy).sqrt();
                                    let rounded = (sum * 1000.0).round() / 1000.0;
                                    // println!("{}", rounded);
                                    state.ui.add_notification(&format!("{}", rounded));
                                    state.mode = Mode::Measure(None);
                                }
                                None => {
                                    state.mode = Mode::Measure(Some(snap_or_pos));
                                }
                            }
                        }
                        // move lines from this selection point
                        Mode::Move(FuncState::SelectPoint) | Mode::Copy(FuncState::SelectPoint) => {
                            let mut new_lines = Vec::new();
                            let mut new_circles = Vec::new();

                            let pos: [f32; 2];
                            if let Some(snap_pos) = state.snap {
                                pos = snap_pos;
                            } else {
                                pos = position;
                            }

                            if matches!(state.mode, Mode::Move(_)) {
                                state.mode = Mode::Move(FuncState::Move(pos));
                            } else {
                                state.mode = Mode::Copy(FuncState::Copy(pos));
                            }

                            for line in &mut state.lines {
                                if line.selected {
                                    let mut new_line = line.clone();
                                    line.selected = false;
                                    line.del = matches!(state.mode, Mode::Move(_));

                                    new_line.is_drawing = true;
                                    new_lines.push(new_line);
                                }
                            }
                            for circle in &mut state.circles {
                                if circle.selected {
                                    let mut new_circle = circle.clone();
                                    circle.selected = false;
                                    circle.del = matches!(state.mode, Mode::Move(_));

                                    new_circle.is_drawing = true;
                                    new_circles.push(new_circle);
                                }
                            }

                            for new_line in new_lines {
                                state.lines.push(new_line);
                            }
                            for new_circle in new_circles {
                                state.add_circle(
                                    [new_circle.center.position[0], new_circle.center.position[1]],
                                    new_circle.radius,
                                    new_circle.center.color,
                                    new_circle.selected,
                                    new_circle.del,
                                    new_circle.is_drawing,
                                );
                            }
                        }
                        // second click: move the selected objects "HERE"
                        Mode::Move(FuncState::Move(starting_position))
                        | Mode::Copy(FuncState::Copy(starting_position)) => {
                            let diff1: f32;
                            let diff2: f32;

                            if let Some(snap_pos) = state.snap {
                                diff1 = starting_position[0] - snap_pos[0];
                                diff2 = starting_position[1] - snap_pos[1];
                            } else {
                                diff1 = starting_position[0] - position[0];
                                diff2 = starting_position[1] - position[1];
                            }

                            for line in &mut state.lines {
                                if line.selected {
                                    line.move_line(diff1, diff2);

                                    line.selected = false;
                                    line.is_drawing = false;
                                }
                            }
                            for circle in &mut state.circles {
                                if circle.selected {
                                    circle.move_circle(diff1, diff2);

                                    circle.selected = false;
                                    circle.is_drawing = false;
                                }
                            }

                            // todo
                            state.lines.retain(|line: &Line| line.del != true);
                            state.circles.retain(|circle: &Circle| circle.del != true);

                            state.update_instance_buffer();
                            // state.update_circle_vertex_buffer();
                            state.update_circle_instance_buffer();

                            state.mode = Mode::Normal;

                            for indicator in &mut state.indicators {
                                indicator.vertices[0].position = [0.0, 0.0, 0.0];
                                indicator.vertices[1].position = [0.0, 0.0, 0.0];
                            }
                        }
                        Mode::CreateText => {
                            // create a new text object
                            let snap_or_pos = state.snap.unwrap_or_else(|| position);
                            state.ui.texts.push(Text {
                                position: snap_or_pos,
                                contents: egui::WidgetText::from("Text"),
                                rect: None,
                                editing: true,
                                annotative: false,
                            });
                            state.ui.mode = UiMode::TextEdit;
                            state.ui.text_edited.contents = String::from("Text");
                            state.ui.text_edited.annotative = false;
                            state.mode = Mode::Normal;
                        }
                        _ => {}
                    },
                    DrawingState::WaitingForSecondPoint(_start_pos) => match state.mode {
                        Mode::DrawLine(DrawLineMode::Normal)
                        | Mode::DrawLine(DrawLineMode::Ortho) => {
                            let mut snap_or_position: [f32; 2] = position;

                            match state.snap {
                                Some(snap_positions) => {
                                    snap_or_position = snap_positions;
                                }
                                None => {}
                            }

                            // println!("{:?}", state.lines);
                            state.update_line(snap_or_position, false);
                            // println!("{:?}", state.lines);
                            state.drawing_state = DrawingState::Idle;
                        }
                        _ => {}
                    },
                    DrawingState::WaitingForRadius(_start_pos) => match state.mode {
                        Mode::DrawCircle => {
                            let mut snap_or_position: [f32; 2] = position;

                            match state.snap {
                                Some(snap_positions) => {
                                    snap_or_position = snap_positions;
                                }
                                None => {}
                            }

                            state.update_circle(snap_or_position, false);
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
        } if matches!(
            state.mode,
            Mode::Normal
                | Mode::Selection
                | Mode::Move(FuncState::Selection)
                | Mode::Copy(FuncState::Selection)
                | Mode::Delete
        ) =>
        {
            if let Some(position) = state.cursor_position {
                let mut update: bool = false;

                let treshold = 5.0 / state.camera.zoom;

                for line in &mut state.lines {
                    let a = line.vertices[0].position;
                    let b = line.vertices[1].position;

                    let d =
                        point_segment_distance(position[0], position[1], a[0], a[1], b[0], b[1]);

                    if d < treshold && !line.selected {
                        if !matches!(state.mode, Mode::Move(_) | Mode::Copy(_) | Mode::Delete) {
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
                        if !matches!(state.mode, Mode::Move(_) | Mode::Copy(_) | Mode::Delete) {
                            state.mode = Mode::Selection;
                        }
                        circle.selected = true;
                        update = true;
                    }
                }

                if update {
                    if state.mode == Mode::Delete {
                        state.lines.retain(|line| line.selected != true);
                        state.circles.retain(|circle| circle.selected != true);
                    }
                    state.update_instance_buffer();
                    // state.update_circle_vertex_buffer();
                    state.update_circle_instance_buffer();
                }
            }
            true
        }

        // edit text
        WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Right,
            ..
        } if matches!(state.mode, Mode::Normal) => {
            if let Some(position) = state.cursor_position {
                let viewport_rect = state.ui.viewport_rect();
                let pixels_per_point = state.ui.pixels_per_point();

                for text in &mut state.ui.texts {
                    if let Some(rect) = text.rect {
                        let pos2_position = world_to_screen(
                            position[0],
                            position[1],
                            viewport_rect,
                            &state.camera,
                            pixels_per_point,
                        );

                        if rect.contains(pos2_position) {
                            state.ui.mode = UiMode::TextEdit;
                            state.ui.text_edited.contents = text.contents.text().to_string();
                            state.ui.text_edited.annotative = text.annotative;
                            text.editing = true;
                            // make it quit the loop
                            return true;
                        }
                    }
                }
            }
            true
        }

        // Scrolling implementation
        WindowEvent::MouseWheel { delta, .. } => {
            let zoom_speed = 0.1;

            let factor = match delta {
                MouseScrollDelta::LineDelta(_, y) => 1.0 + zoom_speed * y.signum(),
                MouseScrollDelta::PixelDelta(pos) => 1.0 + zoom_speed * pos.y.signum() as f32,
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
    let dist = (dx * dx + dy * dy).sqrt();

    (dist - r).abs()
}

pub fn screen_to_world(
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

pub fn world_to_screen(
    world_x: f32,
    world_y: f32,
    veiwport_rect: egui::Rect,
    camera: &Camera,
    pixels_per_point: f32,
) -> egui::Pos2 {
    let screen_cen = veiwport_rect.center();
    let log_zoom = camera.zoom / pixels_per_point;
    let rel_x = world_x - camera.x_offset;
    let rel_y = world_y - camera.y_offset;

    let screen_x = (rel_x * log_zoom) + screen_cen.x;
    let screen_y = screen_cen.y - (rel_y * log_zoom);

    egui::pos2(screen_x, screen_y)
}
