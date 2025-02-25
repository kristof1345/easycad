pub struct Camera {
    pub x_offset: f32,
    pub y_offset: f32,
    pub zoom: f32,
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
            [self.zoom, 0.0, self.x_offset, 0.0],
            [0.0, self.zoom, self.y_offset, 0.0],
            [0.0, 0.0, 1.0, 0.0],
        ]
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
