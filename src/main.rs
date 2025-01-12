use dxf::entities::EntityType;
use dxf::entities::Line as DxfLine;
use dxf::Drawing;
use dxf::Point;
use ggez::{
    event,
    graphics::{self, Color, DrawParam, Mesh},
    input::keyboard::{KeyCode, KeyInput, KeyMods},
    input::mouse::MouseButton,
    Context, GameResult,
};

struct Line {
    start: [f32; 2],
    end: [f32; 2],
}

struct MainState {
    lines: Vec<Line>,
    current_line: Option<Line>,
    drawing: bool,
}

impl MainState {
    fn new() -> GameResult<MainState> {
        Ok(MainState {
            lines: Vec::new(),
            current_line: None,
            drawing: false,
        })
    }

    fn save_to_dxf(&self) {
        let mut drawing = Drawing::new();

        // Convert our lines to DXF lines
        for line in &self.lines {
            let dxf_line = DxfLine::new(
                Point::new(f64::from(line.start[0]), f64::from(line.start[1]), 0.0),
                Point::new(f64::from(line.end[0]), f64::from(line.end[1]), 0.0),
            );
            drawing.add_entity(dxf::entities::Entity::new(EntityType::Line(dxf_line)));
        }

        // Show save file dialog
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("DXF", &["dxf"])
            .save_file()
        {
            // Save the drawing
            match drawing.save_file(path) {
                Ok(_) => println!("File saved successfully!"),
                Err(e) => println!("Error saving file: {}", e),
            }
        }
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);

        // Draw existing lines
        for line in &self.lines {
            let mesh = Mesh::new_line(ctx, &[line.start, line.end], 2.0, Color::BLACK)?;
            canvas.draw(&mesh, DrawParam::default());
        }

        // Draw current line if drawing
        if let Some(line) = &self.current_line {
            let mesh = Mesh::new_line(ctx, &[line.start, line.end], 2.0, Color::BLACK)?;
            canvas.draw(&mesh, DrawParam::default());
        }

        // Draw crosshair cursor
        let mouse_pos = ctx.mouse.position();
        let cursor_size = 10.0;

        // Horizontal line
        let horizontal_cursor = Mesh::new_line(
            ctx,
            &[
                [mouse_pos.x - cursor_size, mouse_pos.y],
                [mouse_pos.x + cursor_size, mouse_pos.y],
            ],
            1.0,
            Color::RED,
        )?;
        canvas.draw(&horizontal_cursor, DrawParam::default());

        // Vertical line
        let vertical_cursor = Mesh::new_line(
            ctx,
            &[
                [mouse_pos.x, mouse_pos.y - cursor_size],
                [mouse_pos.x, mouse_pos.y + cursor_size],
            ],
            1.0,
            Color::RED,
        )?;
        canvas.draw(&vertical_cursor, DrawParam::default());

        canvas.finish(ctx)?;
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        if button == MouseButton::Left {
            self.drawing = true;
            self.current_line = Some(Line {
                start: [x, y],
                end: [x, y],
            });
        }
        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        if button == MouseButton::Left && self.drawing {
            if let Some(mut line) = self.current_line.take() {
                line.end = [x, y];
                self.lines.push(line);
            }
            self.drawing = false;
        }
        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        x: f32,
        y: f32,
        _dx: f32,
        _dy: f32,
    ) -> GameResult {
        if self.drawing {
            if let Some(line) = &mut self.current_line {
                line.end = [x, y];
            }
        }
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        let ctrl_pressed = input.mods == KeyMods::CTRL;
        if input.keycode == Some(KeyCode::S) && ctrl_pressed {
            self.save_to_dxf();
        }
        Ok(())
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("simple_cad", "you")
        .window_setup(ggez::conf::WindowSetup::default().title("Simple CAD"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(800.0, 600.0));

    let (ctx, event_loop) = cb.build()?;
    let state = MainState::new()?;
    event::run(ctx, event_loop, state)
}
