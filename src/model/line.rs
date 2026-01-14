use crate::graphics::gui_elements::ColorScheme;
use crate::graphics::vertex::Vertex;
use crate::{DrawLineMode, DrawingState, Mode, State};
use egui_wgpu::wgpu;

// app line struct
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    // pub id: u64,
    pub vertices: [Vertex; 2],
    pub thickness: f32,
    pub selected: bool,
    pub del: bool,
    pub is_drawing: bool,
}

// GPU Instance struct
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LineInstance {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub color: [f32; 3],
    pub thickness: f32,
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
        let length = (dx * dx + dy * dy).sqrt();
        let scale = desired_len / length;
        self.vertices[1] = Vertex {
            position: [start_pos[0] + dx * scale, start_pos[1] + dy * scale, 0.0],
            color: [1.0, 1.0, 1.0],
        };
        self.is_drawing = false;
    }

    pub fn get_len(&self) -> f32 {
        let dx = self.vertices[0].position[0] - self.vertices[1].position[0];
        let dy = self.vertices[0].position[1] - self.vertices[1].position[1];
        let sum = (dx * dx + dy * dy).sqrt();
        let rounded = (sum * 1000.0).round() / 1000.0;
        rounded
    }
}

impl LineInstance {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LineInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // start position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // end position
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // color
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // thickness
                wgpu::VertexAttribute {
                    offset: 36,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
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

// flatten lines into a vec of instances
pub fn flatten_lines_to_instances(
    lines: &mut Vec<Line>,
    color_scheme: ColorScheme,
) -> Vec<LineInstance> {
    lines
        .iter()
        .map(|line| LineInstance {
            start: line.vertices[0].position,
            end: line.vertices[1].position,
            color: if line.selected {
                [1.0, 0.0, 0.0]
            } else if color_scheme == ColorScheme::Light {
                [0.0, 0.0, 0.0]
            } else {
                [1.0, 1.0, 1.0]
            },
            thickness: line.thickness,
        })
        .collect()
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
            thickness: 2.0,
            selected: false,
            del: false,
            is_drawing: is_drawing_flag,
        });

        if is_drawing_flag {
            let index = self.lines.len() - 1;
            self.active_line_index = Some(index);
        }
        // self.active_line_id = Some(id);

        self.update_instance_buffer();
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

        self.update_instance_buffer();
    }

    // needs to be updated to fit for active lines index
    fn cancel_drawing_line(&mut self) {
        self.lines.pop();
        self.drawing_state = DrawingState::Idle;
        self.mode = Mode::Normal;
        self.update_instance_buffer();
    }

    fn unselect_lines(&mut self) {
        for line in &mut self.lines {
            if line.selected {
                line.selected = false;
            }
        }

        self.update_instance_buffer();
    }
}
