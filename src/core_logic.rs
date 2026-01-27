use crate::porter_image::{ImageBuffer, PorterImage};

/// Convert a PorterImage to iced display handle
pub fn image_to_handle(img: &mut PorterImage) -> iced::widget::image::Handle {
    match img.convert_to_rgba8() {
        Ok(_) => {
            let width = img.width();
            let height = img.height();

            if width < 1 || height < 1 {
                tracing::error!("Image dimensions too small: {width}x{height}");
                return create_fallback_handle();
            }
            if width > 8192 || height > 8192 {
                tracing::error!("Image dimensions too large: {width}x{height}");
                return create_fallback_handle();
            }

            match img.as_rgba8_buffer() {
                Ok(buffer) => {
                    let bytes = buffer.to_vec();

                    // Validate byte length
                    let expected_len = (width as usize)
                        .checked_mul(height as usize)
                        .and_then(|v| v.checked_mul(4));

                    match expected_len {
                        Some(len) if bytes.len() == len => {
                            iced::widget::image::Handle::from_rgba(width, height, bytes)
                        }
                        _ => {
                            tracing::error!("Buffer length mismatch");
                            create_fallback_handle()
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to get RGBA8 buffer: {}", e);
                    create_fallback_handle()
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to convert to RGBA8: {}", e);
            create_fallback_handle()
        }
    }
}

/// Convert an image buffer to an iced::image::Handle for display.
/// Validates dimensions to prevent rendering panics.
pub fn buffer_to_handle(buffer: &ImageBuffer) -> iced::widget::image::Handle {
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
