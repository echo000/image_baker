// Fragment shaders for CoD Fuse Color + Specular
// Note: Uses shared fullscreen quad vertex shader (vs_main)

struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@group(0) @binding(0)
var albedo_texture: texture_2d<f32>;
@group(0) @binding(1)
var albedo_sampler: sampler;

@group(0) @binding(2)
var specular_texture: texture_2d<f32>;
@group(0) @binding(3)
var specular_sampler: sampler;

// Combines separate albedo and specular maps into fused base colour with metalness
// Input format:
//   Albedo: RGB colour map
//   Specular: RGB specular values, Alpha = gloss
// Output format:
//   Base Colour: Fused albedo+specular in RGB, Metalness in Alpha
//   Gloss: Specular alpha channel as grayscale
// References:
//   https://www.activision.com/cdn/research/2017_DD_Rendering_of_COD_IW.pdf
//   GameImageUtil by Philip/Scobalula

const INSULATOR_SPEC_RANGE: f32 = 0.1;

// Output: Fused base colour with metalness
@fragment
fn fs_base_colour(input: VertexOutput) -> @location(0) vec4<f32> {
    let albedo = textureSample(albedo_texture, albedo_sampler, input.tex_coords);
    let specular = textureSample(specular_texture, specular_sampler, input.tex_coords);

    // Calculate metalness from albedo and specular
    // nonmetal = average of albedo RGB
    let nonmetal = clamp((albedo.r + albedo.g + albedo.b) / 3.0 + 0.000001, 0.0, 1.0);

    // metal = average of specular RGB
    let metal = (specular.r + specular.g + specular.b) / 3.0;

    // Split specular into electrical (metal) and insulator components
    let spec_e = clamp(metal - INSULATOR_SPEC_RANGE, 0.0, 1.0);
    let spec_i = min(metal, INSULATOR_SPEC_RANGE);

    // Calculate metalness
    var metalness = spec_e / (spec_e + nonmetal);
    metalness = spec_i + (1.0 - INSULATOR_SPEC_RANGE) * metalness;

    // Fuse albedo and specular by adding (specular - insulatorSpecRange) to albedo
    let fused = vec3<f32>(
        clamp(albedo.r + (specular.r - INSULATOR_SPEC_RANGE), 0.0, 1.0),
        clamp(albedo.g + (specular.g - INSULATOR_SPEC_RANGE), 0.0, 1.0),
        clamp(albedo.b + (specular.b - INSULATOR_SPEC_RANGE), 0.0, 1.0)
    );

    // Apply gamma correction to metalness (pow 2.2)
    let metalness_gamma = pow(metalness, 2.2);

    return vec4<f32>(fused, metalness_gamma);
}

// Output: Gloss map from specular alpha
@fragment
fn fs_gloss(input: VertexOutput) -> @location(0) vec4<f32> {
    let specular = textureSample(specular_texture, specular_sampler, input.tex_coords);

    // Extract gloss from specular alpha and output as grayscale
    return vec4<f32>(specular.a, specular.a, specular.a, 1.0);
}
