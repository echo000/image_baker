use crate::core_logic;
use crate::status::StatusMessage;
use ::image::{DynamicImage, ImageBuffer, Rgba};
use image::GenericImageView;
use std::path::{Path, PathBuf};

/// Validates that an image has acceptable dimensions
pub fn validate_image_dimensions(img: &DynamicImage) -> bool {
    let (w, h) = img.dimensions();
    w > 0 && h > 0 && w <= 8192 && h <= 8192
}

/// Validates that a buffer has acceptable dimensions
pub fn validate_buffer_dimensions(buffer: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> bool {
    let (w, h) = buffer.dimensions();
    w > 0 && h > 0 && w <= 8192 && h <= 8192
}

/// Generic async function to load an image from a file
pub async fn load_image_async<T: Clone>(
    path: PathBuf,
    image_type: T,
) -> (T, Result<DynamicImage, String>) {
    let result = ::image::open(&path)
        .map_err(|e| format!("Failed to open image: {e}"))
        .and_then(|img| {
            if validate_image_dimensions(&img) {
                Ok(img)
            } else {
                Err("Image dimensions are invalid or too large (max 8192x8192)".to_string())
            }
        });

    (image_type, result)
}

/// Generic async function to save an image buffer
pub async fn save_image_async(buffer: ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<PathBuf, String> {
    let dialog = rfd::AsyncFileDialog::new()
        .set_file_name("output.png")
        .add_filter("PNG Image", &["png"])
        .add_filter("TIFF Image", &["tiff", "tif"])
        .add_filter("TGA Image", &["tga"])
        .save_file()
        .await;

    if let Some(file_handle) = dialog {
        let path = file_handle.path().to_path_buf();
        buffer
            .save(&path)
            .map_err(|e| format!("Failed to save image: {e}"))?;
        Ok(path)
    } else {
        Err("Save cancelled".to_string())
    }
}

/// Generic async function to browse for an image file
pub async fn browse_for_image_async<T: Clone>(image_type: T) -> (T, Option<PathBuf>) {
    let dialog = rfd::AsyncFileDialog::new()
        .add_filter("Images", &["png", "tiff", "tif", "dds", "tga"])
        .pick_file()
        .await;

    let path = dialog.map(|handle| handle.path().to_path_buf());
    (image_type, path)
}

/// Creates a status message for loading an image
pub fn loading_status_message(image_type: &dyn std::fmt::Debug, path: &Path) -> StatusMessage {
    StatusMessage::info(format!(
        "Loading {:?} from {:?}...",
        image_type,
        path.file_name().unwrap_or_default()
    ))
}

/// Creates a status message for a loaded image
pub fn loaded_status_message(image_type: &dyn std::fmt::Debug) -> StatusMessage {
    StatusMessage::success(format!("Loaded {image_type:?} image."))
}

/// Creates a status message for an error loading an image
pub fn load_error_status_message(error: &str) -> StatusMessage {
    StatusMessage::error(format!("Error loading image: {error}"))
}

/// Creates a status message for invalid dimensions
pub fn invalid_dimensions_status_message(image_type: &dyn std::fmt::Debug) -> StatusMessage {
    StatusMessage::error(format!("Invalid dimensions for {image_type:?} image"))
}

/// Creates a status message for merge complete
pub fn merge_complete_status_message() -> StatusMessage {
    StatusMessage::success("Merge complete! Output is ready.")
}

/// Creates a status message for merge error
pub fn merge_error_status_message(error: &str) -> StatusMessage {
    StatusMessage::error(format!("Error merging images: {error}"))
}

/// Creates a status message for save success
pub fn save_success_status_message(path: &PathBuf) -> StatusMessage {
    StatusMessage::success(format!("Saved successfully to {path:?}"))
}

/// Creates a status message for save error
pub fn save_error_status_message(error: &str) -> StatusMessage {
    StatusMessage::error(format!("Failed to save: {error}"))
}

/// Creates a status message for unknown file dropped
pub fn unknown_file_status_message(path: &Path, expected_suffixes: &str) -> StatusMessage {
    StatusMessage::warning(format!(
        "Skipped file: {:?} (no {} suffix, try with browse)",
        path.file_name().unwrap_or_default(),
        expected_suffixes
    ))
}

/// Converts an image buffer to a handle for display
pub fn buffer_to_display_handle(
    buffer: &ImageBuffer<Rgba<u8>, Vec<u8>>,
) -> Option<iced::widget::image::Handle> {
    if validate_buffer_dimensions(buffer) {
        Some(core_logic::buffer_to_handle(buffer))
    } else {
        None
    }
}
