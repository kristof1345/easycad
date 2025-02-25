use bytemuck::{Pod, Zeroable};

pub struct Camera {
    pub x_offset: f32,
    pub y_offset: f32,
    pub zoom: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
    matrix: [[f32; 4]; 3], // 48 bytes
    window_size: [f32; 2], // 8 bytes
    _padding: [f32; 2],    // 8 bytes, total 64 bytes for alignment
}

impl Camera {
    pub fn new(x: f32, y: f32, zoom: f32) -> Self {
        Camera {
            x_offset: x,
            y_offset: y,
            zoom,
        }
    }

    pub fn to_matrix(&self) -> [[f32; 4]; 3] {
        [
            [self.zoom, 0.0, -self.x_offset, 0.0],
            [0.0, self.zoom, -self.y_offset, 0.0],
            [0.0, 0.0, 1.0, 0.0],
        ]
    }

    pub fn to_uniform(&self, window_width: f32, window_height: f32) -> CameraUniform {
        CameraUniform {
            matrix: [
                [self.zoom, 0.0, self.x_offset, 0.0],
                [0.0, self.zoom, self.y_offset, 0.0],
                [0.0, 0.0, 1.0, 0.0],
            ],
            window_size: [window_width, window_height],
            _padding: [0.0, 0.0],
        }
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.x_offset += dx;
        self.y_offset += dy;
    }

    pub fn zoom(&mut self, factor: f32) {
        self.zoom *= factor;
        self.zoom = self.zoom.clamp(0.1, 10.0);
    }
}
