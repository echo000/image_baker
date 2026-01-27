//! Type definitions and error types for the texture converter module

use std::path::PathBuf;

/// Result type for texture converter operations
#[allow(dead_code)]
pub type ConverterResult<T> = Result<T, ConverterError>;

/// Result type for shader operations
#[allow(dead_code)]
pub type ShaderResult<T> = Result<T, ShaderError>;

/// Result type for GPU operations
#[allow(dead_code)]
pub type GpuResult<T> = Result<T, GpuError>;

/// Result type for file operations
#[allow(dead_code)]
pub type FileResult<T> = Result<T, FileError>;

/// Supported output image formats (PorterLib supported formats only)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageFormat {
    #[default]
    Png,
    Tga,
    Tiff,
    Dds,
}

impl ImageFormat {
    /// Get file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Tga => "tga",
            ImageFormat::Tiff => "tiff",
            ImageFormat::Dds => "dds",
        }
    }

    /// Get display name for this format
    pub fn display_name(&self) -> &'static str {
        match self {
            ImageFormat::Png => "PNG",
            ImageFormat::Tga => "TGA",
            ImageFormat::Tiff => "TIFF",
            ImageFormat::Dds => "DDS",
        }
    }

    /// Get all available formats
    pub const ALL: [ImageFormat; 4] = [
        ImageFormat::Png,
        ImageFormat::Tga,
        ImageFormat::Tiff,
        ImageFormat::Dds,
    ];
}

impl std::fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Main error type for texture converter operations
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ConverterError {
    Shader(ShaderError),
    Gpu(GpuError),
    File(FileError),
    ImageProcessing(String),
    NoOutputsAvailable,
    NoShaderSelected,
    InvalidOperation(String),
}

impl std::fmt::Display for ConverterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConverterError::Shader(e) => write!(f, "Shader error: {e}"),
            ConverterError::Gpu(e) => write!(f, "GPU error: {e}"),
            ConverterError::File(e) => write!(f, "File error: {e}"),
            ConverterError::ImageProcessing(msg) => write!(f, "Image processing error: {msg}"),
            ConverterError::NoOutputsAvailable => write!(f, "No outputs available to save"),
            ConverterError::NoShaderSelected => write!(f, "No shader selected"),
            ConverterError::InvalidOperation(msg) => write!(f, "Invalid operation: {msg}"),
        }
    }
}

impl std::error::Error for ConverterError {}

impl From<ShaderError> for ConverterError {
    fn from(err: ShaderError) -> Self {
        ConverterError::Shader(err)
    }
}

impl From<GpuError> for ConverterError {
    fn from(err: GpuError) -> Self {
        ConverterError::Gpu(err)
    }
}

impl From<FileError> for ConverterError {
    fn from(err: FileError) -> Self {
        ConverterError::File(err)
    }
}

/// Shader-related errors
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ShaderError {
    DirectoryNotFound,
    NoShadersFound,
    LoadFailed { path: PathBuf, reason: String },
    ParseFailed { path: PathBuf, reason: String },
    ValidationFailed { shader_name: String, reason: String },
    InvalidConfig { shader_name: String, reason: String },
    CompilationFailed { shader_name: String, reason: String },
    GpuInitFailed(String),
}

impl std::fmt::Display for ShaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShaderError::DirectoryNotFound => {
                write!(
                    f,
                    "Shaders directory not found. Please create a 'shaders' directory."
                )
            }
            ShaderError::NoShadersFound => {
                write!(f, "No valid shaders found in shaders directory")
            }
            ShaderError::LoadFailed { path, reason } => {
                write!(f, "Failed to load shader from {path:?}: {reason}")
            }
            ShaderError::ParseFailed { path, reason } => {
                write!(f, "Failed to parse shader config {path:?}: {reason}")
            }
            ShaderError::ValidationFailed {
                shader_name,
                reason,
            } => {
                write!(f, "Shader '{shader_name}' validation failed: {reason}")
            }
            ShaderError::InvalidConfig {
                shader_name,
                reason,
            } => {
                write!(
                    f,
                    "Shader '{shader_name}' has invalid configuration: {reason}"
                )
            }
            ShaderError::CompilationFailed {
                shader_name,
                reason,
            } => {
                write!(f, "Shader '{shader_name}' compilation failed: {reason}")
            }
            ShaderError::GpuInitFailed(reason) => {
                write!(
                    f,
                    "Failed to initialize GPU for shader validation: {reason}"
                )
            }
        }
    }
}

impl std::error::Error for ShaderError {}

/// GPU processing errors
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum GpuError {
    NoImagesProvided,
    ImageConversionFailed { reason: String },
    BufferCreationFailed { reason: String },
    TextureCreationFailed { reason: String },
    PipelineCreationFailed { reason: String },
    RenderFailed { reason: String },
    BufferReadbackFailed { reason: String },
    InvalidDimensions { width: u32, height: u32 },
}

impl std::fmt::Display for GpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuError::NoImagesProvided => write!(f, "No images provided for processing"),
            GpuError::ImageConversionFailed { reason } => {
                write!(f, "Failed to convert image: {reason}")
            }
            GpuError::BufferCreationFailed { reason } => {
                write!(f, "Failed to create GPU buffer: {reason}")
            }
            GpuError::TextureCreationFailed { reason } => {
                write!(f, "Failed to create GPU texture: {reason}")
            }
            GpuError::PipelineCreationFailed { reason } => {
                write!(f, "Failed to create render pipeline: {reason}")
            }
            GpuError::RenderFailed { reason } => write!(f, "Rendering failed: {reason}"),
            GpuError::BufferReadbackFailed { reason } => {
                write!(f, "Failed to read GPU buffer: {reason}")
            }
            GpuError::InvalidDimensions { width, height } => {
                write!(f, "Invalid image dimensions: {width}x{height}")
            }
        }
    }
}

impl std::error::Error for GpuError {}

/// File operation errors
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum FileError {
    LoadFailed { path: PathBuf, reason: String },
    SaveFailed { path: PathBuf, reason: String },
    SaveCancelled,
    InvalidPath(PathBuf),
    UnsupportedFormat(String),
    ConversionFailed(String),
}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileError::LoadFailed { path, reason } => {
                write!(f, "Failed to load file {path:?}: {reason}")
            }
            FileError::SaveFailed { path, reason } => {
                write!(f, "Failed to save file {path:?}: {reason}")
            }
            FileError::SaveCancelled => write!(f, "Save operation was cancelled"),
            FileError::InvalidPath(path) => write!(f, "Invalid file path: {path:?}"),
            FileError::UnsupportedFormat(format) => {
                write!(f, "Unsupported file format: {format}")
            }
            FileError::ConversionFailed(reason) => {
                write!(f, "Failed to convert image buffer: {reason}")
            }
        }
    }
}

impl std::error::Error for FileError {}
