use crate::State;
use crate::Vertex;

#[derive(Copy, Clone, Debug)]
pub struct Circle {
    pub radius: f32,
    pub center: Vertex,
}

pub trait CircleOps {
    fn add_circle(&mut self, coordinates: [f32; 2], radius: f32, color: [f32; 3]);
    fn update_circle(&mut self, position: [f32; 2]);
}

pub fn flatten_circles(circles: &Vec<Circle>) -> Vec<Vertex> {
    let mut flat = Vec::new();
    let n = 36;

    for circle in circles.iter() {
        for i in 0..n {
            let theta = 2.0 * std::f32::consts::PI * (i as f32) / (n as f32);
            let x = circle.center.position[0] + circle.radius * theta.cos();
            let y = circle.center.position[1] + circle.radius * theta.sin();
            flat.push(Vertex {
                position: [x, y, 0.0],
                color: circle.center.color,
            });
        }

        // push the first element again to close the circle
        flat.push(flat[flat.len() - n]);
    }

    flat
}

impl<'a> CircleOps for State<'a> {
    fn add_circle(&mut self, coordinates: [f32; 2], radius: f32, color: [f32; 3]) {
        self.circles.push({
            Circle {
                center: Vertex {
                    position: [coordinates[0], coordinates[1], 0.0],
                    color,
                },
                radius,
            }
        });

        println!("Added circle, circles: {:?}", self.circles);

        self.update_circle_vertex_buffer();
    }

    fn update_circle(&mut self, position: [f32; 2]) {
        let world_x = position[0];
        let world_y = position[1];

        let length = self.circles.len();
        let mut circle = self.circles[(length - 1) as usize];
        let center = circle.center;

        let dx = world_x - center.position[0];
        let dy = world_y - center.position[1];

        circle.radius = (dx * dx + dy * dy).sqrt();

        self.update_vertex_buffer();
    }
}
