//! Compatibility wrapper for porter_texture::Image to ease migration from image crate.
//!
//! This module provides a thin wrapper around porter_texture::Image that provides
//! a more ergonomic API similar to the image crate's DynamicImage.

use porter_texture::{Image, ImageFileType, ImageFormat};
use std::path::Path;

/// Wrapper around porter_texture::Image with ergonomic API
#[derive(Debug, Clone)]
pub struct PorterImage {
    inner: Image,
}

impl PorterImage {
    /// Load an image from a file path, auto-detecting the format
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path = path.as_ref();
        let file_type = detect_file_type(path)?;

        let inner =
            Image::load(path, file_type).map_err(|e| format!("Failed to load image: {e:?}"))?;

        Ok(Self { inner })
    }

    /// Create a new image with the given dimensions and format
    pub fn new(width: u32, height: u32, format: ImageFormat) -> Result<Self, String> {
        let inner = Image::new(width, height, format)
            .map_err(|e| format!("Failed to create image: {e:?}"))?;

        Ok(Self { inner })
    }

    /// Get the dimensions of the image
    pub fn dimensions(&self) -> (u32, u32) {
        (self.inner.width(), self.inner.height())
    }

    /// Get the width of the image
    pub fn width(&self) -> u32 {
        self.inner.width()
    }

    /// Get the height of the image
    pub fn height(&self) -> u32 {
        self.inner.height()
    }

    /// Get the image format
    pub fn format(&self) -> ImageFormat {
        self.inner.format()
    }

    /// Convert to RGBA8 format if not already (keeps sRGB colour space for display)
    pub fn convert_to_rgba8(&mut self) -> Result<(), String> {
        let current_format = self.inner.format();

        // Convert to RGB/sRGB as needed
        let target_format = if current_format.is_srgb() {
            ImageFormat::R8G8B8A8UnormSrgb
        } else {
            ImageFormat::R8G8B8A8Unorm
        };

        // If already in target format, no its a noop
        self.inner
            .convert(target_format, Default::default())
            .map_err(|e| format!("Failed to convert to RGBA8: {e:?}"))?;
        Ok(())
    }

    /// Get a reference to the raw buffer (first frame, RGBA8 format)
    /// Ensures the image is in RGBA8 format first
    pub fn as_rgba8_buffer(&mut self) -> Result<&[u8], String> {
        self.convert_to_rgba8()?;

        let frame = self
            .inner
            .frames()
            .first()
            .ok_or_else(|| "Image has no frames".to_string())?;

        Ok(frame.buffer())
    }

    /// Get the raw buffer without conversion (first frame)
    pub fn raw_buffer(&self) -> Result<&[u8], String> {
        let frame = self
            .inner
            .frames()
            .first()
            .ok_or_else(|| "Image has no frames".to_string())?;

        Ok(frame.buffer())
    }

    /// Get a pixel at the given coordinates (RGBA8 format)
    pub fn get_pixel(&mut self, x: u32, y: u32) -> Result<[u8; 4], String> {
        let width = self.width();
        let height = self.height();

        if x >= width || y >= height {
            return Err(format!("Pixel coordinates ({x}, {y}) out of bounds"));
        }

        self.convert_to_rgba8()?;

        let buffer = self.as_rgba8_buffer()?;
        let bytes_per_pixel = 4;
        let offset = ((y * width + x) * bytes_per_pixel) as usize;

        Ok([
            buffer[offset],
            buffer[offset + 1],
            buffer[offset + 2],
            buffer[offset + 3],
        ])
    }

    /// Save the image to a file
    pub fn save<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let path = path.as_ref();
        let file_type = detect_file_type(path)?;

        let best_format = self.inner.format_for_file_type(file_type);
        self.inner
            .convert(best_format, porter_texture::ImageConvertOptions::None)
            .map_err(|e| format!("Failed to convert image: {e:?}"))?;

        self.inner
            .save(path, file_type)
            .map_err(|e| format!("Failed to save image: {e:?}"))
    }

    /// Get a reference to the inner porter_texture::Image
    pub fn inner(&self) -> &Image {
        &self.inner
    }

    /// Get a mutable reference to the inner porter_texture::Image
    pub fn inner_mut(&mut self) -> &mut Image {
        &mut self.inner
    }

    /// Consume and return the inner porter_texture::Image
    pub fn into_inner(self) -> Image {
        self.inner
    }

    /// Create from an existing porter_texture::Image
    pub fn from_inner(inner: Image) -> Self {
        Self { inner }
    }

    /// Resize the image to new dimensions
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), String> {
        use porter_texture::ResizeAlgorithm;

        self.inner
            .resize(width, height, ResizeAlgorithm::Bicubic)
            .map_err(|e| format!("Failed to resize image: {e:?}"))
    }
}

/// RGBA8 image buffer wrapper
#[derive(Debug, Clone)]
pub struct ImageBuffer {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl ImageBuffer {
    /// Create a new image buffer from dimensions and data
    pub fn from_raw(width: u32, height: u32, data: Vec<u8>) -> Option<Self> {
        let expected_size = (width * height * 4) as usize;
        if data.len() != expected_size {
            return None;
        }

        Some(Self {
            width,
            height,
            data,
        })
    }

    /// Create a new image buffer with all pixels set to a colour
    pub fn from_pixel(width: u32, height: u32, pixel: [u8; 4]) -> Self {
        let size = (width * height * 4) as usize;
        let mut data = Vec::with_capacity(size);

        for _ in 0..(width * height) {
            data.extend_from_slice(&pixel);
        }

        Self {
            width,
            height,
            data,
        }
    }

    /// Get the dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get the width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get a reference to the raw data
    pub fn as_raw(&self) -> &[u8] {
        &self.data
    }

    /// Get the raw data as a mutable slice
    pub fn as_raw_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Convert to a PorterImage (uses sRGB format for display)
    pub fn into_porter_image(self) -> Result<PorterImage, String> {
        let mut image = Image::new(self.width, self.height, ImageFormat::R8G8B8A8UnormSrgb)
            .map_err(|e| format!("Failed to create image: {e:?}"))?;

        let frame = image
            .create_frame()
            .map_err(|e| format!("Failed to create frame: {e:?}"))?;

        frame.buffer_mut().copy_from_slice(&self.data);

        Ok(PorterImage::from_inner(image))
    }

    /// Get a pixel at the given coordinates
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }

        let offset = ((y * self.width + x) * 4) as usize;
        Some([
            self.data[offset],
            self.data[offset + 1],
            self.data[offset + 2],
            self.data[offset + 3],
        ])
    }

    /// Set a pixel at the given coordinates
    pub fn put_pixel(&mut self, x: u32, y: u32, pixel: [u8; 4]) {
        if x >= self.width || y >= self.height {
            return;
        }

        let offset = ((y * self.width + x) * 4) as usize;
        self.data[offset..offset + 4].copy_from_slice(&pixel);
    }

    /// Enumerate all pixels with mutable access
    pub fn enumerate_pixels_mut(&mut self) -> impl Iterator<Item = (u32, u32, &mut [u8])> {
        let width = self.width;
        self.data
            .chunks_exact_mut(4)
            .enumerate()
            .map(move |(i, pixel)| {
                let x = (i as u32) % width;
                let y = (i as u32) / width;
                (x, y, pixel)
            })
    }

    /// Clone the data into a new Vec
    pub fn into_raw(self) -> Vec<u8> {
        self.data
    }
}

/// Detect file type from file extension
fn detect_file_type(path: &Path) -> Result<ImageFileType, String> {
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("No file extension found for: {}", path.display()))?;

    match extension.to_lowercase().as_str() {
        "png" => Ok(ImageFileType::Png),
        "tga" => Ok(ImageFileType::Tga),
        "tif" | "tiff" => Ok(ImageFileType::Tiff),
        "dds" => Ok(ImageFileType::Dds),
        _ => Err(format!("Unsupported file extension: {extension}")),
    }
}
