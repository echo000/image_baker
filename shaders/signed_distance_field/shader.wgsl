// Fragment shader for Signed Distance Field processing
// Note: Uses shared fullscreen quad vertex shader (vs_main)

struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// Fragment shader for Signed Distance Field processing
@group(0) @binding(0)
var input_texture: texture_2d<f32>;
@group(0) @binding(1)
var input_sampler: sampler;

// Converts distance field data to smooth alpha with anti-aliased edges
// Used in Call of Duty: IW8+ (Modern Warfare 2019 and later)
// Commonly used for text rendering or vector-like effects

// Smoothing factor for edge anti-aliasing
const SMOOTHING: f32 = 0.015625; // 1/64

// Smoothstep interpolation function
// Provides smooth hermite interpolation between edge0 and edge1
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    // Scale, bias and saturate x to 0..1 range
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    // Evaluate polynomial: 3t² - 2t³
    return t * t * (3.0 - 2.0 * t);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = textureSample(input_texture, input_sampler, input.tex_coords);

    // Read distance field value from red channel
    let distance_value = pixel.r;

    // Apply smoothstep to create anti-aliased edges
    // The smoothstep transitions smoothly from 0 to 1 around the 0.5 threshold
    let alpha = smoothstep(0.5 - SMOOTHING, 0.5 + SMOOTHING, distance_value);

    // Clamp to ensure valid range (smoothstep should already do this, but be safe)
    let clamped_alpha = clamp(alpha, 0.0, 1.0);

    // Output as grayscale alpha map
    return vec4<f32>(clamped_alpha, clamped_alpha, clamped_alpha, 1.0);
}
