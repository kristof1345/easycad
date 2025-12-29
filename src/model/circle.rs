use crate::graphics::gui_elements::ColorScheme;
use crate::graphics::vertex::Vertex;
use crate::{DrawingState, Mode, State};

#[derive(Copy, Clone, Debug)]
pub struct Circle {
    pub radius: f32,
    pub center: Vertex,
    pub selected: bool,
    pub del: bool,
    pub is_drawing: bool,
}

impl Circle {
    pub fn move_circle(&mut self, dx: f32, dy: f32) {
        self.center.position[0] -= dx;
        self.center.position[1] -= dy;
    }

    pub fn finish_circle_with_radius(&mut self, radius: f32) {
        self.radius = radius;
        self.is_drawing = false;
    }
}

pub trait CircleOps {
    fn add_circle(&mut self, coordinates: [f32; 2], radius: f32, color: [f32; 3], selected_flag: bool, del_flag: bool, is_drawing: bool);
    fn update_circle(&mut self, position: [f32; 2], is_drawing_flag: bool);
    fn cancel_drawing_circle(&mut self);
    fn unselect_circles(&mut self);
}

// flatten vector of circles into flat vector of vertices
pub fn flatten_circles(circles: &mut Vec<Circle>, color_scheme: ColorScheme) -> Vec<Vertex> {
    let mut flat = Vec::new();
    let n = 36;

    for circle in circles.iter_mut() {
        if circle.selected {
            circle.center.color = [1.0, 0.0, 0.0];
        } else if color_scheme == ColorScheme::Light {
            circle.center.color = [0.0, 0.0, 0.0];
        } else {
            circle.center.color = [1.0, 1.0, 1.0];
        }

        for i in 0..n {
            let theta = 2.0 * std::f32::consts::PI * (i as f32) / (n as f32);
            let x = circle.center.position[0] + circle.radius * theta.cos();
            let y = circle.center.position[1] + circle.radius * theta.sin();
            flat.push(Vertex {
                position: [x, y, 0.0],
                color: circle.center.color,
            });
        }
    }

    flat
}

pub fn flatten_circles_for_snap(circles: &mut Vec<Circle>) -> Vec<Vertex> {
    let mut flat = Vec::new();

    for circle in circles.iter_mut() {
        if !circle.is_drawing {
            let x = circle.center.position[0];
            let y = circle.center.position[1];
    
            flat.extend([
                Vertex{ position: [x, y + circle.radius, 0.0], color: circle.center.color }, // vertex above venter point
                Vertex { position: [x - circle.radius, y, 0.0], color: circle.center.color }, // vertex to the left of center point
                Vertex { position: [x, y - circle.radius, 0.0], color: circle.center.color  }, // vertex below venter point
                Vertex { position: [x + circle.radius, y, 0.0], color: circle.center.color }, // vertex to the right of venter point
                circle.center
            ]);
        }
    }

    flat
}

impl<'a> CircleOps for State<'a> {
    fn add_circle(&mut self, coordinates: [f32; 2], radius: f32, color: [f32; 3], selected_flag: bool, del_flag: bool, is_drawing: bool) {
        let segments = 36;

        let all_vertices = flatten_circles(&mut self.circles, self.ui.theme.color_scheme);

        let base_index = all_vertices.len() as u32;

        self.circles.push({
            Circle {
                center: Vertex {
                    position: [coordinates[0], coordinates[1], 0.0],
                    color,
                },
                radius,
                selected: selected_flag,
                del: del_flag,
                is_drawing,
            }
        });

        for i in 0..segments - 1 {
            self.circle_indices.push(base_index + i);
            self.circle_indices.push(base_index + i + 1);
        }
        self.circle_indices.push(base_index + segments - 1);
        self.circle_indices.push(base_index);

        if is_drawing {
            let index = self.circles.len() - 1;
            self.active_circle_index = Some(index);
        }

        self.update_circle_vertex_buffer();
    }

    fn update_circle(&mut self, position: [f32; 2], is_drawing_flag: bool) {
        if let Some(i) = self.active_circle_index {
            let world_x = position[0];
            let world_y = position[1];

            let circle = &mut self.circles[i as usize];
            let center = circle.center;
            let dx = world_x - center.position[0];
            let dy = world_y - center.position[1];
            circle.radius = (dx * dx + dy * dy).sqrt();
            circle.is_drawing = is_drawing_flag;

            if !is_drawing_flag {
                self.active_circle_index = None;
            }
            self.update_circle_vertex_buffer();
        }
    }

    fn cancel_drawing_circle(&mut self) {
        self.circles.pop();
        self.circle_indices.truncate(self.circle_indices.len() - 72);
        self.drawing_state = DrawingState::Idle;
        self.mode = Mode::Normal;
        self.update_circle_vertex_buffer();
    }

    fn unselect_circles(&mut self) {
        for circle in &mut self.circles {
            if circle.selected {
                circle.selected = false;
            }
        }

        self.update_circle_vertex_buffer();
    }
}
