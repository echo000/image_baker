// Fragment shaders for CoD Pack NOG
// Note: Uses shared fullscreen quad vertex shader (vs_main)

struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// Fragment shader for packing NOG (Normal/Occlusion/Gloss) textures
@group(0) @binding(0)
var normal_texture: texture_2d<f32>;
@group(0) @binding(1)
var normal_sampler: sampler;

@group(0) @binding(2)
var gloss_texture: texture_2d<f32>;
@group(0) @binding(3)
var gloss_sampler: sampler;

@group(0) @binding(4)
var occlusion_texture: texture_2d<f32>;
@group(0) @binding(5)
var occlusion_sampler: sampler;

// Packs separate textures into Call of Duty: Infinite Warfare/Modern Warfare NOG format
// Input textures:
//   Gloss: Grayscale gloss map
//   Normal: RGB normal map (tangent space)
//   Occlusion: Grayscale occlusion/AO map
// Output channels:
//   Red: Gloss value
//   Green: Normal X (Hemi-Octahedron encoded)
//   Blue: Occlusion value
//   Alpha: Normal Y (Hemi-Octahedron encoded)
// References:
//   https://www.activision.com/cdn/research/2017_DD_Rendering_of_COD_IW.pdf
//   http://jcgt.org/published/0003/02/01/
//   http://media.steampowered.com/apps/valve/2015/Alex_Vlachos_Advanced_VR_Rendering_GDC2015.pdf

@fragment
fn fs_nog(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample input textures
    let gloss = textureSample(gloss_texture, gloss_sampler, input.tex_coords);
    let normal_rgb = textureSample(normal_texture, normal_sampler, input.tex_coords);
    let occlusion = textureSample(occlusion_texture, occlusion_sampler, input.tex_coords);

    // Convert normal from [0,1] RGB to [-1,1] XYZ and normalize
    var v = normal_rgb.xyz * 2.0 - 1.0;
    v = normalize(v);

    // Hemi-octahedron encoding (hemisphere only, z >= 0)
    // Project the hemisphere onto the hemi-octahedron base
    let l1norm = abs(v.x) + abs(v.y) + abs(v.z);
    let projected = v.xy * (1.0 / l1norm);

    // Apply rotation and scale to map diamond to square
    // This is CoD's specific transformation
    let enc = vec2<f32>(
        projected.x + projected.y,
        projected.x - projected.y
    );

    // Convert from [-1,1] to [0,1] range for texture storage
    let enc_01 = enc * 0.5 + 0.5;

    // Pack into NOG format:
    // R = Gloss (red channel from gloss map)
    // G = Normal X (hemi-octahedron encoded)
    // B = Occlusion (red channel from occlusion map)
    // A = Normal Y (hemi-octahedron encoded)
    return vec4<f32>(
        gloss.r,       // Red: Gloss
        enc_01.x,      // Green: Encoded Normal X
        occlusion.r,   // Blue: Occlusion
        enc_01.y       // Alpha: Encoded Normal Y
    );
}
