// Fragment shader for Detail Map Merger
// Note: Uses shared fullscreen quad vertex shader (vs_main)

struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// Input textures (group 0)
@group(0) @binding(0)
var base_texture: texture_2d<f32>;
@group(0) @binding(1)
var base_sampler: sampler;

@group(0) @binding(2)
var detail_texture: texture_2d<f32>;
@group(0) @binding(3)
var detail_sampler: sampler;

// Parameters uniform buffer (group 1)
struct Parameters {
    detail_intensity: f32,
}

@group(1) @binding(0)
var<uniform> params: Parameters;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample base colour texture
    let base = textureSample(base_texture, base_sampler, input.tex_coords);

    // Sample detail texture
    let detail = textureSample(detail_texture, detail_sampler, input.tex_coords);

    // Get the mask value from the base alpha channel
    let mask_value = base.a;

    // If the mask is 0, the blend does nothing
    if (mask_value == 0.0) {
        return base;
    }

    // Pre-calculate the base strength (from the "Opacity" parameter)
    // This is doubled to match the original implementation (2.0 * detail_intensity)
    let base_strength = 2.0 * params.detail_intensity;

    // Calculate the final strength, scaled by the mask
    let final_strength = base_strength * mask_value;

    // Apply the LINEAR LIGHT formula, per-channel
    // Result = Base + (Detail - 0.5) * (2.0 * Intensity * Mask)
    let modified_r = base.r + (detail.r - 0.5) * final_strength;
    let modified_g = base.g + (detail.g - 0.5) * final_strength;
    let modified_b = base.b + (detail.b - 0.5) * final_strength;

    // Clamp values to valid range
    let result = vec3<f32>(
        clamp(modified_r, 0.0, 1.0),
        clamp(modified_g, 0.0, 1.0),
        clamp(modified_b, 0.0, 1.0)
    );

    // Preserve alpha from base texture
    return vec4<f32>(result, base.a);
}
