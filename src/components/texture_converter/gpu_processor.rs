//! GPU Processor Module
//!
//! Handles GPU-accelerated shader execution and image processing.
//!
//! This module provides functionality to:
//! - Execute WGSL shaders on input textures
//! - Manage GPU resources (textures, buffers, pipelines)
//! - Handle multiple shader outputs
//! - Read back processed results from GPU to CPU

use crate::components::texture_converter::{FULLSCREEN_QUAD_VERTEX_SHADER, ShaderConfig};
use crate::porter_image::{ImageBuffer, PorterImage};
use std::collections::HashMap;
use std::sync::Arc;

/// Result type for GPU operations
pub type GpuResult<T> = Result<T, String>;

/// Process images using the specified shader configuration
///
/// # Arguments
/// * `images` - Input images to process
/// * `shader_config` - Shader configuration with inputs/outputs/parameters
/// * `parameter_values` - User-defined parameter values for the shader
///
/// # Returns
/// * `Ok(Vec<(ImageBuffer, String)>)` - Processed outputs with descriptions
/// * `Err(String)` - Error message if processing fails
pub async fn process_images(
    images: Vec<Arc<PorterImage>>,
    shader_config: ShaderConfig,
    parameter_values: HashMap<String, f32>,
) -> GpuResult<Vec<(ImageBuffer, String)>> {
    if images.is_empty() {
        return Err("No images provided".to_string());
    }

    // Load shader code
    let fragment_shader_code = load_shader_code(&shader_config.shader_path)?;

    // Get GPU resources
    let gpu = porter_gpu::gpu_instance();
    let device = gpu.device();
    let queue = gpu.queue();

    // Create shader modules
    let vertex_shader = create_vertex_shader(device);
    let fragment_shader =
        create_fragment_shader(device, &fragment_shader_code, &shader_config.shader.name)?;

    // Get dimensions from first image
    let (width, height) = images[0].dimensions();
    let texture_size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    // Create input textures and resources
    let (_input_textures, input_views, input_samplers) =
        create_input_textures(device, queue, &images, &shader_config, texture_size)?;

    // Create bind group for textures
    let (texture_bind_group_layout, texture_bind_group) =
        create_texture_bind_group(device, &shader_config, &input_views, &input_samplers)?;

    // Create parameters bind group if needed
    let (params_bind_group_layout, params_bind_group) =
        create_parameters_bind_group(device, queue, &shader_config, &parameter_values)?;

    // Create pipeline layout
    let pipeline_layout = create_pipeline_layout(
        device,
        &texture_bind_group_layout,
        params_bind_group_layout.as_ref(),
    )?;

    // Process each output
    let output_buffers = process_all_outputs(
        device,
        queue,
        &vertex_shader,
        &fragment_shader,
        &pipeline_layout,
        &shader_config,
        texture_size,
        &texture_bind_group,
        params_bind_group.as_ref(),
    )?;

    Ok(output_buffers)
}

/// Load shader WGSL code from file
///
/// Reads the shader source code from disk.
fn load_shader_code(shader_path: &std::path::Path) -> GpuResult<String> {
    std::fs::read_to_string(shader_path)
        .map_err(|e| format!("Failed to load shader from {shader_path:?}: {e}"))
}

/// Create vertex shader module from embedded source
///
/// Uses the fullscreen quad vertex shader for all processing operations.
fn create_vertex_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Fullscreen Quad Vertex Shader"),
        source: wgpu::ShaderSource::Wgsl(
            std::str::from_utf8(FULLSCREEN_QUAD_VERTEX_SHADER)
                .expect("Invalid UTF-8 in vertex shader")
                .into(),
        ),
    })
}

/// Create fragment shader module from loaded WGSL code
///
/// Compiles the user-provided fragment shader.
fn create_fragment_shader(
    device: &wgpu::Device,
    code: &str,
    name: &str,
) -> GpuResult<wgpu::ShaderModule> {
    Ok(device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(&format!("{name} Fragment Shader")),
        source: wgpu::ShaderSource::Wgsl(code.into()),
    }))
}

/// Create input textures for all shader inputs
///
/// Creates GPU textures for loaded images and white placeholders
/// for optional inputs that weren't provided.
fn create_input_textures(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    images: &[Arc<PorterImage>],
    shader_config: &ShaderConfig,
    texture_size: wgpu::Extent3d,
) -> GpuResult<(
    Vec<wgpu::Texture>,
    Vec<wgpu::TextureView>,
    Vec<wgpu::Sampler>,
)> {
    let mut input_textures = Vec::new();
    let mut input_views = Vec::new();
    let mut input_samplers = Vec::new();

    // Create textures for all defined inputs (including placeholders for optional ones)
    for (idx, _input_config) in shader_config.inputs.iter().enumerate() {
        let texture = if idx < images.len() {
            create_image_texture(device, queue, &images[idx], idx)?
        } else {
            create_placeholder_texture(device, queue, texture_size, idx)?
        };

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = create_sampler(device, idx);

        input_textures.push(texture);
        input_views.push(view);
        input_samplers.push(sampler);
    }

    Ok((input_textures, input_views, input_samplers))
}

/// Create a GPU texture from an image
///
/// Converts a PorterImage to RGBA8 format and uploads to GPU.
fn create_image_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    img: &Arc<PorterImage>,
    idx: usize,
) -> GpuResult<wgpu::Texture> {
    let (img_width, img_height) = img.dimensions();
    let img_texture_size = wgpu::Extent3d {
        width: img_width,
        height: img_height,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(&format!("Input Texture {idx}")),
        size: img_texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    // Convert and upload image data
    let mut img_mut = (**img).clone();
    if let Err(e) = img_mut.convert_to_rgba8() {
        return Err(format!("Failed to convert image to RGBA8: {e}"));
    }
    let data = img_mut
        .raw_buffer()
        .map_err(|e| format!("Failed to get raw buffer: {e}"))?;
    let rgba_data: Vec<u8> = data.to_vec();

    queue.write_texture(
        texture.as_image_copy(),
        &rgba_data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * img_width),
            rows_per_image: Some(img_height),
        },
        img_texture_size,
    );

    Ok(texture)
}

/// Create a white placeholder texture for optional inputs
///
/// Used when an optional shader input wasn't provided by the user.
fn create_placeholder_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_size: wgpu::Extent3d,
    idx: usize,
) -> GpuResult<wgpu::Texture> {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(&format!("Input Texture {idx} (Placeholder)")),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    let white_data = vec![255u8; (texture_size.width * texture_size.height * 4) as usize];
    queue.write_texture(
        texture.as_image_copy(),
        &white_data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * texture_size.width),
            rows_per_image: Some(texture_size.height),
        },
        texture_size,
    );

    Ok(texture)
}

/// Create a sampler for texture filtering
///
/// Configures linear filtering and clamp-to-edge addressing.
fn create_sampler(device: &wgpu::Device, idx: usize) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(&format!("Sampler {idx}")),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}

/// Create bind group layout and bind group for textures
///
/// Sets up GPU bindings for all input textures and their samplers.
fn create_texture_bind_group(
    device: &wgpu::Device,
    shader_config: &ShaderConfig,
    input_views: &[wgpu::TextureView],
    input_samplers: &[wgpu::Sampler],
) -> GpuResult<(wgpu::BindGroupLayout, wgpu::BindGroup)> {
    // Create bind group layout for all shader inputs
    let mut layout_entries = Vec::new();
    for i in 0..shader_config.inputs.len() {
        let base_binding = (i * 2) as u32;
        layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: base_binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        });
        layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: base_binding + 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        });
    }

    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &layout_entries,
        });

    // Create bind group entries for textures
    let mut bind_group_entries = Vec::new();
    for (i, (view, sampler)) in input_views.iter().zip(input_samplers.iter()).enumerate() {
        let base_binding = (i * 2) as u32;
        bind_group_entries.push(wgpu::BindGroupEntry {
            binding: base_binding,
            resource: wgpu::BindingResource::TextureView(view),
        });
        bind_group_entries.push(wgpu::BindGroupEntry {
            binding: base_binding + 1,
            resource: wgpu::BindingResource::Sampler(sampler),
        });
    }

    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Texture Bind Group"),
        layout: &texture_bind_group_layout,
        entries: &bind_group_entries,
    });

    Ok((texture_bind_group_layout, texture_bind_group))
}

/// Create bind group for shader parameters if needed
///
/// Creates a uniform buffer containing all parameter values.
/// Returns None if shader has no parameters.
fn create_parameters_bind_group(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    shader_config: &ShaderConfig,
    parameter_values: &HashMap<String, f32>,
) -> GpuResult<(Option<wgpu::BindGroupLayout>, Option<wgpu::BindGroup>)> {
    if shader_config.parameters.is_empty() {
        return Ok((None, None));
    }

    // Build uniform data from parameters
    let mut uniform_data = Vec::new();
    for param in &shader_config.parameters {
        let value = parameter_values
            .get(&param.name)
            .copied()
            .unwrap_or(param.default);
        uniform_data.extend_from_slice(&value.to_le_bytes());
    }

    // Pad to 16-byte alignment
    while uniform_data.len() % 16 != 0 {
        uniform_data.push(0);
    }

    let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Parameters Uniform Buffer"),
        size: uniform_data.len() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    queue.write_buffer(&params_buffer, 0, &uniform_data);

    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Parameters Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Parameters Bind Group"),
        layout: &layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: params_buffer.as_entire_binding(),
        }],
    });

    Ok((Some(layout), Some(bind_group)))
}

/// Create pipeline layout
///
/// Combines texture and parameter bind group layouts.
fn create_pipeline_layout(
    device: &wgpu::Device,
    texture_layout: &wgpu::BindGroupLayout,
    params_layout: Option<&wgpu::BindGroupLayout>,
) -> GpuResult<wgpu::PipelineLayout> {
    let bind_group_layouts: Vec<&wgpu::BindGroupLayout> = if let Some(params) = params_layout {
        vec![texture_layout, params]
    } else {
        vec![texture_layout]
    };

    Ok(
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &bind_group_layouts,
            push_constant_ranges: &[],
        }),
    )
}

/// Process all outputs defined in the shader configuration
#[allow(clippy::too_many_arguments)]
fn process_all_outputs(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    vertex_shader: &wgpu::ShaderModule,
    fragment_shader: &wgpu::ShaderModule,
    pipeline_layout: &wgpu::PipelineLayout,
    shader_config: &ShaderConfig,
    texture_size: wgpu::Extent3d,
    texture_bind_group: &wgpu::BindGroup,
    params_bind_group: Option<&wgpu::BindGroup>,
) -> GpuResult<Vec<(ImageBuffer, String)>> {
    let mut output_buffers = Vec::new();

    for output_config in &shader_config.outputs {
        let output_buffer = process_single_output(
            device,
            queue,
            vertex_shader,
            fragment_shader,
            pipeline_layout,
            output_config,
            texture_size,
            texture_bind_group,
            params_bind_group,
        )?;

        output_buffers.push((output_buffer, output_config.description.clone()));
    }

    Ok(output_buffers)
}

/// Process a single output
#[allow(clippy::too_many_arguments)]
fn process_single_output(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    vertex_shader: &wgpu::ShaderModule,
    fragment_shader: &wgpu::ShaderModule,
    pipeline_layout: &wgpu::PipelineLayout,
    output_config: &crate::components::texture_converter::OutputConfig,
    texture_size: wgpu::Extent3d,
    texture_bind_group: &wgpu::BindGroup,
    params_bind_group: Option<&wgpu::BindGroup>,
) -> GpuResult<ImageBuffer> {
    // Create render pipeline for this output
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&format!("Render Pipeline - {}", output_config.description)),
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: vertex_shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: fragment_shader,
            entry_point: Some(&output_config.entry_point),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    // Create output texture
    let output_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(&format!("Output Texture - {}", output_config.description)),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

    // Render to texture
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some(&format!("Render Encoder - {}", output_config.description)),
    });

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(&format!("Render Pass - {}", output_config.description)),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&pipeline);
        render_pass.set_bind_group(0, texture_bind_group, &[]);
        if let Some(params_bg) = params_bind_group {
            render_pass.set_bind_group(1, params_bg, &[]);
        }
        render_pass.draw(0..6, 0..1);
    }

    // Copy texture to buffer for readback
    let image_buffer =
        copy_texture_to_buffer(device, queue, encoder, &output_texture, texture_size)?;

    Ok(image_buffer)
}

/// Copy texture to buffer and read back to CPU
///
/// Copies GPU texture data to a staging buffer and reads it back.
/// Handles row padding required by GPU buffer alignment.
fn copy_texture_to_buffer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mut encoder: wgpu::CommandEncoder,
    texture: &wgpu::Texture,
    texture_size: wgpu::Extent3d,
) -> GpuResult<ImageBuffer> {
    let width = texture_size.width;
    let height = texture_size.height;
    let bytes_per_row = 4 * width;
    let padded_bytes_per_row = {
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        bytes_per_row.div_ceil(align) * align
    };

    let buffer_size = (padded_bytes_per_row * height) as wgpu::BufferAddress;
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Output Buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &output_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        texture_size,
    );

    let submission = queue.submit(std::iter::once(encoder.finish()));

    // Read back the data
    let buffer_slice = output_buffer.slice(..);
    let (sender, receiver) = futures::channel::oneshot::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).unwrap();
    });

    let _ = device.poll(wgpu::PollType::WaitForSubmissionIndex(submission));

    match pollster::block_on(receiver) {
        Ok(Ok(())) => {}
        Ok(Err(e)) => return Err(format!("Buffer mapping failed: {e}")),
        Err(e) => return Err(format!("Channel error: {e}")),
    }

    let data = buffer_slice.get_mapped_range();
    let mut rgba_data = vec![0u8; (width * height * 4) as usize];

    // Remove padding from rows
    for y in 0..height {
        let src_offset = (y * padded_bytes_per_row) as usize;
        let dst_offset = (y * bytes_per_row) as usize;
        let row_bytes = bytes_per_row as usize;
        rgba_data[dst_offset..dst_offset + row_bytes]
            .copy_from_slice(&data[src_offset..src_offset + row_bytes]);
    }

    drop(data);
    output_buffer.unmap();

    ImageBuffer::from_raw(width, height, rgba_data)
        .ok_or_else(|| "Failed to create ImageBuffer".to_string())
}
