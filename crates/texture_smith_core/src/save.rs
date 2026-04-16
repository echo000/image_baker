//! Save Module
//!
//! Provides image saving functionality for various formats including
//! PNG, TGA, TIFF, and DDS (with BC compression support).

use crate::porter_image::ImageBuffer;
use crate::types::{DdsSaveFormat, ImageFormat};
use std::path::Path;

/// Save a single output buffer to a file.
///
/// Supports PNG, TGA, TIFF, and DDS formats.
/// For DDS, supports both BC-compressed and uncompressed pixel formats.
pub fn save_buffer(
    buffer: &ImageBuffer,
    file_path: &Path,
    format: ImageFormat,
    dds_format: DdsSaveFormat,
) -> Result<(), String> {
    std::fs::create_dir_all(file_path.parent().unwrap_or(file_path))
        .map_err(|e| format!("Failed to create directory: {e}"))?;

    let filename = file_path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "output".to_string());

    if format == ImageFormat::Dds && dds_format.is_bc_compressed() {
        save_bc_compressed(buffer, file_path, dds_format, &filename)
    } else if format == ImageFormat::Dds {
        save_uncompressed_dds(buffer, file_path, dds_format, &filename)
    } else {
        save_standard_format(buffer, file_path, &filename)
    }
}

/// Save multiple output buffers to a directory.
///
/// Each output is saved with its description as the filename.
pub fn save_outputs(
    outputs: &[(ImageBuffer, String)],
    folder_path: &Path,
    format: ImageFormat,
    dds_format: DdsSaveFormat,
) -> Result<Vec<std::path::PathBuf>, String> {
    let mut saved_paths = Vec::new();

    for (buffer, description) in outputs {
        let filename = format!(
            "{}.{}",
            description.to_lowercase().replace([' ', '/', '\\'], "_"),
            format.extension()
        );

        let file_path = folder_path.join(&filename);
        save_buffer(buffer, &file_path, format, dds_format)?;
        saved_paths.push(file_path);
    }

    Ok(saved_paths)
}

fn save_bc_compressed(
    buffer: &ImageBuffer,
    file_path: &Path,
    dds_format: DdsSaveFormat,
    filename: &str,
) -> Result<(), String> {
    let (width, height) = buffer.dimensions();
    let rgba_data = buffer.as_raw();

    let surface = intel_tex_2::RgbaSurface {
        width,
        height,
        stride: width * 4,
        data: rgba_data,
    };

    let compressed = match dds_format {
        DdsSaveFormat::Bc1Unorm | DdsSaveFormat::Bc1UnormSrgb => {
            intel_tex_2::bc1::compress_blocks(&surface)
        }
        DdsSaveFormat::Bc3Unorm | DdsSaveFormat::Bc3UnormSrgb => {
            intel_tex_2::bc3::compress_blocks(&surface)
        }
        DdsSaveFormat::Bc4Unorm => intel_tex_2::bc4::compress_blocks(&surface),
        DdsSaveFormat::Bc5Unorm => intel_tex_2::bc5::compress_blocks(&surface),
        DdsSaveFormat::Bc7Unorm | DdsSaveFormat::Bc7UnormSrgb => {
            let settings = intel_tex_2::bc7::alpha_ultra_fast_settings();
            intel_tex_2::bc7::compress_blocks(&settings, &surface)
        }
        _ => unreachable!(),
    };

    let porter_fmt = dds_format.bc_porter_format().unwrap();

    let mut img = porter_texture::Image::new(width, height, porter_fmt)
        .map_err(|e| format!("Failed to create image: {e:?}"))?;

    let frame = img
        .create_frame()
        .map_err(|e| format!("Failed to create frame: {e:?}"))?;

    frame.buffer_mut().copy_from_slice(&compressed);

    img.save(file_path, porter_texture::ImageFileType::Dds)
        .map_err(|e| format!("Failed to save {filename}: {e:?}"))
}

fn save_uncompressed_dds(
    buffer: &ImageBuffer,
    file_path: &Path,
    dds_format: DdsSaveFormat,
    filename: &str,
) -> Result<(), String> {
    let (width, height) = buffer.dimensions();
    let rgba_data = buffer.as_raw();

    let source_fmt = dds_format.source_porter_format();
    let target_fmt = dds_format.uncompressed_porter_format().unwrap();

    let mut img = porter_texture::Image::new(width, height, source_fmt)
        .map_err(|e| format!("Failed to create image: {e:?}"))?;

    let frame = img
        .create_frame()
        .map_err(|e| format!("Failed to create frame: {e:?}"))?;

    frame.buffer_mut().copy_from_slice(rgba_data);

    if target_fmt != source_fmt {
        img.convert(target_fmt, porter_texture::ImageConvertOptions::None)
            .map_err(|e| format!("Failed to convert to {target_fmt:?}: {e:?}"))?;
    }

    img.save(file_path, porter_texture::ImageFileType::Dds)
        .map_err(|e| format!("Failed to save {filename}: {e:?}"))
}

fn save_standard_format(
    buffer: &ImageBuffer,
    file_path: &Path,
    filename: &str,
) -> Result<(), String> {
    let mut img = buffer.clone().into_porter_image()?;
    img.save(file_path)
        .map_err(|e| format!("Failed to save {filename}: {e}"))
}
