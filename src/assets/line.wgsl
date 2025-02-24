struct VertexInput {
    @location(0) position: vec3<f32>,  // Position from the vertex buffer
    @location(1) color: vec3<f32>,     // Color from the vertex buffer
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@group(0) @binding(0)
var<uniform> zoom: f32; // Uniform var for zoom

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;  // Pass the color to the fragment shader
    out.clip_position = vec4<f32>(model.position.xy * zoom, model.position.z, 1.0); // Transform position to clip space
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
