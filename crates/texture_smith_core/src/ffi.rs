use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;
use std::sync::Arc;

use crate::gpu_processor::process_images;
use crate::porter_image::PorterImage;
use crate::save::{save_buffer, save_outputs};
use crate::shader_manager::load_shaders_from_dir;
use crate::types::{DdsSaveFormat, ImageFormat};

/// Opaque handle to processed output images.
pub struct TsResult {
    outputs: Vec<(crate::porter_image::ImageBuffer, String)>,
}

/// Process images through a named shader.
///
/// - `shaders_dir`:  path to the directory containing shader subdirectories
/// - `shader_name`:  the `[shader].name` value from the target `config.toml`
/// - `input_data`:   array of pointers to raw image file bytes (PNG/TGA/DDS/TIFF)
/// - `input_lens`:   byte length of each input buffer
/// - `extensions`:   null-terminated file extension for each input (e.g. `"png"`)
/// - `image_count`:  number of input images
/// - `param_names`:  array of parameter name strings (may be null if `param_count == 0`)
/// - `param_values`: array of f32 parameter values   (may be null if `param_count == 0`)
/// - `param_count`:  number of parameters
///
/// Returns an opaque `TsResult*` on success, or null on failure.
/// On failure, call `ts_last_error()` for the error message.
/// Free the result with `ts_result_free` when done.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_process(
    shaders_dir: *const c_char,
    shader_name: *const c_char,
    input_data: *const *const u8,
    input_lens: *const usize,
    extensions: *const *const c_char,
    image_count: u32,
    param_names: *const *const c_char,
    param_values: *const f32,
    param_count: u32,
) -> *mut TsResult {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        ts_process_inner(
            shaders_dir,
            shader_name,
            input_data,
            input_lens,
            extensions,
            image_count,
            param_names,
            param_values,
            param_count,
        )
    }));

    match result {
        Ok(Ok(outputs)) => Box::into_raw(Box::new(TsResult { outputs })),
        Ok(Err(e)) => {
            set_last_error(&e);
            std::ptr::null_mut()
        }
        Err(_) => {
            set_last_error("Panic inside ts_process");
            std::ptr::null_mut()
        }
    }
}

unsafe fn ts_process_inner(
    shaders_dir: *const c_char,
    shader_name: *const c_char,
    input_data: *const *const u8,
    input_lens: *const usize,
    extensions: *const *const c_char,
    image_count: u32,
    param_names: *const *const c_char,
    param_values: *const f32,
    param_count: u32,
) -> Result<Vec<(crate::porter_image::ImageBuffer, String)>, String> {
    let shaders_dir = unsafe { CStr::from_ptr(shaders_dir) }
        .to_str()
        .map_err(|_| "Invalid shaders_dir string".to_string())?;

    let shader_name = unsafe { CStr::from_ptr(shader_name) }
        .to_str()
        .map_err(|_| "Invalid shader_name string".to_string())?;

    // Load shaders from the given directory and find the requested one.
    let (shaders, _) = pollster::block_on(load_shaders_from_dir(Path::new(shaders_dir)))?;

    let shader = shaders
        .into_iter()
        .find(|s| s.shader.name == shader_name)
        .ok_or_else(|| format!("Shader '{shader_name}' not found in '{shaders_dir}'"))?;

    // Build PorterImages from raw file bytes.
    let mut images: Vec<Arc<PorterImage>> = Vec::with_capacity(image_count as usize);
    for i in 0..image_count as usize {
        let ptr = unsafe { *input_data.add(i) };
        let len = unsafe { *input_lens.add(i) };
        let ext = unsafe { CStr::from_ptr(*extensions.add(i)) }
            .to_str()
            .map_err(|_| format!("Invalid extension string for input {i}"))?;

        let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
        let img = PorterImage::from_bytes(bytes, ext)?;
        images.push(Arc::new(img));
    }

    // Build parameter map.
    let mut params: HashMap<String, f32> = HashMap::new();
    for i in 0..param_count as usize {
        let name = unsafe { CStr::from_ptr(*param_names.add(i)) }
            .to_str()
            .map_err(|_| format!("Invalid param name string at index {i}"))?;
        let value = unsafe { *param_values.add(i) };
        params.insert(name.to_string(), value);
    }

    pollster::block_on(process_images(images, shader, params))
}

/// Number of output images in a result.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_result_count(result: *const TsResult) -> u32 {
    unsafe { (*result).outputs.len() as u32 }
}

/// Pointer to the raw RGBA8 pixel bytes for output `index`.
/// Valid until `ts_result_free` is called.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_result_data(result: *const TsResult, index: u32) -> *const u8 {
    unsafe { (&(*result).outputs)[index as usize].0.as_raw().as_ptr() }
}

/// Byte length of the raw pixel data for output `index`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_result_len(result: *const TsResult, index: u32) -> usize {
    unsafe { (&(*result).outputs)[index as usize].0.as_raw().len() }
}

/// Width in pixels of output `index`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_result_width(result: *const TsResult, index: u32) -> u32 {
    unsafe { (&(*result).outputs)[index as usize].0.width() }
}

/// Height in pixels of output `index`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_result_height(result: *const TsResult, index: u32) -> u32 {
    unsafe { (&(*result).outputs)[index as usize].0.height() }
}

/// Save all outputs to `output_dir`.
///
/// - `format`: 0 = PNG, 1 = TGA, 2 = TIFF, 3 = DDS
/// - `dds_format`: index into `DdsSaveFormat::ALL` (ignored when `format != 3`)
///
/// Returns null on success, or an error string on failure.
/// Free the error string with `ts_free_string`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_result_save(
    result: *const TsResult,
    output_dir: *const c_char,
    format: u32,
    dds_format: u32,
) -> *mut c_char {
    let output_dir = match unsafe { CStr::from_ptr(output_dir) }.to_str() {
        Ok(s) => s,
        Err(_) => return make_error_string("Invalid output_dir string"),
    };

    let image_format = match format {
        0 => ImageFormat::Png,
        1 => ImageFormat::Tga,
        2 => ImageFormat::Tiff,
        3 => ImageFormat::Dds,
        _ => return make_error_string("Invalid format index (0=PNG 1=TGA 2=TIFF 3=DDS)"),
    };

    let dds_fmt = DdsSaveFormat::ALL
        .get(dds_format as usize)
        .copied()
        .unwrap_or_default();

    let outputs = unsafe { &(*result).outputs };
    match save_outputs(outputs, Path::new(output_dir), image_format, dds_fmt) {
        Ok(_) => std::ptr::null_mut(),
        Err(e) => make_error_string(&e),
    }
}

/// Save a single output at `index` to `output_dir`.
///
/// - `format`: 0 = PNG, 1 = TGA, 2 = TIFF, 3 = DDS
/// - `dds_format`: index into `DdsSaveFormat::ALL` (ignored when `format != 3`)
///
/// Returns null on success, or an error string on failure.
/// Free the error string with `ts_free_string`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_result_save_one(
    result: *const TsResult,
    index: u32,
    output_dir: *const c_char,
    format: u32,
    dds_format: u32,
) -> *mut c_char {
    let output_dir = match unsafe { CStr::from_ptr(output_dir) }.to_str() {
        Ok(s) => s,
        Err(_) => return make_error_string("Invalid output_dir string"),
    };

    let image_format = match format {
        0 => ImageFormat::Png,
        1 => ImageFormat::Tga,
        2 => ImageFormat::Tiff,
        3 => ImageFormat::Dds,
        _ => return make_error_string("Invalid format index (0=PNG 1=TGA 2=TIFF 3=DDS)"),
    };

    let dds_fmt = DdsSaveFormat::ALL
        .get(dds_format as usize)
        .copied()
        .unwrap_or_default();

    let outputs = unsafe { &(*result).outputs };
    let Some((buffer, description)) = outputs.get(index as usize) else {
        return make_error_string("Output index out of range");
    };

    let filename = format!(
        "{}.{}",
        description.to_lowercase().replace([' ', '/', '\\'], "_"),
        image_format.extension()
    );

    match save_buffer(buffer, &Path::new(output_dir).join(&filename), image_format, dds_fmt) {
        Ok(_) => std::ptr::null_mut(),
        Err(e) => make_error_string(&e),
    }
}

/// Free a `TsResult` returned by `ts_process`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_result_free(result: *mut TsResult) {
    if !result.is_null() {
        unsafe { drop(Box::from_raw(result)) };
    }
}

/// Free a string returned by this library (e.g. from `ts_result_save`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe { drop(CString::from_raw(ptr)) };
    }
}

/// Returns the last error message set by this library, or null if there is none.
/// The returned pointer is valid until the next library call on this thread.
#[unsafe(no_mangle)]
pub extern "C" fn ts_last_error() -> *const c_char {
    LAST_ERROR.with(|cell| {
        cell.borrow()
            .as_ref()
            .map(|s| s.as_ptr())
            .unwrap_or(std::ptr::null())
    })
}

// ── Internals ────────────────────────────────────────────────────────────────

std::thread_local! {
    static LAST_ERROR: std::cell::RefCell<Option<CString>> = const { std::cell::RefCell::new(None) };
}

fn set_last_error(msg: &str) {
    LAST_ERROR.with(|cell| {
        *cell.borrow_mut() = CString::new(msg).ok();
    });
}

fn make_error_string(msg: &str) -> *mut c_char {
    CString::new(msg)
        .map(CString::into_raw)
        .unwrap_or(std::ptr::null_mut())
}
