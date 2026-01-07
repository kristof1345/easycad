struct CameraUniform {
    matrix: mat4x4<f32>,
    window_size: vec2<f32>,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct InstanceInput {
    @location(0) start_pos: vec3<f32>,
    @location(1) end_pos: vec3<f32>,
    @location(2) color: vec3<f32>,
    @location(3) thickness: f32, // <--- Per-line width input
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) v_index: u32,
    input: InstanceInput
) -> VertexOutput {
    var out: VertexOutput;
    out.color = input.color;

    // 1. Project World Points to "Pixel Space"
    // Apply camera matrix to get screen positions
    let start_proj = camera.matrix * vec4<f32>(input.start_pos.xy, 0.0, 1.0);
    let end_proj   = camera.matrix * vec4<f32>(input.end_pos.xy, 0.0, 1.0);

    // Get 2D screen coordinates
    let p0 = start_proj.xy;
    let p1 = end_proj.xy;

    // 2. Compute Segment Vectors
    let dir = p1 - p0;

    // Normal vector (perpendicular to direction)
    // Rotated 90 degrees: (x, y) -> (-y, x)
    let normal = normalize(vec2<f32>(-dir.y, dir.x));

    // 3. Apply Thickness
    // Scale the normal by half the width to get the offset
    let offset = normal * (input.thickness / 2.0);

    // 4. Generate the 4 Corners
    // TriangleStrip order: 0:StartTop, 1:StartBottom, 2:EndTop, 3:EndBottom
    var pos_pixel: vec2<f32>;

    if (v_index == 0u) {
        pos_pixel = p0 + offset;
    } else if (v_index == 1u) {
        pos_pixel = p0 - offset;
    } else if (v_index == 2u) {
        pos_pixel = p1 + offset;
    } else { // v_index == 3u
        pos_pixel = p1 - offset;
    }

    // 5. Convert Pixel Space back to Clip Space (-1.0 to 1.0)
    // Note: This assumes your camera matrix outputted pixels relative to center
    // Adjust based on your specific coordinate system logic
    let clip_x = pos_pixel.x / (camera.window_size.x * 0.5);
    let clip_y = pos_pixel.y / (camera.window_size.y * 0.5);

    out.clip_position = vec4<f32>(clip_x, clip_y, input.start_pos.z, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
