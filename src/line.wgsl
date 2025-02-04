// struct VertexOutput {
//     @builtin(position) clip_position: vec4<f32>,
//     @location(0) vert_pos: vec3<f32>,
// };
//
// @vertex
// fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
//     var out: VertexOutput;
//
//     // Define the two endpoints of the line
//     let p0 = vec2<f32>(-0.5, -0.5); // Start of the line
//     let p1 = vec2<f32>(0.5, 0.5);   // End of the line
//
//     // Select the correct vertex position based on the vertex index
//     let pos = select(p1, p0, in_vertex_index == 0u);
//
//     out.clip_position = vec4<f32>(pos, 0.0, 1.0);
//     out.vert_pos = out.clip_position.xyz;
//
//     return out;
// }

struct VertexInput {
    @location(0) position: vec3<f32>,  // Position from the vertex buffer
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
    out.clip_position = vec4<f32>(model.position, 1.0); // Transform position to clip space
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
