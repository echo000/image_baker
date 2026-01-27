// Fragment shader for Texture Baker
// Note: Uses shared fullscreen quad vertex shader (vs_main)

struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// Input textures (group 0)
@group(0) @binding(0)
var colour_texture: texture_2d<f32>;
@group(0) @binding(1)
var colour_sampler: sampler;

@group(0) @binding(2)
var specular_texture: texture_2d<f32>;
@group(0) @binding(3)
var specular_sampler: sampler;

@group(0) @binding(4)
var occlusion_texture: texture_2d<f32>;
@group(0) @binding(5)
var occlusion_sampler: sampler;

// Parameters uniform buffer (group 1)
struct Parameters {
    ao_contrast_power: f32,
    specular_intensity: f32,
}

@group(1) @binding(0)
var<uniform> params: Parameters;

// Apply power curve to AO value
fn apply_ao_contrast(value: f32, power: f32) -> f32 {
    return pow(clamp(value, 0.0, 1.0), power);
}

// Apply intensity scaling to specular value
fn apply_specular_intensity(value: f32, intensity: f32) -> f32 {
    return clamp(value * intensity, 0.0, 1.0);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample colour texture (main input - always present)
    let colour = textureSample(colour_texture, colour_sampler, input.tex_coords);

    // Sample optional textures
    let spec_sample = textureSample(specular_texture, specular_sampler, input.tex_coords);
    let specular = spec_sample.rgb;

    let occ_sample = textureSample(occlusion_texture, occlusion_sampler, input.tex_coords);
    let occlusion = occ_sample.rgb;

    // Detect if optional textures are present by checking if they're non-zero
    // (Default white texture will have all 1.0 values)
    let has_specular = specular.r < 0.99 || specular.g < 0.99 || specular.b < 0.99;
    let has_occlusion = occlusion.r < 0.99 || occlusion.g < 0.99 || occlusion.b < 0.99;

    var result: vec3<f32>;

    // Case 1: Only colour (no specular, no occlusion)
    if (!has_specular && !has_occlusion) {
        result = colour.rgb;
    }
    // Case 2: Colour + occlusion only (no specular)
    else if (!has_specular && has_occlusion) {
        // Apply AO contrast power curve
        let adjusted_occlusion = vec3<f32>(
            apply_ao_contrast(occlusion.r, params.ao_contrast_power),
            apply_ao_contrast(occlusion.g, params.ao_contrast_power),
            apply_ao_contrast(occlusion.b, params.ao_contrast_power)
        );

        // Apply occlusion to colour
        result = colour.rgb * adjusted_occlusion;
    }
    // Case 3: Colour + specular only (no occlusion)
    else if (has_specular && !has_occlusion) {
        // Apply specular intensity
        let adjusted_specular = vec3<f32>(
            apply_specular_intensity(specular.r, params.specular_intensity),
            apply_specular_intensity(specular.g, params.specular_intensity),
            apply_specular_intensity(specular.b, params.specular_intensity)
        );

        // Check if colour pixel is near-black (threshold of 38/255 ≈ 0.149)
        let is_black = colour.r < 0.149 && colour.g < 0.149 && colour.b < 0.149;

        // Blend based on whether pixel is near-black
        if (is_black) {
            // For black pixels: use specular
            result = adjusted_specular;
        } else {
            // For non-black pixels: use colour
            result = colour.rgb;
        }
    }
    // Case 4: All three (colour + specular + occlusion)
    else {
        // Apply specular intensity
        let adjusted_specular = vec3<f32>(
            apply_specular_intensity(specular.r, params.specular_intensity),
            apply_specular_intensity(specular.g, params.specular_intensity),
            apply_specular_intensity(specular.b, params.specular_intensity)
        );

        // Apply AO contrast power curve
        let adjusted_occlusion = vec3<f32>(
            apply_ao_contrast(occlusion.r, params.ao_contrast_power),
            apply_ao_contrast(occlusion.g, params.ao_contrast_power),
            apply_ao_contrast(occlusion.b, params.ao_contrast_power)
        );

        // Check if colour pixel is near-black (threshold of 38/255 ≈ 0.149)
        let is_black = colour.r < 0.149 && colour.g < 0.149 && colour.b < 0.149;

        // Blend based on whether pixel is near-black
        if (is_black) {
            // For black pixels: use specular * occlusion
            result = adjusted_specular * adjusted_occlusion;
        } else {
            // For non-black pixels: use colour * occlusion
            result = colour.rgb * adjusted_occlusion;
        }
    }

    // Preserve alpha from colour texture
    return vec4<f32>(result, colour.a);
}
