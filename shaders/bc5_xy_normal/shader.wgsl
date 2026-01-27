// Fragment shader for BC5 XY Normal Map
// Note: Uses shared fullscreen quad vertex shader (vs_main)

struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// Fragment shader for BC5 XY Normal Map reconstruction
@group(0) @binding(0)
var input_texture: texture_2d<f32>;
@group(0) @binding(1)
var input_sampler: sampler;

// Reconstructs the Z component of a BC5 XY normal map
// BC5 format stores X in red channel and Y in green channel
// This shader reconstructs Z using: Z = sqrt(1 - X² - Y²)
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = textureSample(input_texture, input_sampler, input.tex_coords);

    // Convert from [0,1] to [-1,1] range
    let nX = pixel.r * 2.0 - 1.0;
    let nY = pixel.g * 2.0 - 1.0;

    // Calculate Z component: Z = sqrt(1 - X² - Y²)
    let nZ_squared = 1.0 - (nX * nX) - (nY * nY);

    // Only compute sqrt if the value is positive, otherwise clamp to 0
    let nZ = select(0.0, sqrt(nZ_squared), nZ_squared > 0.0);

    // Convert back to [0,1] range for output
    let outputX = nX * 0.5 + 0.5;
    let outputY = nY * 0.5 + 0.5;
    let outputZ = nZ * 0.5 + 0.5;

    return vec4<f32>(outputX, outputY, outputZ, 1.0);
}
