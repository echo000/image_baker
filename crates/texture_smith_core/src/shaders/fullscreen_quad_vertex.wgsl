// Shared vertex shader for fullscreen quad rendering
// Used by all texture processing shaders in the image_merge tool

struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Generate a fullscreen quad using vertex index
    // Two triangles covering the entire screen from -1 to 1 in NDC space
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),  // Bottom-left
        vec2<f32>(-1.0, 1.0),   // Top-left
        vec2<f32>(1.0, -1.0),   // Bottom-right
        vec2<f32>(1.0, -1.0),   // Bottom-right
        vec2<f32>(-1.0, 1.0),   // Top-left
        vec2<f32>(1.0, 1.0)     // Top-right
    );

    // UV coordinates mapping: (0,0) top-left to (1,1) bottom-right
    var tex_coords = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0)
    );

    var output: VertexOutput;
    output.vert_pos = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    output.tex_coords = tex_coords[vertex_index];
    return output;
}
