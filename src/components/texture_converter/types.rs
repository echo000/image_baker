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

/// DDS pixel format sub-selection (shown when DDS is the export format).
///
/// BC formats are encoded with intel_tex_2.
/// Uncompressed formats are converted via porter-texture's GPU converter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DdsSaveFormat {
    // ── BC compressed ──────────────────────────────────────────────────────
    /// BC1 / DXT1, linear, 4 bpp (RGB, 1-bit alpha)
    Bc1Unorm,
    /// BC1 / DXT1, sRGB, 4 bpp
    Bc1UnormSrgb,
    /// BC3 / DXT5, linear, 8 bpp (RGBA)
    Bc3Unorm,
    /// BC3 / DXT5, sRGB, 8 bpp
    Bc3UnormSrgb,
    /// BC4, linear, 4 bpp (R only — greyscale/AO/roughness)
    Bc4Unorm,
    /// BC5, linear, 8 bpp (RG — normal maps)
    Bc5Unorm,
    /// BC7, linear, 8 bpp (high-quality RGBA)
    Bc7Unorm,
    /// BC7, sRGB, 8 bpp
    Bc7UnormSrgb,

    // ── Uncompressed ───────────────────────────────────────────────────────
    /// R8G8B8A8 Unorm (linear) — default
    #[default]
    Rgba8Unorm,
    /// R8G8B8A8 sRGB
    Rgba8UnormSrgb,
    /// B8G8R8A8 Unorm (linear)
    Bgra8Unorm,
    /// B8G8R8A8 sRGB
    Bgra8UnormSrgb,
    /// R8 Unorm — single channel greyscale
    R8Unorm,
    /// R8G8 Unorm — two channel
    R8G8Unorm,
    /// R16G16B16A16 Float — half-precision HDR
    Rgba16Float,
    /// R32 Float — single channel 32-bit float
    R32Float,
}

impl DdsSaveFormat {
    pub fn display_name(&self) -> &'static str {
        match self {
            DdsSaveFormat::Bc1Unorm => "BC1 (Linear, DXT1)",
            DdsSaveFormat::Bc1UnormSrgb => "BC1 (sRGB, DX 10+)",
            DdsSaveFormat::Bc3Unorm => "BC3 (Linear, DXT5)",
            DdsSaveFormat::Bc3UnormSrgb => "BC3 (sRGB, DX 10+)",
            DdsSaveFormat::Bc4Unorm => "BC4 (Linear, Unsigned)",
            DdsSaveFormat::Bc5Unorm => "BC5 (Linear, Unsigned)",
            DdsSaveFormat::Bc7Unorm => "BC7 (Linear, DX 11+)",
            DdsSaveFormat::Bc7UnormSrgb => "BC7 (sRGB, DX 11+)",
            DdsSaveFormat::Rgba8Unorm => "R8G8B8A8 (Linear, Unorm)",
            DdsSaveFormat::Rgba8UnormSrgb => "R8G8B8A8 (sRGB)",
            DdsSaveFormat::Bgra8Unorm => "B8G8R8A8 (Linear, Unorm)",
            DdsSaveFormat::Bgra8UnormSrgb => "B8G8R8A8 (sRGB)",
            DdsSaveFormat::R8Unorm => "R8 (Linear, Unsigned)",
            DdsSaveFormat::R8G8Unorm => "R8G8 (Linear, Unsigned)",
            DdsSaveFormat::Rgba16Float => "R16G16B16A16 (Float)",
            DdsSaveFormat::R32Float => "R32 (Float)",
        }
    }

    /// Whether this format requires BC compression via intel_tex_2.
    pub fn is_bc_compressed(self) -> bool {
        matches!(
            self,
            DdsSaveFormat::Bc1Unorm
                | DdsSaveFormat::Bc1UnormSrgb
                | DdsSaveFormat::Bc3Unorm
                | DdsSaveFormat::Bc3UnormSrgb
                | DdsSaveFormat::Bc4Unorm
                | DdsSaveFormat::Bc5Unorm
                | DdsSaveFormat::Bc7Unorm
                | DdsSaveFormat::Bc7UnormSrgb
        )
    }

    /// The source porter-texture ImageFormat to use when building the Image
    /// before conversion (sRGB source for sRGB targets avoids gamma round-trips).
    pub fn source_porter_format(self) -> porter_texture::ImageFormat {
        if matches!(
            self,
            DdsSaveFormat::Rgba8UnormSrgb | DdsSaveFormat::Bgra8UnormSrgb
        ) {
            porter_texture::ImageFormat::R8G8B8A8UnormSrgb
        } else {
            porter_texture::ImageFormat::R8G8B8A8Unorm
        }
    }

    /// The target porter-texture ImageFormat for uncompressed DDS saves.
    /// Returns `None` for BC formats (handled separately via intel_tex_2).
    pub fn uncompressed_porter_format(self) -> Option<porter_texture::ImageFormat> {
        match self {
            DdsSaveFormat::Rgba8Unorm => Some(porter_texture::ImageFormat::R8G8B8A8Unorm),
            DdsSaveFormat::Rgba8UnormSrgb => Some(porter_texture::ImageFormat::R8G8B8A8UnormSrgb),
            DdsSaveFormat::Bgra8Unorm => Some(porter_texture::ImageFormat::B8G8R8A8Unorm),
            DdsSaveFormat::Bgra8UnormSrgb => Some(porter_texture::ImageFormat::B8G8R8A8UnormSrgb),
            DdsSaveFormat::R8Unorm => Some(porter_texture::ImageFormat::R8Unorm),
            DdsSaveFormat::R8G8Unorm => Some(porter_texture::ImageFormat::R8G8Unorm),
            DdsSaveFormat::Rgba16Float => {
                Some(porter_texture::ImageFormat::R16G16B16A16Float)
            }
            DdsSaveFormat::R32Float => Some(porter_texture::ImageFormat::R32Float),
            _ => None, // BC formats
        }
    }

    /// The porter-texture ImageFormat used as the frame format for BC saves.
    /// Returns `None` for uncompressed formats.
    pub fn bc_porter_format(self) -> Option<porter_texture::ImageFormat> {
        match self {
            DdsSaveFormat::Bc1Unorm => Some(porter_texture::ImageFormat::Bc1Unorm),
            DdsSaveFormat::Bc1UnormSrgb => Some(porter_texture::ImageFormat::Bc1UnormSrgb),
            DdsSaveFormat::Bc3Unorm => Some(porter_texture::ImageFormat::Bc3Unorm),
            DdsSaveFormat::Bc3UnormSrgb => Some(porter_texture::ImageFormat::Bc3UnormSrgb),
            DdsSaveFormat::Bc4Unorm => Some(porter_texture::ImageFormat::Bc4Unorm),
            DdsSaveFormat::Bc5Unorm => Some(porter_texture::ImageFormat::Bc5Unorm),
            DdsSaveFormat::Bc7Unorm => Some(porter_texture::ImageFormat::Bc7Unorm),
            DdsSaveFormat::Bc7UnormSrgb => Some(porter_texture::ImageFormat::Bc7UnormSrgb),
            _ => None,
        }
    }

    pub const ALL: [DdsSaveFormat; 16] = [
        // BC compressed
        DdsSaveFormat::Bc1Unorm,
        DdsSaveFormat::Bc1UnormSrgb,
        DdsSaveFormat::Bc3Unorm,
        DdsSaveFormat::Bc3UnormSrgb,
        DdsSaveFormat::Bc4Unorm,
        DdsSaveFormat::Bc5Unorm,
        DdsSaveFormat::Bc7Unorm,
        DdsSaveFormat::Bc7UnormSrgb,
        // Uncompressed
        DdsSaveFormat::Rgba8Unorm,
        DdsSaveFormat::Rgba8UnormSrgb,
        DdsSaveFormat::Bgra8Unorm,
        DdsSaveFormat::Bgra8UnormSrgb,
        DdsSaveFormat::R8Unorm,
        DdsSaveFormat::R8G8Unorm,
        DdsSaveFormat::Rgba16Float,
        DdsSaveFormat::R32Float,
    ];
}

impl std::fmt::Display for DdsSaveFormat {
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
