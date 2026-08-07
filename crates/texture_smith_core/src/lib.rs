//! # Texture Smith Core
//!
//! Core image processing library for texture merging and baking.
//!
//! This crate provides pure image processing logic with no UI dependencies,
//! suitable for use as a library in CLI tools, C# bindings, or other frontends.
//!
//! ## Modules
//!
//! - [`porter_image`] — Image loading and conversion via porter-texture
//! - [`types`] — Shared type definitions, error types, and shader configuration
//! - [`gpu_processor`] — GPU-accelerated shader execution and image processing
//! - [`shader_manager`] — Shader discovery, loading, and validation
//! - [`save`] — Image saving in various formats (PNG, TGA, TIFF, DDS)

#[cfg(feature = "library")]
pub mod ffi;
pub mod gpu_processor;
pub mod porter_image;
pub mod save;
pub mod shader_manager;
pub mod types;

// Re-export commonly used types at the crate root for convenience.
pub use gpu_processor::process_images;
pub use porter_image::{ImageBuffer, PorterImage};
pub use save::{save_buffer, save_outputs};
pub use shader_manager::load_shaders;
pub use types::{
    ConverterError, ConverterResult, DdsSaveFormat, FileError, FileResult, GpuError, GpuResult,
    ImageFormat, InputConfig, OutputConfig, ShaderConfig, ShaderError, ShaderMetadata,
    ShaderParameter, ShaderResult,
};
