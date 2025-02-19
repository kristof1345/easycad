use crate::graphics::vertex::Vertex;
use crate::State;

pub trait LineOps {
    fn add_line(&mut self, start: [f32; 2], end: [f32; 2]);
    fn update_line(&mut self, position: [f32; 2]);
}

impl<'a> LineOps for State<'a> {
    fn add_line(&mut self, start: [f32; 2], end: [f32; 2]) {
        self.vertices.push(Vertex {
            position: [start[0] / self.zoom, start[1] / self.zoom, 0.0],
            color: [1.0, 1.0, 1.0],
        });
        self.vertices.push(Vertex {
            position: [end[0] / self.zoom, end[1] / self.zoom, 0.0],
            color: [1.0, 1.0, 1.0],
        });
        self.update_vertex_buffer();
    }

    fn update_line(&mut self, position: [f32; 2]) {
        let world_x = position[0] / self.zoom; // Apply zoom correctly
        let world_y = position[1] / self.zoom;

        self.vertices[(self.num_vertices - 1) as usize] = Vertex {
            position: [world_x, world_y, 0.0],
            color: [1.0, 1.0, 1.0],
        };
        self.update_vertex_buffer();
    }
}
