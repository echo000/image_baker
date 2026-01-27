// Fragment shader for CoD Fused Colour + Specular
// Note: Uses shared fullscreen quad vertex shader (vs_main)

struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// Fragment shaders for CoD Fused Colour/Specular separation
@group(0) @binding(0)
var input_texture: texture_2d<f32>;
@group(0) @binding(1)
var input_sampler: sampler;

// Separates fused colour/specular from Call of Duty: Infinite Warfare/Modern Warfare
// Input format:
//   RGB: Fused diffuse and specular colours
//   Alpha: Metalness mask
// References:
//   https://www.activision.com/cdn/research/2017_DD_Rendering_of_COD_IW.pdf
//   GameImageUtil by Philip/Scobalula

const INSULATOR_SPEC_RANGE: f32 = 0.1;
const MIN_SPECULAR: f32 = 0.21; // 56/255 - clamped to 56/56/56

// Output: Diffuse/Albedo colour map
@fragment
fn fs_albedo(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = textureSample(input_texture, input_sampler, input.tex_coords);

    // Compute metal mask: clamp(pixel.W - insulatorSpecRange, max: 1.0, min: 0.0) * (1.0 / (1.0 - insulatorSpecRange))
    let m = clamp(pixel.a - INSULATOR_SPEC_RANGE, 0.0, 1.0) * (1.0 / (1.0 - INSULATOR_SPEC_RANGE));

    // Compute diffuse mask: clamp(1.0 - m, max: 1.0, min: 0.0)
    let d = clamp(1.0 - m, 0.0, 1.0);

    // Diffuse colour = RGB * diffuse mask
    let albedo = vec3<f32>(pixel.r * d, pixel.g * d, pixel.b * d);

    return vec4<f32>(albedo, 1.0);
}

// Output: Specular colour map
@fragment
fn fs_specular(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = textureSample(input_texture, input_sampler, input.tex_coords);

    // Compute metal mask
    let m = clamp(pixel.a - INSULATOR_SPEC_RANGE, 0.0, 1.0) * (1.0 / (1.0 - INSULATOR_SPEC_RANGE));

    // Base reflectance (capped at insulatorSpecRange)
    let r = min(pixel.a, INSULATOR_SPEC_RANGE);

    // Specular = base reflectance + metal-modulated colour
    // Clamp each channel: clamp(r + m * pixel, max: 1.0, min: 0.21)
    let specular = vec3<f32>(
        clamp(r + m * pixel.r, MIN_SPECULAR, 1.0),
        clamp(r + m * pixel.g, MIN_SPECULAR, 1.0),
        clamp(r + m * pixel.b, MIN_SPECULAR, 1.0)
    );

    return vec4<f32>(specular, 1.0);
}
