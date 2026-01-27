// Fragment shaders for CoD WW2 Colour/Specular Split
// Note: Uses shared fullscreen quad vertex shader (vs_main)

struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// Fragment shaders for CoD WW2 Split Colour/Specular
@group(0) @binding(0)
var fused_texture: texture_2d<f32>;
@group(0) @binding(1)
var fused_sampler: sampler;

@group(0) @binding(2)
var mask_texture: texture_2d<f32>;
@group(0) @binding(3)
var mask_sampler: sampler;

// Splits fused colour/specular from Call of Duty: World War II
// Input 1 (main): Fused diffuse and specular colours
// Input 2 (mask): Metalness mask texture
// References:
//   https://www.activision.com/cdn/research/2017_DD_Rendering_of_COD_IW.pdf

const INSULATOR_SPEC_RANGE: f32 = 0.1;
const MIN_SPECULAR: f32 = 0.21; // 56/255

// Output: Diffuse/Albedo colour map
@fragment
fn fs_albedo(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = textureSample(fused_texture, fused_sampler, input.tex_coords);
    let mask_pixel = textureSample(mask_texture, mask_sampler, input.tex_coords);

    // Compute metal mask: remap red channel values above insulatorSpecRange to [0,1]
    let m = clamp((mask_pixel.r - INSULATOR_SPEC_RANGE) / (1.0 - INSULATOR_SPEC_RANGE), 0.0, 1.0);

    // Compute diffuse mask (inverse of metal mask)
    let d = clamp(1.0 - m, 0.0, 1.0);

    // Diffuse colour = RGB * diffuse mask, preserve original alpha
    let albedo = vec3<f32>(pixel.r * d, pixel.g * d, pixel.b * d);

    return vec4<f32>(albedo, pixel.a);
}

// Output: Specular colour map
@fragment
fn fs_specular(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = textureSample(fused_texture, fused_sampler, input.tex_coords);
    let mask_pixel = textureSample(mask_texture, mask_sampler, input.tex_coords);

    // Compute metal mask from red channel
    let m = clamp((mask_pixel.r - INSULATOR_SPEC_RANGE) / (1.0 - INSULATOR_SPEC_RANGE), 0.0, 1.0);

    // Base reflectance from mask alpha (capped at insulatorSpecRange)
    let r = min(mask_pixel.a, INSULATOR_SPEC_RANGE);

    // Specular = base reflectance + metal-modulated colour, with minimum of 0.21 per channel
    let specular = vec3<f32>(
        max(r + m * pixel.r, MIN_SPECULAR),
        max(r + m * pixel.g, MIN_SPECULAR),
        max(r + m * pixel.b, MIN_SPECULAR)
    );

    return vec4<f32>(specular, 1.0);
}
