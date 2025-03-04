
struct CameraUniform {
    matrix: mat4x4<f32>,
    window_size: vec2<f32>,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct CircleUniform {
    center: vec2<f32>,
    radius: f32,
    color: vec3<f32>,
    segments: u32,
};
@group(1) @binding(0) var<uniform> circle: CircleUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let angle = 2.0 * 3.1415926535 * f32(vertex_index) / f32(circle.segments);
    let x = circle.center.x + circle.radius * cos(angle);
    let y = circle.center.y + circle.radius * sin(angle);
    let world_pos = vec4<f32>(x, y, 0.0, 1.0);

    let transformed_pos = camera.matrix * world_pos;
    let clip_x = transformed_pos.x / (camera.window_size.x * 0.5);
    let clip_y = transformed_pos.y / (camera.window_size.y * 0.5);

    var out: VertexOutput;
    out.clip_position = vec4<f32>(clip_x, clip_y, 0.0, 1.0);
    out.color = circle.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
