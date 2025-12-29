use crate::graphics::vertex::Vertex;
use crate::graphics::gui_elements::ColorScheme;
use crate::{DrawLineMode, DrawingState, Mode, State};

#[derive(Debug, Clone, Copy)]
pub struct Line {
    // pub id: u64,
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

    pub fn finish_line_with_length(&mut self, start_pos: [f32; 2], desired_len: f32) {
        let end_pos = self.vertices[1].position;
        let dx = end_pos[0] - start_pos[0];
        let dy = end_pos[1] - start_pos[1];
        let length = (dx*dx + dy*dy).sqrt();
        let scale = desired_len / length;
        self.vertices[1] = Vertex {
            position: [start_pos[0] + dx*scale, start_pos[1] + dy*scale, 0.0],
            color: [1.0, 1.0, 1.0],
        };
        self.is_drawing = false;
    }
}

// flatten lines vector into a flat vector of vertices
pub fn flatten_lines(lines: &mut Vec<Line>, color_scheme: ColorScheme) -> Vec<Vertex> {
    let mut flat = Vec::new();

    for line in lines.iter_mut() {
        if line.selected {
            line.vertices[0].color = [1.0, 0.0, 0.0]; // bright red
            line.vertices[1].color = [1.0, 0.0, 0.0]; // bright red
        } else if color_scheme == ColorScheme::Light { 
            line.vertices[0].color = [0.0, 0.0, 0.0]; // black
            line.vertices[1].color = [0.0, 0.0, 0.0]; // black
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
        // let id = self.next_line_id;
        // self.next_line_id += 1;

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
            // id,
            selected: false,
            del: false,
            is_drawing: is_drawing_flag,
        });

        if is_drawing_flag {
            let index = self.lines.len() - 1;
            self.active_line_index = Some(index);
        }
        // self.active_line_id = Some(id);

        self.update_vertex_buffer();
    }

    fn update_line(&mut self, position: [f32; 2], is_drawing_flag: bool) {
        let world_x = position[0];
        let world_y = position[1];

        if self.mode == Mode::DrawLine(DrawLineMode::Normal) {
            if let Some(i) = self.active_line_index { 
                // println!("{:?}", self.active_line_index);
                // println!("{:?}", self.active_line_id);
                let last_line = &mut self.lines[i as usize];
                // println!("{:?}", last_line);

                last_line.vertices[1] = Vertex {
                    position: [world_x, world_y, 0.0],
                    color: [1.0, 1.0, 1.0],
                };
                last_line.is_drawing = is_drawing_flag;

                if !is_drawing_flag {
                    // self.active_line_id = None;
                    self.active_line_index = None;
                }
            }
        } else if self.mode == Mode::DrawLine(DrawLineMode::Ortho) {
            if let Some(i) = self.active_line_index { 
                let last_line = &mut self.lines[i as usize];
                let prev_vertice = last_line.vertices[0];

                if (prev_vertice.position[0] - world_x).abs()
                    > (prev_vertice.position[1] - world_y).abs()
                {
                    last_line.vertices[1] = Vertex {
                        position: [world_x, prev_vertice.position[1], 0.0],
                        color: [1.0, 1.0, 1.0],
                    };
                    last_line.is_drawing = is_drawing_flag;
                } else {
                    last_line.vertices[1] = Vertex {
                        position: [prev_vertice.position[0], world_y, 0.0],
                        color: [1.0, 1.0, 1.0],
                    };
                    last_line.is_drawing = is_drawing_flag;
                }

                if !is_drawing_flag {
                    // self.active_line_id = None;
                    self.active_line_index = None;
                }
            }
        }

        // println!("{:?}", self.lines);
        self.update_vertex_buffer();
    }

    // needs to be updated to fit for active lines index
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
