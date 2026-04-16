// Fragment shaders for CoD Pack NOG
// Note: Uses shared fullscreen quad vertex shader (vs_main)

struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// Fragment shader for packing NOG (Normal/Occlusion/Gloss) textures
@group(0) @binding(0)
var spec_texture: texture_2d<f32>;
@group(0) @binding(1)
var spec_sampler: sampler;

@group(0) @binding(2)
var gloss_texture: texture_2d<f32>;
@group(0) @binding(3)
var gloss_sampler: sampler;

// Packs separate textures into Call of Duty: Infinite Warfare/Modern Warfare NOG format
// Input textures:
//   Gloss: Grayscale gloss map
//   Specular: RGB normal map (tangent space)
// Output channels:
//   Red: Specular value R
//   Green: Specular value G
//   Blue: Specular value B
//   Alpha: Gloss

@fragment
fn fs_spec(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample input textures
    let spec = textureSample(spec_texture, spec_sampler, input.tex_coords);
    let gloss = textureSample(gloss_texture, gloss_sampler, input.tex_coords);

    return vec4<f32>(
        spec.r,       // Red: Spec
        spec.g,       // Green: Spec
        spec.b,       // Blue: Spec
        gloss.x       // Alpha: Gloss
    );
}
