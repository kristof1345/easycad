use crate::Vertex;

#[derive(Copy, Clone, Debug)]
pub struct Circle {
    pub radius: f32,
    pub center: Vertex,
}

pub fn flatten_circles(circles: &Vec<Circle>) -> Vec<Vertex> {
    let mut flat = Vec::new();

    for circle in circles.iter() {
        flat.push(circle.center);
    }

    flat
}
