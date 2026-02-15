// Fragment shader for CoD XY Normal Map
// Note: Uses shared fullscreen quad vertex shader (vs_main)

struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// Fragment shader for CoD XY Normal Map reconstruction
@group(0) @binding(0)
var input_texture: texture_2d<f32>;
@group(0) @binding(1)
var input_sampler: sampler;

// Reconstructs the Z component of a CoD DXT5 XY normal map
// Used in older Call of Duty games: MW/WaW/MW2/MW3/BO1
// DXT5 format stores X in alpha channel and Y in green channel
// This shader reconstructs Z using: Z = sqrt(1 - X² - Y²)
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = textureSample(input_texture, input_sampler, input.tex_coords);

    // 1. Apply the specific CoD constants
    // Note: pixel.a is pixel.W, pixel.g is pixel.Y
    let nX = pixel.a * 4.08 - 2.08;
    let nY = pixel.g * 4.064516 - 2.064516;

    // 2. Set Z to 1.0
    // Instead of calculating Z from X and Y, we assume a base height of 1.0
    let nZ = 1.0;

    // 3. Normalize the vector
    // This creates the final normal from the slope data
    let dist = sqrt(nX * nX + nY * nY + nZ * nZ);

    // Avoid division by zero
    let finalX = nX / dist;
    let finalY = nY / dist;
    let finalZ = nZ / dist;

    // 4. Pack back to [0,1] range for output )
    let finalZOutput = select(0.0, sqrt(finalZ) * 0.5 + 0.5, finalZ > 0.0);

    return vec4<f32>(
        finalX * 0.5 + 0.5,
        finalY * 0.5 + 0.5,
        finalZOutput,
        1.0
    );
}
