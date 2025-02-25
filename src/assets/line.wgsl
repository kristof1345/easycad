// struct VertexInput {
//     @location(0) position: vec3<f32>,  // Position from the vertex buffer
//     @location(1) color: vec3<f32>,     // Color from the vertex buffer
// };

// struct VertexOutput {
//     @builtin(position) clip_position: vec4<f32>,
//     @location(0) color: vec3<f32>,
// };

// @group(0) @binding(0)
// var<uniform> zoom: f32; // Uniform var for zoom

// @vertex
// fn vs_main(model: VertexInput) -> VertexOutput {
//     var out: VertexOutput;
//     out.color = model.color;  // Pass the color to the fragment shader
//     out.clip_position = vec4<f32>(model.position.xy * zoom, model.position.z, 1.0); // Transform position to clip space
//     return out;
// }

// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     return vec4<f32>(in.color, 1.0);
// }



// struct CameraUniform {
    // matrix: mat3x3<f32>, // Transformation matrix (panning + zooming)
// };
@group(0) @binding(0)
var<uniform> camera: mat3x3<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,  // Position from the vertex buffer (clip space)
    @location(1) color: vec3<f32>,     // Color from the vertex buffer
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;  // Pass the color to the fragment shader

    // Apply the camera transformation matrix to the clip-space position
    let transformed_pos = camera * vec3<f32>(model.position.xy, 1.0);

    // Output to clip space, preserving z and adding w=1.0
    out.clip_position = vec4<f32>(transformed_pos.xy, model.position.z, 1.0);
    return out;
}

// Fragment shader (unchanged)
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
