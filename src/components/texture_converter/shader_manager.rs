//! Shader Manager Module
//!
//! Handles shader discovery, loading, and validation.
//!
//! This module provides functionality to:
//! - Find and load shader files from the shaders directory
//! - Parse shader configuration (TOML)
//! - Validate WGSL shader code syntax
//! - Initialize GPU device for shader validation

use crate::components::texture_converter::ShaderConfig;
use std::path::{Path, PathBuf};

/// Result type for shader operations
pub type ShaderResult<T> = Result<T, String>;

/// Loads all shaders from the shaders directory
///
/// Scans for shader directories containing `shader.wgsl` and `config.toml`,
/// validates each shader, and returns successfully loaded shaders along with
/// a count of failed shaders.
///
/// # Returns
/// - `Ok((Vec<ShaderConfig>, usize))` - Loaded shaders and failed count
/// - `Err(String)` - Error message if shader directory not found or GPU init fails
pub async fn load_shaders() -> ShaderResult<(Vec<ShaderConfig>, usize)> {
    tracing::info!("Starting shader loading process...");

    // Initialize wgpu for shader validation
    let (device, _queue) = initialize_gpu_device().await?;

    // Find the shaders directory
    let shaders_dir = find_shaders_directory()?;
    tracing::info!("Loading shaders from: {}", shaders_dir.display());

    // Discover all shader files
    let shader_files = discover_shader_files(&shaders_dir)?;

    if shader_files.is_empty() {
        return Ok((Vec::new(), 0));
    }

    // Load and validate each shader
    let mut loaded_shaders = Vec::new();
    let mut failed_count = 0;

    for shader_path in shader_files {
        match load_and_validate_shader(&shader_path, &device).await {
            Ok(shader) => loaded_shaders.push(shader),
            Err(e) => {
                tracing::error!("Failed to load shader {:?}: {}", shader_path, e);
                failed_count += 1;
            }
        }
    }

    tracing::info!(
        "Shader loading complete: {} loaded, {} failed",
        loaded_shaders.len(),
        failed_count
    );

    Ok((loaded_shaders, failed_count))
}

/// Initialize GPU device for shader validation
///
/// Creates a temporary GPU device used only for validating shader syntax.
/// This device is not used for actual rendering.
async fn initialize_gpu_device() -> ShaderResult<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .map_err(|_| "Failed to find suitable GPU adapter for shader validation".to_string())?;

    adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("Shader Validation Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: Default::default(),
        })
        .await
        .map_err(|e| format!("Failed to create GPU device for shader validation: {e}"))
}

/// Find the shaders directory in various possible locations
///
/// Searches in order:
/// 1. Next to the executable
/// 2. Current working directory
/// 3. Parent directory (for development)
fn find_shaders_directory() -> ShaderResult<PathBuf> {
    // Try multiple locations for the shaders directory
    let possible_locations = vec![
        // Next to the executable
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|p| p.join("shaders"))),
        // In current working directory
        Some(PathBuf::from("shaders")),
        // In parent directory (for development)
        Some(PathBuf::from("../shaders")),
    ];

    for location in possible_locations.into_iter().flatten() {
        if location.exists() && location.is_dir() {
            return Ok(location);
        }
    }

    Err("Could not find 'shaders' directory. Please create it and add shader files.".to_string())
}

/// Discover all shader files in the directory
///
/// Looks for subdirectories containing both `shader.wgsl` and `config.toml`.
/// Each subdirectory represents one shader.
fn discover_shader_files(shaders_dir: &Path) -> ShaderResult<Vec<PathBuf>> {
    let entries = std::fs::read_dir(shaders_dir)
        .map_err(|e| format!("Failed to read shaders directory: {e}"))?;

    let mut shader_files = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();

        // Look for subdirectories containing shader.wgsl and config.toml
        if path.is_dir() {
            let shader_wgsl = path.join("shader.wgsl");
            let config_toml = path.join("config.toml");

            if shader_wgsl.exists() && config_toml.exists() {
                shader_files.push(path);
            }
        }
    }

    Ok(shader_files)
}

/// Load and validate a single shader
///
/// Performs the following steps:
/// 1. Load and parse config.toml
/// 2. Validate WGSL shader code
/// 3. Validate configuration consistency
async fn load_and_validate_shader(
    shader_dir: &Path,
    device: &wgpu::Device,
) -> ShaderResult<ShaderConfig> {
    let shader_wgsl = shader_dir.join("shader.wgsl");
    let config_toml = shader_dir.join("config.toml");

    // Load and parse config
    let config_content = std::fs::read_to_string(&config_toml)
        .map_err(|e| format!("Failed to read config.toml: {e}"))?;

    let mut shader_config: ShaderConfig =
        toml::from_str(&config_content).map_err(|e| format!("Failed to parse config.toml: {e}"))?;

    // Store the shader path
    shader_config.shader_path = shader_wgsl.clone();

    // Validate the shader code
    validate_shader_code(&shader_wgsl, device)?;

    // Validate config consistency
    validate_shader_config(&shader_config)?;

    tracing::info!("Successfully loaded shader: {}", shader_config.shader.name);

    Ok(shader_config)
}

/// Validate shader WGSL code
///
/// Attempts to compile the shader to check for syntax errors.
/// Uses panic catching to handle compilation failures gracefully.
fn validate_shader_code(shader_path: &Path, device: &wgpu::Device) -> ShaderResult<()> {
    let shader_code = std::fs::read_to_string(shader_path)
        .map_err(|e| format!("Failed to read shader file: {e}"))?;

    // Try to create a shader module to validate the code
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Validation Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        })
    }));

    match result {
        Ok(_) => Ok(()),
        Err(_) => Err("Shader compilation failed - invalid WGSL syntax".to_string()),
    }
}

/// Validate shader configuration
///
/// Checks that:
/// - At least one output is defined
/// - All output entry points and suffixes are non-empty
/// - Parameter ranges are valid (min <= default <= max)
fn validate_shader_config(config: &ShaderConfig) -> ShaderResult<()> {
    // Check that shader has at least one output
    if config.outputs.is_empty() {
        return Err("Shader must have at least one output".to_string());
    }

    // Check that all outputs have valid entry points
    for output in &config.outputs {
        if output.entry_point.is_empty() {
            return Err("Output entry point cannot be empty".to_string());
        }
        if output.suffix.is_empty() {
            return Err("Output suffix cannot be empty".to_string());
        }
    }

    // Check that all parameters have valid ranges
    for param in &config.parameters {
        if param.min > param.max {
            return Err(format!(
                "Parameter '{}' has invalid range: min ({}) > max ({})",
                param.name, param.min, param.max
            ));
        }
        if param.default < param.min || param.default > param.max {
            return Err(format!(
                "Parameter '{}' default value ({}) is outside valid range [{}, {}]",
                param.name, param.default, param.min, param.max
            ));
        }
    }

    Ok(())
}
