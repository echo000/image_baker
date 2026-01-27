// Fragment shader for merging RGB and alpha textures
// Note: Uses shared fullscreen quad vertex shader (vs_main)
struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// Fragment shaders for CoD Spec/Gloss (RGB/A) split
@group(0) @binding(0)
var input_texture: texture_2d<f32>;
@group(0) @binding(1)
var input_sampler: sampler;

// Splits Call of Duty Spec/Gloss textures
// Input format:
//   RGB: Specular colour
//   Alpha: Gloss value

// Output: Specular map (RGB channels)
@fragment
fn fs_rgb(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = textureSample(input_texture, input_sampler, input.tex_coords);
    return vec4<f32>(pixel.r, pixel.g, pixel.b, 1.0);
}

// Output: Gloss map (alpha channel as grayscale)
@fragment
fn fs_alpha(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = textureSample(input_texture, input_sampler, input.tex_coords);
    return vec4<f32>(pixel.a, pixel.a, pixel.a, 1.0);
}
