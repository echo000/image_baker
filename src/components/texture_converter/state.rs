//! State Management Module
//!
//! Manages all state for the texture converter component, including:
//! - Shader selection and parameters
//! - Input image slots with cached handles
//! - Output buffers and previews
//! - Processing state and generation counters
//!
//! The state module implements caching strategies to avoid redundant
//! work during UI rendering.

use crate::components::droppable_image_slot::DroppableImageSlot;
use crate::components::texture_converter::{ShaderConfig, ShaderParameter};
use crate::porter_image::ImageBuffer;
use crate::status::StatusMessage;
use std::collections::HashMap;

/// State for the texture converter component with cached image handles
///
/// This struct maintains all component state including:
/// - Available shaders and current selection
/// - Parameter values for each shader
/// - Input slots with cached image handles (performance optimization)
/// - Output buffers and display handles
/// - Debouncing counters for parameter changes
pub struct TextureConverterState {
    pub status: StatusMessage,
    pub processing: bool,
    pub shaders: Vec<ShaderConfig>,
    pub selected_shader: Option<String>,
    pub shaders_loading: bool,

    // Store parameter values: shader_name -> (parameter_name -> value)
    pub parameter_values: HashMap<String, HashMap<String, f32>>,

    // Input slots for all shaders
    pub input_slots: Vec<DroppableImageSlot>,

    // Cached handles for input slots to avoid regenerating every frame
    pub input_slot_handles: Vec<Option<iced::widget::image::Handle>>,
    pub input_slot_generations: Vec<u64>,

    // Outputs for preview (multiple outputs per shader)
    pub outputs: Vec<iced::widget::image::Handle>,
    pub output_buffers: Vec<ImageBuffer>,
    pub output_descriptions: Vec<String>,
    pub current_output_index: usize,
    pub merge_generation: u64,
    pub is_saving: bool,

    // Debouncing for parameter changes
    pub parameter_debounce_generation: u64,
}

impl TextureConverterState {
    /// Create a new texture converter state with default values
    ///
    /// Initializes empty collections and sets status to "Ready".
    pub fn new() -> Self {
        Self {
            status: StatusMessage::info("Ready. Select a shader and load images."),
            processing: false,
            shaders: Vec::new(),
            selected_shader: None,
            shaders_loading: false,
            parameter_values: HashMap::new(),
            input_slots: Vec::new(),
            input_slot_handles: Vec::new(),
            input_slot_generations: Vec::new(),
            outputs: Vec::new(),
            output_buffers: Vec::new(),
            output_descriptions: Vec::new(),
            current_output_index: 0,
            merge_generation: 0,
            is_saving: false,
            parameter_debounce_generation: 0,
        }
    }

    /// Get the currently selected shader configuration
    ///
    /// Searches for the shader matching the selected name.
    pub fn get_selected_shader(&self) -> Option<ShaderConfig> {
        self.selected_shader.as_ref().and_then(|name| {
            self.shaders
                .iter()
                .find(|s| s.shader.name == *name)
                .cloned()
        })
    }

    /// Check if all required input slots are filled
    ///
    /// Returns true only if every required input has an image loaded.
    /// Optional inputs are not checked.
    pub fn all_required_slots_filled(&self) -> bool {
        if let Some(shader) = self.get_selected_shader() {
            for (idx, input_config) in shader.inputs.iter().enumerate() {
                if input_config.required
                    && idx < self.input_slots.len()
                    && self.input_slots[idx].image.is_none()
                {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    /// Initialize input slots based on shader configuration
    ///
    /// Clears existing slots and creates new ones for each shader input.
    /// Also clears all cached handles and generation counters.
    pub fn initialize_input_slots(&mut self, shader: &ShaderConfig) {
        self.input_slots.clear();
        self.input_slot_handles.clear();
        self.input_slot_generations.clear();

        for input_config in &shader.inputs {
            self.input_slots
                .push(DroppableImageSlot::new(&input_config.description));
            self.input_slot_handles.push(None);
            self.input_slot_generations.push(0);
        }
    }

    /// Initialize parameter values for a shader
    ///
    /// Sets default values for parameters that haven't been set yet.
    /// Preserves existing parameter values if already configured.
    pub fn initialize_parameters(&mut self, shader_name: &str, parameters: &[ShaderParameter]) {
        let param_map = self
            .parameter_values
            .entry(shader_name.to_string())
            .or_default();

        for param in parameters {
            param_map.entry(param.name.clone()).or_insert(param.default);
        }
    }

    /// Update cached handle for an input slot
    ///
    /// Regenerates the image handle for the specified slot and stores it
    /// in the cache. This should be called when an image is loaded or changed.
    /// Increments the generation counter for the slot.
    pub fn update_input_slot_handle(&mut self, slot_idx: usize) {
        if slot_idx >= self.input_slots.len() {
            return;
        }

        if let Some(img) = &self.input_slots[slot_idx].image {
            let (w, h) = img.dimensions();
            if w > 0 && h > 0 && w <= 8192 && h <= 8192 {
                let mut img_clone = (**img).clone();
                let handle = crate::core_logic::image_to_handle(&mut img_clone);

                // Ensure vectors are large enough
                while self.input_slot_handles.len() <= slot_idx {
                    self.input_slot_handles.push(None);
                }
                while self.input_slot_generations.len() <= slot_idx {
                    self.input_slot_generations.push(0);
                }

                self.input_slot_handles[slot_idx] = Some(handle);
                self.input_slot_generations[slot_idx] += 1;
            }
        } else {
            // Clear the handle if no image
            if slot_idx < self.input_slot_handles.len() {
                self.input_slot_handles[slot_idx] = None;
            }
        }
    }

    /// Get cached handle for an input slot, or None if not available
    ///
    /// Returns a reference to the cached handle without regenerating it.
    /// Used during rendering to avoid redundant image conversions.
    pub fn get_input_slot_handle(&self, slot_idx: usize) -> Option<&iced::widget::image::Handle> {
        self.input_slot_handles
            .get(slot_idx)
            .and_then(|h| h.as_ref())
    }

    /// Clear all input slots and cached handles
    ///
    /// Removes all loaded images and resets caches while preserving
    /// the slot structure for the current shader.
    pub fn clear_inputs(&mut self) {
        for slot in &mut self.input_slots {
            slot.clear();
        }
        self.input_slot_handles.clear();
        self.input_slot_generations.clear();

        // Recreate empty caches
        for _ in 0..self.input_slots.len() {
            self.input_slot_handles.push(None);
            self.input_slot_generations.push(0);
        }
    }

    /// Clear all outputs
    ///
    /// Removes all output buffers, handles, and descriptions.
    /// Resets the output index to 0.
    pub fn clear_outputs(&mut self) {
        self.outputs.clear();
        self.output_buffers.clear();
        self.output_descriptions.clear();
        self.current_output_index = 0;
    }

    /// Navigate to next output
    ///
    /// Increments the output index if not already at the last output.
    pub fn next_output(&mut self) {
        if self.current_output_index < self.outputs.len().saturating_sub(1) {
            self.current_output_index += 1;
        }
    }

    /// Navigate to previous output
    ///
    /// Decrements the output index if not already at the first output.
    pub fn previous_output(&mut self) {
        if self.current_output_index > 0 {
            self.current_output_index -= 1;
        }
    }

    /// Set outputs from processing results
    ///
    /// Converts output buffers to displayable handles and stores them.
    /// Clears any existing outputs and resets the index to 0.
    pub fn set_outputs(&mut self, outputs: Vec<(ImageBuffer, String)>) {
        self.outputs.clear();
        self.output_buffers.clear();
        self.output_descriptions.clear();

        for (buffer, description) in outputs {
            let handle = crate::core_logic::buffer_to_handle(&buffer);
            self.outputs.push(handle);
            self.output_buffers.push(buffer);
            self.output_descriptions.push(description);
        }

        self.current_output_index = 0;
    }

    /// Get the current output buffer for saving
    #[allow(dead_code)]
    pub fn get_current_output(&self) -> Option<(&ImageBuffer, &str)> {
        if self.current_output_index < self.output_buffers.len() {
            Some((
                &self.output_buffers[self.current_output_index],
                &self.output_descriptions[self.current_output_index],
            ))
        } else {
            None
        }
    }
}

impl Default for TextureConverterState {
    fn default() -> Self {
        Self::new()
    }
}
