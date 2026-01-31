// Fragment shaders for PBR(MR) -> Colour/Spec/Gloss
// Note: Uses shared fullscreen quad vertex shader (vs_main)

struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

// Input textures (group 0)
@group(0) @binding(0)
var albedo_texture: texture_2d<f32>;
@group(0) @binding(1)
var albedo_sampler: sampler;

@group(0) @binding(2)
var metal_texture: texture_2d<f32>;
@group(0) @binding(3)
var metal_sampler: sampler;

@group(0) @binding(4)
var rough_texture: texture_2d<f32>;
@group(0) @binding(5)
var rough_sampler: sampler;

// Parameters (group 1)
struct Parameters {
    colour_mask: f32,
    spec_mask: f32,
};

@group(1) @binding(0)
var<uniform> params: Parameters;

fn saturate1(x: f32) -> f32 {
    return clamp(x, 0.0, 1.0);
}

fn saturate3(x: vec3<f32>) -> vec3<f32> {
    return clamp(x, vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fs_colour(input: VertexOutput) -> @location(0) vec4<f32> {
    let a = textureSample(albedo_texture, albedo_sampler, input.tex_coords);
    let m = saturate1(textureSample(metal_texture, metal_sampler, input.tex_coords).r);

    // Full conversion diffuse (simple, matches many legacy tools)
    let converted = a.rgb * (1.0 - m);

    // Slider behavior:
    // 0% => passthrough albedo
    // 100% => converted diffuse
    let t = saturate1(params.colour_mask);
    let out_rgb = mix(a.rgb, converted, vec3<f32>(t));

    return vec4<f32>(saturate3(out_rgb), a.a);
}

@fragment
fn fs_spec(input: VertexOutput) -> @location(0) vec4<f32> {
    let a = textureSample(albedo_texture, albedo_sampler, input.tex_coords);
    let m = saturate1(textureSample(metal_texture, metal_sampler, input.tex_coords).r);

    let f0 = vec3<f32>(0.15);

    // Full conversion spec:
    // metal=0 => 0.04
    // metal=1 => albedo
    let converted = mix(f0, a.rgb, vec3<f32>(m));

    // Slider behavior:
    let t = saturate1(params.spec_mask);
    let out_rgb = mix(f0, converted, vec3<f32>(t));

    return vec4<f32>(saturate3(out_rgb), 1.0);
}

@fragment
fn fs_gloss(input: VertexOutput) -> @location(0) vec4<f32> {
    let r = saturate1(textureSample(rough_texture, rough_sampler, input.tex_coords).r);
    let gloss = 1.0 - r;

    // Store in RGB for convenience
    return vec4<f32>(vec3<f32>(saturate1(gloss)), 1.0);
}
