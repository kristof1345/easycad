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
    pub thickness: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CircleInstance {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub radius: f32,
    pub thickness: f32,
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

impl CircleInstance {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CircleInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: 28,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

pub trait CircleOps {
    fn add_circle(
        &mut self,
        coordinates: [f32; 2],
        radius: f32,
        color: [f32; 3],
        selected_flag: bool,
        del_flag: bool,
        is_drawing: bool,
    );
    fn update_circle(&mut self, position: [f32; 2], is_drawing_flag: bool);
    fn cancel_drawing_circle(&mut self);
    fn unselect_circles(&mut self);
}

// flatten vector of circles into flat vector of vertices
pub fn flatten_circles_to_instances(
    circles: &mut Vec<Circle>,
    color_scheme: ColorScheme,
    // zoom: f32,
) -> Vec<CircleInstance> {
    circles
        .iter()
        .map(|circle| CircleInstance {
            position: circle.center.position,
            color: if circle.selected {
                [1.0, 0.0, 0.0]
            } else if color_scheme == ColorScheme::Light {
                [0.0, 0.0, 0.0]
            } else {
                [1.0, 1.0, 1.0]
            },
            radius: circle.radius,
            thickness: circle.thickness,
        })
        .collect()
}

pub fn flatten_circles_for_snap(circles: &mut Vec<Circle>) -> Vec<Vertex> {
    let mut flat = Vec::new();

    for circle in circles.iter_mut() {
        if !circle.is_drawing {
            let x = circle.center.position[0];
            let y = circle.center.position[1];

            flat.extend([
                Vertex {
                    position: [x, y + circle.radius, 0.0],
                    color: circle.center.color,
                }, // vertex above venter point
                Vertex {
                    position: [x - circle.radius, y, 0.0],
                    color: circle.center.color,
                }, // vertex to the left of center point
                Vertex {
                    position: [x, y - circle.radius, 0.0],
                    color: circle.center.color,
                }, // vertex below venter point
                Vertex {
                    position: [x + circle.radius, y, 0.0],
                    color: circle.center.color,
                }, // vertex to the right of venter point
                circle.center,
            ]);
        }
    }

    flat
}

impl<'a> CircleOps for State<'a> {
    fn add_circle(
        &mut self,
        coordinates: [f32; 2],
        radius: f32,
        color: [f32; 3],
        selected_flag: bool,
        del_flag: bool,
        is_drawing: bool,
    ) {
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
                thickness: 5.0,
            }
        });

        if is_drawing {
            let index = self.circles.len() - 1;
            self.active_circle_index = Some(index);
        }

        self.update_circle_instance_buffer();
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
            self.update_circle_instance_buffer();
        }
    }

    fn cancel_drawing_circle(&mut self) {
        self.circles.pop();
        self.drawing_state = DrawingState::Idle;
        self.mode = Mode::Normal;
        self.update_circle_instance_buffer();
    }

    fn unselect_circles(&mut self) {
        for circle in &mut self.circles {
            if circle.selected {
                circle.selected = false;
            }
        }

        self.update_circle_instance_buffer();
    }
}
