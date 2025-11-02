use crate::messages::ImageType;
use image::{DynamicImage, GenericImageView, ImageBuffer, Pixel, Rgba, imageops::FilterType};
use rayon::prelude::*;
use std::path::Path;

pub fn merge_images(
    colour_map: &DynamicImage,
    specular_map: Option<&DynamicImage>,
    occlusion_map: Option<&DynamicImage>,
    ao_contrast_power: f64,
    specular_intensity: f64,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (width, height) = colour_map.dimensions();

    let resized_spec: Option<DynamicImage> = specular_map.map(|s| {
        if s.dimensions() == (width, height) {
            s.clone()
        } else {
            s.resize(width, height, FilterType::CatmullRom)
        }
    });

    let resized_occ: Option<DynamicImage> = occlusion_map.map(|o| {
        if o.dimensions() == (width, height) {
            o.clone()
        } else {
            o.resize(width, height, FilterType::CatmullRom)
        }
    });

    // Create a LUT for the AO power curve
    let ao_lut: [u8; 256] = (0..=255)
        .map(|v| {
            ((v as f64 / 255.0).powf(ao_contrast_power) * 255.0)
                .round()
                .clamp(0.0, 255.0) as u8
        })
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap();

    let spec_lut: [u8; 256] = (0..=255)
        .map(|v| {
            ((v as f64 / 255.0) * specular_intensity * 255.0)
                .round()
                .clamp(0.0, 255.0) as u8
        })
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap();

    let mut output = ImageBuffer::new(width, height);

    // We need to use refs for the parallel loop
    let spec_ref = resized_spec.as_ref();
    let occ_ref = resized_occ.as_ref();

    output
        .par_enumerate_pixels_mut()
        .for_each(|(x, y, output_pixel)| {
            let colour_pixel = colour_map.get_pixel(x, y);

            let specular_pixel = spec_ref.map(|s| s.get_pixel(x, y));
            let occlusion_pixel = occ_ref.map(|o| o.get_pixel(x, y));

            let is_black = colour_pixel[0] < 38 && colour_pixel[1] < 38 && colour_pixel[2] < 38;

            let mut result = [0u8; 4];

            for i in 0..3 {
                let c = colour_pixel[i];

                // Use 255 (white) as default if maps not provided
                let s_u8 = specular_pixel.map(|p| p[i]).unwrap_or(255);
                let o_u8 = occlusion_pixel.map(|p| p[i]).unwrap_or(255);

                let adjusted_o = ao_lut[o_u8 as usize];
                let adjusted_s = spec_lut[s_u8 as usize];

                let baked_u16 = if is_black {
                    (adjusted_s as u16 * adjusted_o as u16) / 255
                } else {
                    (c as u16 * adjusted_o as u16) / 255
                };

                result[i] = baked_u16 as u8;
            }
            // Preserve original alpha
            result[3] = colour_pixel[3];
            *output_pixel = Rgba(result);
        });

    output
}

pub fn apply_detail_map(
    base_colour: &DynamicImage,
    detail_map: &DynamicImage,
    base_intensity: f64,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (width, height) = base_colour.dimensions();
    let detail_dims = detail_map.dimensions();

    let mut output = ImageBuffer::new(width, height);

    // Pre-calculate the base strength (from the "Opacity" 50%)
    let base_strength = 2.0 * base_intensity;

    output
        .par_enumerate_pixels_mut()
        .for_each(|(x, y, output_pixel)| {
            let base_pixel = base_colour.get_pixel(x, y).to_rgba();

            // Get the mask value from the base alpha channel
            let mask_value = base_pixel[3] as f64 / 255.0;

            // If the mask is 0, the blend does nothing.
            if mask_value == 0.0 {
                *output_pixel = base_pixel;
                return;
            }

            // Scale-to-fit mapping
            let norm_x = x as f64 / (width - 1) as f64;
            let norm_y = y as f64 / (height - 1) as f64;

            let detail_x = (norm_x * (detail_dims.0 - 1) as f64).round() as u32;
            let detail_y = (norm_y * (detail_dims.1 - 1) as f64).round() as u32;

            let detail_pixel = detail_map.get_pixel(detail_x, detail_y).to_rgba();

            let mut result = [0u8; 4];

            for i in 0..3 {
                // Iterate over R, G, B
                // Normalize linear values to 0.0 - 1.0
                let base_linear = base_pixel[i] as f64 / 255.0;
                let detail_linear = detail_pixel[i] as f64 / 255.0;

                // Calculate the final strength, scaled by the mask
                let final_strength = base_strength * mask_value;

                // Apply the LINEAR LIGHT formula, per-channel
                //    Result = Base + (Detail - 0.5) * (2.0 * Opacity * Mask)
                let modified_linear = base_linear + (detail_linear - 0.5) * final_strength;

                // Clamp and convert
                result[i] = (modified_linear.clamp(0.0, 1.0) * 255.0).round() as u8;
            }

            // Preserve base alpha
            result[3] = base_pixel[3];
            *output_pixel = Rgba(result);
        });

    output
}

/// Identifies which slot the image belongs to based on filename.
pub fn identify_image_type(path: &Path) -> Option<ImageType> {
    let filename = path.file_name()?.to_str()?;
    if filename.contains("_c.") {
        Some(ImageType::Colour)
    } else if filename.contains("_s.") {
        Some(ImageType::Specular)
    } else if filename.contains("_o.") {
        Some(ImageType::Occlusion)
    } else {
        None
    }
}

/// Identifies detail mapper image type based on filename suffix.
/// Returns BaseColor for _c, DetailMap for _d, or None if unrecognized.
pub fn identify_detail_image_type(
    path: &Path,
) -> Option<crate::components::detail_mapper::DetailImageType> {
    let filename = path.file_name()?.to_str()?;
    if filename.contains("_c.") {
        Some(crate::components::detail_mapper::DetailImageType::BaseColour)
    } else if filename.contains("_d.") {
        Some(crate::components::detail_mapper::DetailImageType::DetailMap)
    } else {
        None
    }
}

/// Optimized to avoid one full-image memory allocation and copy.
pub fn image_to_handle(img: &DynamicImage) -> iced::widget::image::Handle {
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    if width < 1 || height < 1 {
        tracing::error!("Image dimensions too small: {width}x{height}");
        return create_fallback_handle();
    }
    if width > 8192 || height > 8192 {
        tracing::error!("Image dimensions too large: {width}x{height}");
        return create_fallback_handle();
    }
    let width_f = width as f64;
    let height_f = height as f64;
    if !width_f.is_finite() || !height_f.is_finite() {
        tracing::error!("Image dimensions not finite");
        return create_fallback_handle();
    }
    if !width_f.is_normal() || !height_f.is_normal() {
        tracing::error!("Image dimensions not normal");
        return create_fallback_handle();
    }

    // This *consumes* rgba and gives its Vec<u8> without cloning.
    let bytes = rgba.into_raw();

    // Validate byte length
    let expected_len = match (width as usize)
        .checked_mul(height as usize)
        .and_then(|v| v.checked_mul(4))
    {
        Some(len) => len,
        None => {
            tracing::error!("Dimension overflow");
            return create_fallback_handle();
        }
    };

    if bytes.len() != expected_len {
        tracing::error!(
            "Buffer length mismatch after into_raw: {} vs {}",
            bytes.len(),
            expected_len
        );
        return create_fallback_handle();
    }

    iced::widget::image::Handle::from_rgba(width, height, bytes)
}

/// Convert an image buffer to an iced::image::Handle for display.
/// Validates dimensions to prevent rendering panics.
pub fn buffer_to_handle(buffer: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> iced::widget::image::Handle {
    let (width, height) = buffer.dimensions();

    // Check for zero or absurdly small dimensions
    if width < 1 || height < 1 {
        tracing::error!("Dimensions too small: {width}x{height}");
        return create_fallback_handle();
    }

    // Check for absurdly large dimensions
    if width > 8192 || height > 8192 {
        tracing::error!("Dimensions too large: {width}x{height}");
        return create_fallback_handle();
    }

    // Check as floats for NaN/infinity
    let width_f = width as f64;
    let height_f = height as f64;

    if !width_f.is_finite() || !height_f.is_finite() {
        tracing::error!("Dimensions not finite");
        return create_fallback_handle();
    }

    if !width_f.is_normal() || !height_f.is_normal() {
        tracing::error!("Dimensions not normal");
        return create_fallback_handle();
    }

    // Check if dimensions would cause overflow
    let expected_len = match (width as usize)
        .checked_mul(height as usize)
        .and_then(|v| v.checked_mul(4))
    {
        Some(len) => len,
        None => {
            tracing::error!("Dimension overflow");
            return create_fallback_handle();
        }
    };

    let bytes = buffer.as_raw();

    // Validate byte length matches expected dimensions
    if bytes.len() != expected_len {
        tracing::error!(
            "Buffer length mismatch: {} vs {}",
            bytes.len(),
            expected_len
        );
        return create_fallback_handle();
    }

    // Clone to avoid reference issues
    let bytes_clone = bytes.to_vec();

    // Verify the dimensions one more time before creating handle
    // This catches any edge cases where dimensions became invalid
    if width < 1 || height < 1 || width > 8192 || height > 8192 {
        tracing::error!("Final dimension check failed");
        return create_fallback_handle();
    }

    // Create the handle - at this point dimensions are guaranteed valid
    iced::widget::image::Handle::from_rgba(width, height, bytes_clone)
}

/// Create a safe fallback handle that will never cause rendering issues
fn create_fallback_handle() -> iced::widget::image::Handle {
    // Return a small 2x2 red pixel as error indicator
    iced::widget::image::Handle::from_rgba(
        2,
        2,
        vec![
            255, 0, 0, 255, // Red pixel
            255, 0, 0, 255, // Red pixel
            255, 0, 0, 255, // Red pixel
            255, 0, 0, 255, // Red pixel
        ],
    )
}
