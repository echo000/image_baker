// Fragment shader for merging RGB and alpha textures
// Note: Uses shared fullscreen quad vertex shader (vs_main)
struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// Fragment shaders for Deathloop Packed Texture channel split
@group(0) @binding(0)
var input_texture: texture_2d<f32>;
@group(0) @binding(1)
var input_sampler: sampler;

// Splits packed texture from Deathloop
// R = Ambient Occlusion
// G = Roughness
// B = Metallic

// Output: Ambient Occlusion map (red channel as grayscale)
@fragment
fn fs_red(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = textureSample(input_texture, input_sampler, input.tex_coords);
    return vec4<f32>(pixel.r, pixel.r, pixel.r, 1.0);
}

// Output: Roughness map (green channel as grayscale)
@fragment
fn fs_green(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = textureSample(input_texture, input_sampler, input.tex_coords);
    return vec4<f32>(pixel.g, pixel.g, pixel.g, 1.0);
}

// Output: Metallic map (blue channel as grayscale)
@fragment
fn fs_blue(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = textureSample(input_texture, input_sampler, input.tex_coords);
    return vec4<f32>(pixel.b, pixel.b, pixel.b, 1.0);
}
