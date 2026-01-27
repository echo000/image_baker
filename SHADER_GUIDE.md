# Shader System Guide

This guide explains how to create custom shaders for the Texture Converter.

## Overview

The Texture Converter uses WGSL (WebGPU Shading Language) shaders to process textures. Each shader is defined by two files:

- `config.toml` - Shader metadata and configuration
- `shader.wgsl` - WGSL fragment shader code

**Important:** As of the latest update, all texture processing shaders use a **shared vertex shader** that is automatically provided by the system. Individual shader files only need to contain fragment shader code and the `VertexOutput` struct definition. This eliminates code duplication and improves performance.

## Directory Structure

Shaders are organized in the `shaders/` directory:

```
shaders/
├── my_shader/
│   ├── config.toml
│   └── shader.wgsl
├── another_shader/
│   ├── config.toml
│   └── shader.wgsl
```

Each shader must be in its own subdirectory containing both required files.

## Configuration File (config.toml)

### Basic Single-Input Shader

```toml
[shader]
name = "My Shader Name"
description = "Brief description of what this shader does"
author = "Your Name"
version = "1.0.0"

[[outputs]]
entry_point = "fs_main"
suffix = "_output"
description = "Description of this output"
format = "Rgba8Unorm"
```

### Multi-Input Shader

For shaders that need additional textures (e.g., merging separate RGB and alpha):

```toml
[shader]
name = "Merge RGB/Alpha"
description = "Combines separate RGB and alpha textures"
author = "Your Name"
version = "1.0.0"

[[inputs]]
suffix = "_a"
description = "Alpha texture"
required = true

[[outputs]]
entry_point = "fs_main"
suffix = "_merged"
description = "Merged RGBA texture"
format = "Rgba8Unorm"
```

### Multiple Outputs

Shaders can produce multiple output textures:

```toml
[shader]
name = "Channel Splitter"
description = "Splits RGBA into separate channels"
author = "Your Name"
version = "1.0.0"

[[outputs]]
entry_point = "fs_red"
suffix = "_r"
description = "Red channel"
format = "Rgba8Unorm"

[[outputs]]
entry_point = "fs_green"
suffix = "_g"
description = "Green channel"
format = "Rgba8Unorm"

[[outputs]]
entry_point = "fs_blue"
suffix = "_b"
description = "Blue channel"
format = "Rgba8Unorm"
```

### Configuration Fields

#### [shader] Section

- `name` (required) - Display name shown in the UI
- `description` (required) - Brief explanation of functionality
- `author` (optional) - Shader creator
- `version` (optional) - Version string

#### [[inputs]] Section (optional, repeatable)

- `suffix` (required) - Filename suffix to search for (e.g., "_a" for "texture_a.png")
- `description` (required) - Description of this input
- `required` (optional, default: true) - Whether this input must exist

#### [[outputs]] Section (required, repeatable)

- `entry_point` (required) - Fragment shader function name (see "Shader File" section below for details)
- `suffix` (required) - Output filename suffix (e.g., "_normal" produces "texture_normal.png")
- `description` (required) - Description of this output
- `format` (optional, default: "Rgba8Unorm") - Texture format

## Shader File (shader.wgsl)

### Basic Structure

Every shader file needs:
1. The `VertexOutput` struct definition (required for fragment shaders to receive vertex data)
2. One or more fragment shader functions (where you implement your custom logic)

**Important:** Do NOT include a vertex shader (`@vertex fn vs_main()`) in your shader file. The system automatically provides a shared vertex shader that generates a fullscreen quad. Including your own vertex shader will cause compilation errors.

```wgsl
// Fragment shader for [Your Shader Name]
// Note: Uses shared fullscreen quad vertex shader (vs_main)

struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@group(0) @binding(0)
var input_texture: texture_2d<f32>;
@group(0) @binding(1)
var input_sampler: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = textureSample(input_texture, input_sampler, input.tex_coords);
    // Process pixel here
    return pixel;
}
```

### Shared Vertex Shader

The system provides a shared vertex shader (`src/shaders/fullscreen_quad_vertex.wgsl`) that:
- Generates a fullscreen quad using only the vertex index (no vertex buffers needed)
- Covers the entire screen from (-1, -1) to (1, 1) in NDC space
- Provides UV coordinates from (0, 0) at top-left to (1, 1) at bottom-right
- Is compiled once and reused for all shader pipelines (better performance)

You only need to focus on writing fragment shader code that processes pixels.

### Texture Bindings

#### Single Input (Main Texture)

```wgsl
@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
```

#### Multiple Inputs

Additional inputs use sequential bindings:

```wgsl
// Main input
@group(0) @binding(0) var rgb_texture: texture_2d<f32>;
@group(0) @binding(1) var rgb_sampler: sampler;

// First additional input
@group(0) @binding(2) var alpha_texture: texture_2d<f32>;
@group(0) @binding(3) var alpha_sampler: sampler;

// Second additional input (if needed)
@group(0) @binding(4) var mask_texture: texture_2d<f32>;
@group(0) @binding(5) var mask_sampler: sampler;
```

Pattern: Each texture requires two bindings (texture + sampler) at consecutive binding points.

## Examples

### Example 1: Simple Inversion

Inverts RGB values while preserving alpha.

**config.toml:**
```toml
[shader]
name = "RGB Invert"
description = "Inverts RGB colour values"
author = "Example"
version = "1.0.0"

[[outputs]]
entry_point = "fs_main"
suffix = "_inverted"
description = "Inverted RGB image"
format = "Rgba8Unorm"
```

**shader.wgsl:**
```wgsl
// Fragment shader for RGB inversion
// Note: Uses shared fullscreen quad vertex shader (vs_main)

struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@group(0) @binding(0)
var input_texture: texture_2d<f32>;
@group(0) @binding(1)
var input_sampler: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = textureSample(input_texture, input_sampler, input.tex_coords);

    // Invert RGB, preserve alpha
    let inverted = vec3<f32>(
        1.0 - pixel.r,
        1.0 - pixel.g,
        1.0 - pixel.b
    );

    return vec4<f32>(inverted, pixel.a);
}
```

### Example 2: Channel Extraction

Extracts individual colour channels as grayscale images.

**config.toml:**
```toml
[shader]
name = "Extract Channels"
description = "Splits RGBA into separate grayscale images"
author = "Example"
version = "1.0.0"

[[outputs]]
entry_point = "fs_red"
suffix = "_r"
description = "Red channel"
format = "Rgba8Unorm"

[[outputs]]
entry_point = "fs_green"
suffix = "_g"
description = "Green channel"
format = "Rgba8Unorm"

[[outputs]]
entry_point = "fs_blue"
suffix = "_b"
description = "Blue channel"
format = "Rgba8Unorm"
```

**shader.wgsl:**
```wgsl
// Fragment shaders for channel extraction
// Note: Uses shared fullscreen quad vertex shader (vs_main)

struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@group(0) @binding(0)
var input_texture: texture_2d<f32>;
@group(0) @binding(1)
var input_sampler: sampler;

@fragment
fn fs_red(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = textureSample(input_texture, input_sampler, input.tex_coords);
    return vec4<f32>(pixel.r, pixel.r, pixel.r, 1.0);
}

@fragment
fn fs_green(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = textureSample(input_texture, input_sampler, input.tex_coords);
    return vec4<f32>(pixel.g, pixel.g, pixel.g, 1.0);
}

@fragment
fn fs_blue(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = textureSample(input_texture, input_sampler, input.tex_coords);
    return vec4<f32>(pixel.b, pixel.b, pixel.b, 1.0);
}
```

### Example 3: Normal Map Reconstruction

Reconstructs Z component from XY normal map data.

**config.toml:**
```toml
[shader]
name = "XY Normal Reconstruct"
description = "Reconstructs Z component from XY normal map"
author = "Example"
version = "1.0.0"

[[outputs]]
entry_point = "fs_main"
suffix = "_normal"
description = "Full RGB normal map"
format = "Rgba8Unorm"
```

**shader.wgsl:**
```wgsl
// Fragment shader for XY normal map reconstruction
// Note: Uses shared fullscreen quad vertex shader (vs_main)

struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@group(0) @binding(0)
var input_texture: texture_2d<f32>;
@group(0) @binding(1)
var input_sampler: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = textureSample(input_texture, input_sampler, input.tex_coords);

    // Convert from [0,1] to [-1,1] range
    let nX = pixel.r * 2.0 - 1.0;
    let nY = pixel.g * 2.0 - 1.0;

    // Calculate Z: Z = sqrt(1 - X^2 - Y^2)
    let nZ_squared = 1.0 - (nX * nX) - (nY * nY);
    let nZ = select(0.0, sqrt(nZ_squared), nZ_squared > 0.0);

    // Convert back to [0,1] range
    let outputX = nX * 0.5 + 0.5;
    let outputY = nY * 0.5 + 0.5;
    let outputZ = nZ * 0.5 + 0.5;

    return vec4<f32>(outputX, outputY, outputZ, 1.0);
}
```

### Example 4: Multi-Input Merge

Merges separate RGB and alpha textures.

**config.toml:**
```toml
[shader]
name = "Merge RGB + Alpha"
description = "Combines separate RGB and alpha textures"
author = "Example"
version = "1.0.0"

[[inputs]]
suffix = "_a"
description = "Alpha texture"
required = true

[[outputs]]
entry_point = "fs_main"
suffix = "_merged"
description = "Merged RGBA texture"
format = "Rgba8Unorm"
```

**shader.wgsl:**
```wgsl
// Fragment shader for merging RGB and alpha textures
// Note: Uses shared fullscreen quad vertex shader (vs_main)

struct VertexOutput {
    @builtin(position) vert_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@group(0) @binding(0)
var rgb_texture: texture_2d<f32>;
@group(0) @binding(1)
var rgb_sampler: sampler;

@group(0) @binding(2)
var alpha_texture: texture_2d<f32>;
@group(0) @binding(3)
var alpha_sampler: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let rgb = textureSample(rgb_texture, rgb_sampler, input.tex_coords);
    let alpha = textureSample(alpha_texture, alpha_sampler, input.tex_coords);

    // Use red channel from alpha texture as alpha value
    return vec4<f32>(rgb.r, rgb.g, rgb.b, alpha.r);
}
```

## WGSL Reference

### Common Functions

- `textureSample(texture, sampler, coords)` - Sample a texture at given coordinates
- `clamp(value, min, max)` - Clamp value to range
- `min(a, b)` - Minimum of two values
- `max(a, b)` - Maximum of two values
- `abs(value)` - Absolute value
- `sqrt(value)` - Square root
- `normalize(vector)` - Normalize a vector
- `select(false_value, true_value, condition)` - Conditional selection

### Vector Construction

```wgsl
let v2 = vec2<f32>(1.0, 2.0);
let v3 = vec3<f32>(1.0, 2.0, 3.0);
let v4 = vec4<f32>(1.0, 2.0, 3.0, 4.0);

// Swizzling
let rgb = pixel.rgb;
let rg = pixel.rg;
let alpha = pixel.a;
```

### Colour Space Conversions

```wgsl
// [0,1] to [-1,1]
let value = pixel.r * 2.0 - 1.0;

// [-1,1] to [0,1]
let value = pixel.r * 0.5 + 0.5;
```

## Testing Your Shader

1. Create your shader directory in `shaders/`
2. Add `config.toml` and `shader.wgsl`
3. Launch the application
4. Switch to the Texture Converter tab (shaders reload automatically)
5. Select your shader from the dropdown
6. Drop a test texture to process

Check the log output for any errors during shader loading or processing.

## Common Patterns

### Grayscale Conversion

```wgsl
let gray = pixel.r * 0.299 + pixel.g * 0.587 + pixel.b * 0.114;
return vec4<f32>(gray, gray, gray, 1.0);
```

### Alpha Testing

```wgsl
if pixel.a < 0.5 {
    // Discard or process transparent pixels differently
}
```

### Channel Swizzling

```wgsl
// Swap red and blue
return vec4<f32>(pixel.b, pixel.g, pixel.r, pixel.a);

// Copy alpha to RGB
return vec4<f32>(pixel.a, pixel.a, pixel.a, 1.0);
```

### Smoothstep Interpolation

```wgsl
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}
```

## Troubleshooting

### Shader Not Appearing

- Verify directory structure (shader must be in its own folder)
- Check both `config.toml` and `shader.wgsl` exist
- Ensure TOML syntax is valid
- Check application logs for parsing errors

### Black Output

- Verify you're returning valid colour values (0.0 to 1.0 range)
- Check texture sampling coordinates are valid
- Ensure fragment shader returns `vec4<f32>`

### Binding Errors

- Verify binding numbers match your config
- Main texture: bindings 0 and 1
- Each additional input: sequential pairs (2/3, 4/5, 6/7, etc.)
- Each input needs both texture and sampler bindings

### Missing Additional Inputs

- Check filename suffix matches config exactly
- Ensure files are in the same directory
- Verify `required` field in config if input is optional
- Check file extensions match

## Best Practices

1. **Document your shader** - Add comments explaining the algorithm
2. **Use descriptive names** - Make binding variables and functions clear
3. **Test edge cases** - Try with various texture sizes and formats
4. **Keep it simple** - Complex operations may impact performance

## Performance Considerations

- GPU operations are parallel by nature
- Each pixel is processed independently
- Avoid complex branching when possible
- Texture sampling is relatively fast
- Mathematical operations are highly optimized

## Further Resources

- [WGSL Specification](https://www.w3.org/TR/WGSL/)
- [WebGPU Documentation](https://gpuweb.github.io/gpuweb/)
- Study existing shaders in the `shaders/` directory for reference
- Test with the provided example shaders first
- See `SHADER_OPTIMIZATION.md` for technical details about the shared vertex shader system
