struct CameraUniform {
    matrix: mat4x4<f32>,
    window_size: vec2<f32>,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct InstanceInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) radius: f32,
    @location(3) thickness: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) local_pos: vec2<f32>,   // <--- We pass the offset from center to fragment
    @location(2) thickness: f32,         // Pass to fragment
    @location(3) radius: f32,            // Pass to fragment
};

@vertex
fn vs_main(
    @builtin(vertex_index) v_index: u32,
    input: InstanceInput
) -> VertexOutput {
    var out: VertexOutput;
    out.color = input.color;
    out.radius = input.radius;
    out.thickness = input.thickness;

    // 1. Define a unit square (-1 to +1) based on vertex index
    // 0: (-1, -1), 1: (-1, 1), 2: (1, -1), 3: (1, 1)
    // This creates a "Triangle Strip" quad
    var corner = vec2<f32>(0.0, 0.0);
    if (v_index == 0u) { corner = vec2<f32>(-1.0, -1.0); }
    else if (v_index == 1u) { corner = vec2<f32>(-1.0, 1.0); }
    else if (v_index == 2u) { corner = vec2<f32>(1.0, -1.0); }
    else { corner = vec2<f32>(1.0, 1.0); }

    // 2. Calculate the size of the billboard
    // We need the quad to be big enough to hold the outer edge of the thick circle
    let outer_radius = input.radius + (input.thickness * 0.5);

    // This is the offset in PIXELS from the circle center
    let pixel_offset = corner * outer_radius;

    // Save this for the fragment shader (so it knows how far it is from center)
    out.local_pos = pixel_offset;

    // 3. Move the quad to the correct world position
    let world_pos = input.position.xy + pixel_offset;

    // 4. Standard Camera Projection (Same as your lines)
    let transformed_pos = camera.matrix * vec4<f32>(world_pos, 0.0, 1.0);
    let clip_x = transformed_pos.x / (camera.window_size.x * 0.5);
    let clip_y = transformed_pos.y / (camera.window_size.y * 0.5);

    out.clip_position = vec4<f32>(clip_x, clip_y, input.position.z, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Calculate distance from the center of the circle
    let dist = length(in.local_pos);

    // 2. Define the Ring limits
    let radius = in.radius;
    let half_thick = in.thickness * 0.5;

    // 3. Hard Discard (Basic version)
    // If we are outside the circle OR inside the hole, discard pixel
    if (dist > radius + half_thick || dist < radius - half_thick) {
        discard;
    }

    return vec4<f32>(in.color, 1.0);
}
