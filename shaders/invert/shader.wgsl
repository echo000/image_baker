// Fragment shader for RGB inversion
// Note: Uses shared fullscreen quad vertex shader (vs_main)

struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@group(0) @binding(0)
var input_texture: texture_2d<f32>;
@group(0) @binding(1)
var input_sampler: sampler;

// Inverts RGB colour values by computing (1.0 - colour) for each channel
// Alpha channel is preserved
// Useful for inverting roughness maps, AO maps, or creating negative-like effects

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = textureSample(input_texture, input_sampler, input.tex_coords);

    // Invert RGB values: 1.0 - value (clamped to 0..1 range)
    let inverted_r = 1.0 - clamp(pixel.r, 0.0, 1.0);
    let inverted_g = 1.0 - clamp(pixel.g, 0.0, 1.0);
    let inverted_b = 1.0 - clamp(pixel.b, 0.0, 1.0);

    // Preserve original alpha channel
    return vec4<f32>(inverted_r, inverted_g, inverted_b, pixel.a);
}
