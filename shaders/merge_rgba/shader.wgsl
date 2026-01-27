// Fragment shader for Merge RGB/Alpha
// Note: Uses shared fullscreen quad vertex shader (vs_main)

struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@group(0) @binding(0)
var rgb_texture: texture_2d<f32>;
@group(0) @binding(1)
var rgb_sampler: sampler;

@group(0) @binding(2)
var alpha_texture: texture_2d<f32>;
@group(0) @binding(3)
var alpha_sampler: sampler;

// Merges separate RGB and alpha textures into a single RGBA texture
// RGB texture: Colour data
// Alpha texture: Alpha/transparency data (uses red channel)

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample RGB colour from main texture
    let rgb = textureSample(rgb_texture, rgb_sampler, input.tex_coords);

    // Sample alpha from additional texture (use red channel)
    let alpha_pixel = textureSample(alpha_texture, alpha_sampler, input.tex_coords);

    // Combine RGB with alpha channel
    return vec4<f32>(rgb.r, rgb.g, rgb.b, alpha_pixel.r);
}
