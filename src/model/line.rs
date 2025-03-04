use crate::graphics::vertex::Vertex;
use crate::{DrawingState, Mode, State};

pub trait LineOps {
    fn add_line(&mut self, start: [f32; 2], end: [f32; 2]);
    fn update_line(&mut self, position: [f32; 2]);
    fn cancel_drawing_line(&mut self);
}

// add offsets
impl<'a> LineOps for State<'a> {
    fn add_line(&mut self, start: [f32; 2], end: [f32; 2]) {
        self.vertices.push(Vertex {
            position: [start[0], start[1], 0.0],
            color: [1.0, 1.0, 1.0],
        });
        self.vertices.push(Vertex {
            position: [end[0], end[1], 0.0],
            color: [1.0, 1.0, 1.0],
        });
        self.update_vertex_buffer();
    }

    fn update_line(&mut self, position: [f32; 2]) {
        let world_x = position[0];
        let world_y = position[1];

        self.vertices[(self.num_vertices - 1) as usize] = Vertex {
            position: [world_x, world_y, 0.0],
            color: [1.0, 1.0, 1.0],
        };
        self.update_vertex_buffer();
    }

    fn cancel_drawing_line(&mut self) {
        self.vertices.pop();
        self.vertices.pop();
        self.drawing_state = DrawingState::Idle;
        self.mode = Mode::Normal;
        self.update_vertex_buffer();
    }
}
