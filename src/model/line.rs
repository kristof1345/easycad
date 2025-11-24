use crate::graphics::vertex::Vertex;
use crate::{DrawLineMode, DrawingState, Mode, State};

#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub vertices: [Vertex; 2],
    pub selected: bool,
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
    fn add_line(&mut self, start: [f32; 2], end: [f32; 2]);
    fn update_line(&mut self, position: [f32; 2]);
    fn cancel_drawing_line(&mut self);
}

// add offsets
impl<'a> LineOps for State<'a> {
    fn add_line(&mut self, start: [f32; 2], end: [f32; 2]) {
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
        });

        self.update_vertex_buffer();
    }

    fn update_line(&mut self, position: [f32; 2]) {
        let world_x = position[0];
        let world_y = position[1];

        let length = self.lines.len();

        let prev_vertice = self.lines[(length - 1) as usize].vertices[0];

        if self.mode == Mode::DrawLine(DrawLineMode::Normal) {
            self.lines[(length - 1) as usize].vertices[1] = Vertex {
                position: [world_x, world_y, 0.0],
                color: [1.0, 1.0, 1.0],
            };
            // this section is buggy, I need to subtract the prev_vertice from world_x and world_y
        } else if self.mode == Mode::DrawLine(DrawLineMode::Ortho) {
            if (prev_vertice.position[0] - world_x).abs()
                > (prev_vertice.position[1] - world_y).abs()
            {
                self.lines[(length - 1) as usize].vertices[1] = Vertex {
                    position: [world_x, prev_vertice.position[1], 0.0],
                    color: [1.0, 1.0, 1.0],
                };
            } else {
                self.lines[(length - 1) as usize].vertices[1] = Vertex {
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
}
