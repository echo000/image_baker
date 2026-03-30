// Fragment shader for RGBA values

struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// Parameters uniform buffer (group 1)
struct Parameters {
    R: f32,
    G: f32,
    B: f32,
    A: f32,
}

@group(0) @binding(1)
var<uniform> params: Parameters;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Output the specified RGBA values as a solid color
    return vec4<f32>(params.R, params.G, params.B, params.A);
}
