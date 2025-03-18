struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) position: vec2<f32>, // Pass world-space position to fragment
};

struct FragmentInput {
    @location(0) color: vec3<f32>,
    @location(1) position: vec2<f32>,
};

struct CircleUniform {
    radius: f32,
    // Add more if needed, e.g., screen resolution for scaling
};

@group(0) @binding(0)
var<uniform> circle_uniform: CircleUniform;

@vertex
fn vs_main(
    model: VertexInput,
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;

    let size = circle_uniform.radius * 2.0; // Quad width/height = diameter

    // Assign offset based on vertex_index (0, 1, 2, 3 for quad corners)
    var offset: vec2<f32>;
    if (vertex_index % 4u == 0u) {
        offset = vec2<f32>(-size, -size); // Bottom-left
    } else if (vertex_index % 4u == 1u) {
        offset = vec2<f32>(size, -size);  // Bottom-right
    } else if (vertex_index % 4u == 2u) {
        offset = vec2<f32>(-size, size);  // Top-left
    } else {
        offset = vec2<f32>(size, size);   // Top-right
    }

    let world_pos = model.position.xy + offset;
    out.position = world_pos; // Pass to fragment shader
    out.clip_position = vec4<f32>(world_pos, model.position.z, 1.0);
    return out;
}

@fragment
fn fs_main(in: FragmentInput) -> @location(0) vec4<f32> {
    // Distance from fragment to center (in.position is world-space)
    let center = in.position - circle_uniform.radius; // Adjust based on vertex offset
    let dist = length(in.position);

    // Discard fragments outside the radius
    if (dist > circle_uniform.radius) {
        discard;
    }

    return vec4<f32>(in.color, 1.0);
}
