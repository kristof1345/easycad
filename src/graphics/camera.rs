use bytemuck::{Pod, Zeroable};

pub struct Camera {
    pub x_offset: f32,
    pub y_offset: f32,
    pub zoom: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
    matrix: [[f32; 4]; 4], // 48 bytes
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

    pub fn to_uniform(&self, window_width: f32, window_height: f32) -> CameraUniform {
        CameraUniform {
            matrix: [
                [self.zoom, 0.0, 0.0, 0.0],
                [0.0, self.zoom, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [
                    -self.x_offset * self.zoom,
                    -self.y_offset * self.zoom,
                    0.0,
                    1.0,
                ],
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
        self.zoom = self.zoom.clamp(0.01, 100.0);
    }

    pub fn zoom_at_cursor(&mut self, factor: f32, mouse_world_x: f32, mouse_world_y: f32) {
        let old_zoom = self.zoom;
        self.zoom *= factor;
        self.zoom = self.zoom.clamp(0.1, 1000.0);

        self.x_offset = mouse_world_x - ((mouse_world_x - self.x_offset) * old_zoom / self.zoom);
        self.y_offset = mouse_world_y - ((mouse_world_y - self.y_offset) * old_zoom / self.zoom);
    }
}
