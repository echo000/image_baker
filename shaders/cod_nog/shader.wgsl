// Fragment shaders for CoD NOG (Normal/Occlusion/Gloss) processing
// Note: Uses shared fullscreen quad vertex shader (vs_main)

struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@group(0) @binding(0)
var input_texture: texture_2d<f32>;
@group(0) @binding(1)
var input_sampler: sampler;

// Processes Call of Duty: Infinite Warfare/Modern Warfare NOG textures
// Input channels:
//   Red: Gloss Map
//   Green: Normal X (Hemi-Octahedron)
//   Blue: Occlusion
//   Alpha: Normal Y (Hemi-Octahedron)
// References:
//   https://www.activision.com/cdn/research/2017_DD_Rendering_of_COD_IW.pdf
//   http://jcgt.org/published/0003/02/01/
//   http://media.steampowered.com/apps/valve/2015/Alex_Vlachos_Advanced_VR_Rendering_GDC2015.pdf

// Output: Gloss map (red channel as grayscale)
@fragment
fn fs_red(input: VertexOutput) -> @location(0) vec4<f32> {
    let colour = textureSample(input_texture, input_sampler, input.tex_coords);
    return vec4<f32>(colour.r, colour.r, colour.r, 1.0);
}

// Output: Occlusion map (blue channel as grayscale)
@fragment
fn fs_blue(input: VertexOutput) -> @location(0) vec4<f32> {
    let colour = textureSample(input_texture, input_sampler, input.tex_coords);
    return vec4<f32>(colour.b, colour.b, colour.b, 1.0);
}

// Output: Decoded normal map (from green and alpha channels using hemi-octahedron decoding)
@fragment
fn fs_green_alpha(input: VertexOutput) -> @location(0) vec4<f32> {
    let colour = textureSample(input_texture, input_sampler, input.tex_coords);

    // Extract the two encoded channels and map from [0,1] to [-1,1] range
    let normalVector = vec2<f32>(colour.g * 2.0 - 1.0, colour.a * 2.0 - 1.0);

    // Rotate and scale the unit square back to the center diamond
    // This is the key transformation: (x+y, x-y) * 0.5
    let xy = vec2<f32>(
        normalVector.x + normalVector.y,
        normalVector.x - normalVector.y
    ) * 0.5;

    // Reconstruct the 3D normal with Z component
    let z = 1.0 - abs(xy.x) - abs(xy.y);
    var xyz = vec3<f32>(xy.x, xy.y, z);

    // Normalize to ensure unit length
    xyz = normalize(xyz);

    // Convert from [-1,1] range to [0,1] range for RGB storage
    let rgb_normal = xyz * 0.5 + 0.5;

    // Output as RGB normal map with full opacity
    return vec4<f32>(rgb_normal, 1.0);
}
