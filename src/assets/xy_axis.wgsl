// struct VertexInput {
//     @location(0) position: vec3<f32>,
//     @location(1) color: vec3<f32>,
// }

// struct VertexOutput {
//     @builtin(position) clip_position: vec4<f32>,
//     @location(0) color: vec3<f32>,
// }

// @vertex
// fn vs_main(model: VertexInput) -> VertexOutput {
//     var out: VertexOutput;
//     out.color = model.color;
//     out.clip_position = vec4<f32>(model.position, 1.0);
//     return out;
// }

// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     return vec4<f32>(in.color, 1.0);
// }







struct CameraUniform {
    matrix: mat4x4<f32>,
    window_size: vec2<f32>, // Width and height in pixels
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,  // World space (e.g., pixels)
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;

    var matrix = camera.matrix;
    matrix[0][0] = 1.0;
    matrix[1][1] = 1.0;

    let transformed_pos = matrix * vec4<f32>(model.position.xy, 0.0, 1.0);

    let clip_x = transformed_pos.x / (camera.window_size.x * 0.5);
    let clip_y = transformed_pos.y / (camera.window_size.y * 0.5);    

    out.clip_position = vec4<f32>(clip_x, clip_y, model.position.z, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
    // return vec4<f32>(in.clip_position.xy * 0.5 + 0.5, 0.0, 1.0);
}
