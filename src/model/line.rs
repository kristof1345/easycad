use crate::graphics::vertex::Vertex;
use crate::{DrawLineMode, DrawingState, Mode, State};

#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub vertices: [Vertex; 2],
    pub selected: bool,
    pub del: bool,
    pub is_drawing: bool,
}

impl Line {
    pub fn move_line(&mut self, dx: f32, dy: f32) {
        for v in &mut self.vertices {
            v.position[0] -= dx;
            v.position[1] -= dy;
        }
    }
}

// flatten lines vector into a flat vector of vertices
pub fn flatten_lines(lines: &mut Vec<Line>) -> Vec<Vertex> {
    let mut flat = Vec::new();

    for line in lines.iter_mut() {
        if line.selected {
            line.vertices[0].color = [1.0, 0.0, 0.0]; // bright red
            line.vertices[1].color = [1.0, 0.0, 0.0]; // bright red
        } else {
            line.vertices[0].color = [1.0, 1.0, 1.0]; // white
            line.vertices[1].color = [1.0, 1.0, 1.0]; // white
        }
        flat.push(line.vertices[0]);
        flat.push(line.vertices[1]);
    }

    flat
}

pub trait LineOps {
    fn add_line(&mut self, start: [f32; 2], end: [f32; 2], is_drawing_flag: bool);
    fn update_line(&mut self, position: [f32; 2], is_drawing_flag: bool);
    fn cancel_drawing_line(&mut self);
    fn unselect_lines(&mut self);
}


// add offsets
impl<'a> LineOps for State<'a> {
    fn add_line(&mut self, start: [f32; 2], end: [f32; 2], is_drawing_flag: bool) {
        self.lines.push(Line {
            vertices: [
                Vertex {
                    position: [start[0], start[1], 0.0],
                    color: [1.0, 1.0, 1.0],
                },
                Vertex {
                    position: [end[0], end[1], 0.0],
                    color: [1.0, 1.0, 1.0],
                },
            ],
            selected: false,
            del: false,
            is_drawing: is_drawing_flag,
        });

        self.update_vertex_buffer();
    }

    fn update_line(&mut self, position: [f32; 2], is_drawing_flag: bool) {
        let world_x = position[0];
        let world_y = position[1];

        // let length = self.lines.len();

        let last_line = self.lines.last_mut().unwrap();
        let prev_vertice = last_line.vertices[0];


        if self.mode == Mode::DrawLine(DrawLineMode::Normal) {
            last_line.vertices[1] = Vertex {
                position: [world_x, world_y, 0.0],
                color: [1.0, 1.0, 1.0],
            };
            last_line.is_drawing = is_drawing_flag;
            // this section is buggy, I need to subtract the prev_vertice from world_x and world_y
            // outdated - update to sit with last_line
        } else if self.mode == Mode::DrawLine(DrawLineMode::Ortho) {
            if (prev_vertice.position[0] - world_x).abs()
                > (prev_vertice.position[1] - world_y).abs()
            {
                last_line.vertices[1] = Vertex {
                    position: [world_x, prev_vertice.position[1], 0.0],
                    color: [1.0, 1.0, 1.0],
                };
            } else {
                last_line.vertices[1] = Vertex {
                    position: [prev_vertice.position[0], world_y, 0.0],
                    color: [1.0, 1.0, 1.0],
                };
            }
        }

        self.update_vertex_buffer();
    }

    fn cancel_drawing_line(&mut self) {
        self.lines.pop();
        self.drawing_state = DrawingState::Idle;
        self.mode = Mode::Normal;
        self.update_vertex_buffer();
    }

    fn unselect_lines(&mut self) {
        for line in &mut self.lines {
            if line.selected {
                line.selected = false;
            }
        }

        self.update_vertex_buffer();
    }
}
