// Fragment shader for format conversion (passthrough)
// Note: Uses shared fullscreen quad vertex shader (vs_main)

struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@group(0) @binding(0)
var input_texture: texture_2d<f32>;
@group(0) @binding(1)
var input_sampler: sampler;

// Simple passthrough shader that returns the input texture as-is
// Useful for converting between image formats (e.g., PNG to TGA)

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample and return the input texture directly - no processing
    return textureSample(input_texture, input_sampler, input.tex_coords);
}
