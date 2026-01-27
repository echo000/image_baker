// Fragment shader for merging RGB and alpha textures
// Note: Uses shared fullscreen quad vertex shader (vs_main)
struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// Fragment shaders for CoD WW2 Green Image channel split
@group(0) @binding(0)
var input_texture: texture_2d<f32>;
@group(0) @binding(1)
var input_sampler: sampler;

// Splits packed RGB channels from Call of Duty: World War II "Green Image"
// R = Specular
// G = Occlusion
// B = Gloss

// Output: Specular map (red channel as grayscale)
@fragment
fn fs_red(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = textureSample(input_texture, input_sampler, input.tex_coords);
    return vec4<f32>(pixel.r, pixel.r, pixel.r, 1.0);
}

// Output: Occlusion map (green channel as grayscale)
@fragment
fn fs_green(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = textureSample(input_texture, input_sampler, input.tex_coords);
    return vec4<f32>(pixel.g, pixel.g, pixel.g, 1.0);
}

// Output: Gloss map (blue channel as grayscale)
@fragment
fn fs_blue(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = textureSample(input_texture, input_sampler, input.tex_coords);
    return vec4<f32>(pixel.b, pixel.b, pixel.b, 1.0);
}
