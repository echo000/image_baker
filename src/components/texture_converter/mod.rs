//! Texture Converter Module
//!
//! A modular texture processing component that uses GPU shaders to transform images.
//! Organized into focused submodules for maintainability:
//! - `gpu_processor`: GPU shader execution and rendering
//! - `shader_manager`: Shader loading and validation
//! - `state`: Component state management with caching
//! - `types`: Error types and type aliases

mod gpu_processor;
mod shader_manager;
mod state;
mod types;

// Re-export public items
pub use gpu_processor::process_images;
pub use shader_manager::load_shaders;
pub use state::TextureConverterState;
pub use types::ImageFormat;

// Keep original types here for compatibility
use crate::components::droppable_image_slot::DroppableImageSlot;
use crate::messages::Message;
use crate::porter_image::{ImageBuffer, PorterImage};
use crate::status::StatusMessage;
use iced::widget::{button, column, container, pick_list, row, text};
use iced::{Element, Length, Task};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

// Constants for UI sizing and timing
/// Debounce delay for parameter changes (milliseconds)
const PARAMETER_DEBOUNCE_MS: u64 = 150;

/// Maximum image dimension supported
#[allow(dead_code)]
const MAX_IMAGE_DIMENSION: u32 = 8192;

/// Size for single input slot preview
const SLOT_SIZE_SINGLE: f32 = 280.0;
/// Size for two input slots preview
const SLOT_SIZE_TWO: f32 = 240.0;
/// Size for three input slots preview
const SLOT_SIZE_THREE: f32 = 220.0;
/// Size for four input slots preview
const SLOT_SIZE_FOUR: f32 = 200.0;
/// Size for five or more input slots preview
const SLOT_SIZE_MANY: f32 = 180.0;

// Shared vertex shader for all texture processing operations
pub const FULLSCREEN_QUAD_VERTEX_SHADER: &[u8] =
    include_bytes!("../../shaders/fullscreen_quad_vertex.wgsl");

/// Texture converter component
pub struct TextureSplitter {
    state: TextureConverterState,
    selected_format: ImageFormat,
}

/// Messages produced by the texture splitter component
#[derive(Debug, Clone)]
pub enum TextureSplitterMessage {
    ShaderSelected(String),
    ShadersLoaded(Result<(Vec<ShaderConfig>, usize), String>),
    ParameterChanged(String, f32),  // (parameter_name, value)
    DebouncedParameterProcess(u64), // Process parameters after debounce (generation)
    BrowseInput(usize),             // Browse for input slot at index
    InputFileSelected(usize, Option<PathBuf>), // Input slot index, path
    InputImageLoaded(usize, Result<PorterImage, String>), // Input slot index, image
    MergeCompleted(Result<Vec<(ImageBuffer, String)>, String>, u64), // Result (outputs with descriptions), generation
    SaveAllPressed,
    FormatSelected(ImageFormat),
    ClearPressed,
    AllImagesSaved(Result<Vec<PathBuf>, String>),
    NextOutput,
    PreviousOutput,
    ReloadShaders,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderConfig {
    pub shader: ShaderMetadata,
    #[serde(default)]
    pub inputs: Vec<InputConfig>,
    pub outputs: Vec<OutputConfig>,
    #[serde(default)]
    pub parameters: Vec<ShaderParameter>,
    #[serde(skip)]
    pub shader_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderMetadata {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    pub suffix: String,
    pub description: String,
    #[serde(default = "default_true")]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub entry_point: String,
    pub suffix: String,
    pub description: String,
    #[serde(default = "default_format")]
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub default: f32,
    pub min: f32,
    pub max: f32,
    pub description: String,
}

fn default_true() -> bool {
    true
}

fn default_format() -> String {
    "Rgba8Unorm".to_string()
}

impl std::fmt::Display for ShaderConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.shader.name)
    }
}

impl PartialEq for ShaderConfig {
    fn eq(&self, other: &Self) -> bool {
        self.shader.name == other.shader.name
    }
}

impl Eq for ShaderConfig {}

impl TextureSplitter {
    /// Creates a new texture splitter component
    pub fn new() -> Self {
        Self {
            state: TextureConverterState::new(),
            selected_format: ImageFormat::default(),
        }
    }

    /// Initialize the texture splitter by loading shaders
    pub fn initialize() -> Task<Message> {
        Task::perform(load_shaders(), |result| {
            Message::Main(crate::windows::MainMessage::TextureSplitter(
                TextureSplitterMessage::ShadersLoaded(result),
            ))
        })
    }

    /// Update the texture splitter state based on messages
    pub fn update(&mut self, message: TextureSplitterMessage) -> Task<Message> {
        match message {
            TextureSplitterMessage::ShaderSelected(name) => self.on_shader_selected(name),
            TextureSplitterMessage::ShadersLoaded(result) => self.on_shaders_loaded(result),
            TextureSplitterMessage::ParameterChanged(param_name, value) => {
                if let Some(shader_name) = &self.state.selected_shader {
                    if let Some(param_map) = self.state.parameter_values.get_mut(shader_name) {
                        param_map.insert(param_name, value);
                    }
                    // Increment debounce generation and schedule delayed processing
                    self.state.parameter_debounce_generation += 1;
                    let generation = self.state.parameter_debounce_generation;

                    // Wait before processing (debounce)
                    return Task::perform(
                        async move {
                            futures_timer::Delay::new(std::time::Duration::from_millis(
                                PARAMETER_DEBOUNCE_MS,
                            ))
                            .await;
                            generation
                        },
                        |debounce_gen| {
                            Message::Main(crate::windows::MainMessage::TextureSplitter(
                                TextureSplitterMessage::DebouncedParameterProcess(debounce_gen),
                            ))
                        },
                    );
                }
                Task::none()
            }
            TextureSplitterMessage::DebouncedParameterProcess(generation) => {
                // Only process if this is still the latest parameter change
                if generation == self.state.parameter_debounce_generation
                    && self.state.all_required_slots_filled()
                {
                    return self.trigger_merge_from_slots();
                }
                Task::none()
            }
            TextureSplitterMessage::BrowseInput(slot_idx) => self.on_browse_input(slot_idx),
            TextureSplitterMessage::InputFileSelected(slot_idx, path_opt) => {
                self.on_input_file_selected(slot_idx, path_opt)
            }
            TextureSplitterMessage::InputImageLoaded(slot_idx, result) => {
                self.on_input_image_loaded(slot_idx, result)
            }
            TextureSplitterMessage::MergeCompleted(result, generation) => {
                self.on_merge_completed(result, generation)
            }
            TextureSplitterMessage::SaveAllPressed => self.on_save_all(),
            TextureSplitterMessage::FormatSelected(format) => {
                self.selected_format = format;
                Task::none()
            }
            TextureSplitterMessage::ClearPressed => self.on_clear(),
            TextureSplitterMessage::AllImagesSaved(result) => self.on_all_images_saved(result),
            TextureSplitterMessage::NextOutput => {
                self.state.next_output();
                Task::none()
            }
            TextureSplitterMessage::PreviousOutput => {
                self.state.previous_output();
                Task::none()
            }
            TextureSplitterMessage::ReloadShaders => {
                self.state.shaders_loading = true;
                self.state.status = StatusMessage::info("Reloading shaders...");
                Task::perform(load_shaders(), |result| {
                    Message::Main(crate::windows::MainMessage::TextureSplitter(
                        TextureSplitterMessage::ShadersLoaded(result),
                    ))
                })
            }
        }
    }

    /// Render the texture splitter UI
    pub fn view(&self) -> Element<'_, TextureSplitterMessage> {
        use crate::components::baker_layout::*;
        use crate::widget_helpers::pick_list_style;

        let status_bar = text(&self.state.status.message)
            .size(12)
            .color(self.state.status.colour());

        // Shader picker
        let shader_label = text("Select Shader:").size(16);
        let shader_picker = if !self.state.shaders.is_empty() {
            pick_list(
                self.state.shaders.as_slice(),
                self.state
                    .selected_shader
                    .as_ref()
                    .and_then(|name| self.state.shaders.iter().find(|s| s.shader.name == *name)),
                |shader: ShaderConfig| {
                    TextureSplitterMessage::ShaderSelected(shader.shader.name.clone())
                },
            )
            .placeholder("Select a shader...")
            .style(pick_list_style)
        } else {
            pick_list(
                &[] as &[ShaderConfig],
                None::<&ShaderConfig>,
                |_: ShaderConfig| TextureSplitterMessage::ShaderSelected(String::new()),
            )
            .placeholder("No shaders available")
            .style(pick_list_style)
        };

        let reload_button = button("Reload Shaders")
            .on_press(TextureSplitterMessage::ReloadShaders)
            .padding(8)
            .style(crate::widget_helpers::primary_button_style);

        let shader_picker_section = row![
            column![shader_label, shader_picker]
                .spacing(8)
                .align_x(iced::Alignment::Start),
            container(reload_button)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Right)
        ]
        .spacing(20)
        .padding(20)
        .width(Length::Fill)
        .align_y(iced::Alignment::Center);

        // Render input slots using cached handles
        let mut input_slot_views = Vec::new();

        for (idx, slot) in self.state.input_slots.iter().enumerate() {
            let slot_view = self.view_cached_slot(idx, slot);
            input_slot_views.push(slot_view);
        }

        // Output preview with navigation
        let output_widget = if !self.state.outputs.is_empty() {
            let current_handle = &self.state.outputs[self.state.current_output_index];
            let current_desc = &self.state.output_descriptions[self.state.current_output_index];

            // Create navigation info text
            let nav_text = if self.state.outputs.len() > 1 {
                text(format!(
                    "{} ({}/{})",
                    current_desc,
                    self.state.current_output_index + 1,
                    self.state.outputs.len()
                ))
                .size(14)
            } else {
                text(current_desc.clone()).size(14)
            };

            // Create navigation buttons
            let prev_button = button("<")
                .on_press_maybe(if self.state.current_output_index > 0 {
                    Some(TextureSplitterMessage::PreviousOutput)
                } else {
                    None
                })
                .padding(8)
                .style(crate::widget_helpers::primary_button_style);

            let next_button = button(">")
                .on_press_maybe(
                    if self.state.current_output_index < self.state.outputs.len() - 1 {
                        Some(TextureSplitterMessage::NextOutput)
                    } else {
                        None
                    },
                )
                .padding(8)
                .style(crate::widget_helpers::primary_button_style);

            // Build preview with navigation
            let preview = container(
                iced::widget::image(current_handle.clone())
                    .content_fit(iced::ContentFit::Contain)
                    .width(400)
                    .height(400),
            )
            .width(400)
            .height(400)
            .center_x(Length::Fill);

            if self.state.outputs.len() > 1 {
                column![
                    nav_text,
                    row![
                        prev_button,
                        container(preview)
                            .width(Length::Fill)
                            .center_x(Length::Fill),
                        next_button
                    ]
                    .spacing(10)
                    .align_y(iced::Alignment::Center)
                ]
                .spacing(8)
                .align_x(iced::Alignment::Center)
                .into()
            } else {
                column![nav_text, preview]
                    .spacing(8)
                    .align_x(iced::Alignment::Center)
                    .into()
            }
        } else {
            create_output_preview(&None, "Output will appear here")
        };

        // Create parameter sliders
        let mut controls = Vec::new();
        if let Some(shader) = self.state.get_selected_shader() {
            let shader_name = shader.shader.name.clone();
            let param_values = self.state.parameter_values.get(&shader_name);

            for param in &shader.parameters {
                let current_value = param_values
                    .and_then(|map| map.get(&param.name))
                    .copied()
                    .unwrap_or(param.default);

                let param_name = param.name.clone();
                let param_slider = create_slider_control(
                    &param.description,
                    current_value as f64,
                    param.min as f64..=param.max as f64,
                    move |val| {
                        TextureSplitterMessage::ParameterChanged(param_name.clone(), val as f32)
                    },
                );
                controls.push(param_slider);
            }
        }

        // Create action buttons with format selector (always visible)
        let mut buttons = Vec::new();

        // Format selector and save button (always visible)
        let format_selector = pick_list(
            &ImageFormat::ALL[..],
            Some(self.selected_format),
            TextureSplitterMessage::FormatSelected,
        )
        .padding(12)
        .placeholder("Format")
        .style(pick_list_style);

        let save_all_button = create_save_all_button(
            self.state.is_saving,
            !self.state.output_buffers.is_empty(),
            TextureSplitterMessage::SaveAllPressed,
        );

        let save_row = row![
            container(format_selector).width(Length::FillPortion(1)),
            container(save_all_button).width(Length::FillPortion(2)),
        ]
        .spacing(10)
        .width(Length::Fill);

        buttons.push(save_row.into());

        let clear_button = create_clear_button(TextureSplitterMessage::ClearPressed);
        buttons.push(clear_button);

        // Build main layout using baker_layout
        let baker_content = create_baker_layout(BakerLayoutConfig {
            input_slots: input_slot_views,
            output_widget,
            controls,
            buttons,
            status_bar: status_bar.into(),
        });

        // Combine shader picker at top with baker layout below
        column![shader_picker_section, baker_content]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Handle file dropped onto the window
    ///
    /// Loads the dropped file into the first available empty slot.
    /// If all slots are filled, displays a warning message.
    pub fn on_file_dropped(&mut self, path: PathBuf) -> Task<Message> {
        // Load into the first empty slot
        for (idx, slot) in self.state.input_slots.iter().enumerate() {
            if slot.image.is_none() {
                return self.on_input_file_selected(idx, Some(path));
            }
        }

        // If all slots are filled, show a message
        self.state.status =
            StatusMessage::warning("All input slots are filled. Clear to load new images.");
        Task::none()
    }

    /// Handle shaders loaded result
    ///
    /// Initializes the first shader's parameters and input slots on success.
    /// Updates status message with load results.
    fn on_shaders_loaded(
        &mut self,
        result: Result<(Vec<ShaderConfig>, usize), String>,
    ) -> Task<Message> {
        match result {
            Ok((shaders, failed_count)) => {
                if shaders.is_empty() {
                    self.state.status = StatusMessage::warning(
                        "No shaders found! Add shaders to the 'shaders' directory.",
                    );
                } else {
                    let first_shader_name = shaders[0].shader.name.clone();
                    self.state.selected_shader = Some(first_shader_name.clone());

                    // Initialize the first shader's parameters and input slots
                    if let Some(shader) =
                        shaders.iter().find(|s| s.shader.name == first_shader_name)
                    {
                        self.state
                            .initialize_parameters(&first_shader_name, &shader.parameters);
                        self.state.initialize_input_slots(shader);
                    }

                    if failed_count > 0 {
                        self.state.status = StatusMessage::warning(format!(
                            "{} shader{} loaded, {} failed. Check image_baker.log for details.",
                            shaders.len(),
                            if shaders.len() == 1 { "" } else { "s" },
                            failed_count
                        ));
                    } else {
                        self.state.status = StatusMessage::info(format!(
                            "Ready. Loaded {} shader{}. Drag and drop a texture.",
                            shaders.len(),
                            if shaders.len() == 1 { "" } else { "s" }
                        ));
                    }
                }
                self.state.shaders = shaders;
                self.state.shaders_loading = false;
            }
            Err(e) => {
                self.state.status = StatusMessage::error(format!("Error loading shaders: {e}"));
                self.state.shaders_loading = false;
            }
        }
        Task::none()
    }

    /// Handle shader selection change
    ///
    /// Initializes parameter values and input slots for the selected shader.
    /// Clears existing outputs when switching shaders.
    fn on_shader_selected(&mut self, name: String) -> Task<Message> {
        self.state.selected_shader = Some(name.clone());

        // Initialize parameter values and input slots
        if let Some(shader) = self.state.get_selected_shader() {
            self.state.initialize_parameters(&name, &shader.parameters);
            self.state.initialize_input_slots(&shader);

            // Clear outputs when switching shaders
            self.state.clear_outputs();
        }

        Task::none()
    }

    /// Open file browser for selecting input image
    ///
    /// Opens an async file dialog filtered for common image formats.
    fn on_browse_input(&mut self, slot_idx: usize) -> Task<Message> {
        Task::perform(
            async move {
                rfd::AsyncFileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg", "tga", "dds", "tiff"])
                    .pick_file()
                    .await
                    .map(|handle| handle.path().to_path_buf())
            },
            move |path_opt| {
                Message::Main(crate::windows::MainMessage::TextureSplitter(
                    TextureSplitterMessage::InputFileSelected(slot_idx, path_opt),
                ))
            },
        )
    }

    /// Handle file selection from browser
    ///
    /// Loads the selected image file and updates the corresponding input slot.
    fn on_input_file_selected(
        &mut self,
        slot_idx: usize,
        path_opt: Option<PathBuf>,
    ) -> Task<Message> {
        if let Some(path) = path_opt {
            self.state.status =
                StatusMessage::info(format!("Loading image for slot {slot_idx}..."));

            Task::perform(
                async move {
                    match PorterImage::open(&path) {
                        Ok(img) => Ok(img),
                        Err(e) => Err(format!("Failed to load image: {e}")),
                    }
                },
                move |result| {
                    Message::Main(crate::windows::MainMessage::TextureSplitter(
                        TextureSplitterMessage::InputImageLoaded(slot_idx, result),
                    ))
                },
            )
        } else {
            Task::none()
        }
    }

    /// Handle loaded image result
    ///
    /// Updates the input slot with the loaded image and triggers processing
    /// if all required slots are filled.
    fn on_input_image_loaded(
        &mut self,
        slot_idx: usize,
        result: Result<PorterImage, String>,
    ) -> Task<Message> {
        match result {
            Ok(img) => {
                if slot_idx < self.state.input_slots.len() {
                    self.state.input_slots[slot_idx].load_image(img);
                    // Update cached handle for this slot
                    self.state.update_input_slot_handle(slot_idx);
                    self.state.status =
                        StatusMessage::success(format!("Loaded image for slot {slot_idx}"));

                    // If all required slots are filled, trigger merge
                    if self.state.all_required_slots_filled() {
                        return self.trigger_merge_from_slots();
                    }
                }
            }
            Err(e) => {
                self.state.status = StatusMessage::error(e);
            }
        }
        Task::none()
    }

    /// Trigger GPU processing with current input images
    ///
    /// Collects images from all slots and processes them using the selected shader
    /// with current parameter values. Uses generation counter to handle concurrent requests.
    fn trigger_merge_from_slots(&mut self) -> Task<Message> {
        if !self.state.all_required_slots_filled() {
            return Task::none();
        }

        if let Some(shader) = self.state.get_selected_shader() {
            // Collect images from slots
            let mut images: Vec<Arc<PorterImage>> = Vec::new();
            for slot in &self.state.input_slots {
                if let Some(img) = &slot.image {
                    images.push(Arc::clone(img));
                }
            }

            if images.is_empty() {
                return Task::none();
            }

            // Get parameters
            let shader_name = shader.shader.name.clone();
            let param_values = self
                .state
                .parameter_values
                .get(&shader_name)
                .cloned()
                .unwrap_or_default();

            self.state.processing = true;
            self.state.merge_generation += 1;
            let generation = self.state.merge_generation;
            self.state.status = StatusMessage::info("Processing...");

            Task::perform(
                process_images(images, shader, param_values),
                move |result| {
                    Message::Main(crate::windows::MainMessage::TextureSplitter(
                        TextureSplitterMessage::MergeCompleted(result, generation),
                    ))
                },
            )
        } else {
            Task::none()
        }
    }

    /// Save all output images to a selected folder
    ///
    /// Opens folder picker and saves all outputs with auto-generated filenames
    /// using the currently selected image format.
    fn on_save_all(&mut self) -> Task<Message> {
        if !self.state.is_saving && !self.state.output_buffers.is_empty() {
            self.state.is_saving = true;
            self.state.status = StatusMessage::info("Saving all outputs...");

            // Clone all output buffers and descriptions
            let outputs: Vec<(ImageBuffer, String)> = self
                .state
                .output_buffers
                .iter()
                .zip(self.state.output_descriptions.iter())
                .map(|(buffer, desc)| (buffer.clone(), desc.clone()))
                .collect();

            let format = self.selected_format;

            Task::perform(
                async move {
                    let folder = rfd::AsyncFileDialog::new()
                        .set_title("Select folder to save all outputs")
                        .pick_folder()
                        .await;

                    if let Some(folder_handle) = folder {
                        let folder_path = folder_handle.path().to_path_buf();
                        let mut saved_paths = Vec::new();

                        for (buffer, description) in outputs {
                            let filename = format!(
                                "{}.{}",
                                description
                                    .to_lowercase()
                                    .replace(" ", "_")
                                    .replace("/", "_")
                                    .replace("\\", "_"),
                                format.extension()
                            );

                            let file_path = folder_path.join(&filename);

                            std::fs::create_dir_all(file_path.parent().unwrap_or(&file_path))
                                .map_err(|e| format!("Failed to create directory: {e}"))?;

                            match buffer.into_porter_image() {
                                Ok(mut img) => match img.save(&file_path) {
                                    Ok(_) => {
                                        saved_paths.push(file_path);
                                    }
                                    Err(e) => {
                                        return Err(format!("Failed to save {filename}: {e}"));
                                    }
                                },
                                Err(e) => {
                                    return Err(format!(
                                        "Failed to convert buffer for {filename}: {e}"
                                    ));
                                }
                            }
                        }

                        Ok(saved_paths)
                    } else {
                        Err("Save cancelled".to_string())
                    }
                },
                |result| {
                    Message::Main(crate::windows::MainMessage::TextureSplitter(
                        TextureSplitterMessage::AllImagesSaved(result),
                    ))
                },
            )
        } else {
            Task::none()
        }
    }

    /// Clear all input images and outputs
    ///
    /// Resets the component to its initial state, clearing all loaded images
    /// and processing results.
    fn on_clear(&mut self) -> Task<Message> {
        self.state.clear_inputs();
        self.state.clear_outputs();
        self.state.status = StatusMessage::info("Cleared all images.");
        tracing::info!("Cleared all loaded images");
        Task::none()
    }

    /// Handle save all completion
    ///
    /// Updates status with success/failure message after batch save operation.
    fn on_all_images_saved(&mut self, result: Result<Vec<PathBuf>, String>) -> Task<Message> {
        self.state.is_saving = false;
        match result {
            Ok(paths) => {
                self.state.status =
                    StatusMessage::success(format!("Saved {} output(s) successfully", paths.len()));
                for path in &paths {
                    tracing::info!("Saved output to: {}", path.display());
                }
            }
            Err(e) => {
                self.state.status = StatusMessage::error(format!("Save all failed: {e}"));
                tracing::error!("Save all failed: {}", e);
            }
        }
        Task::none()
    }

    /// Render an input slot using cached image handle
    ///
    /// Uses cached image handles to avoid regenerating them every frame.
    /// Slot size is responsive based on the total number of input slots.
    fn view_cached_slot<'a>(
        &'a self,
        idx: usize,
        slot: &'a DroppableImageSlot,
    ) -> Element<'a, TextureSplitterMessage> {
        use crate::widget_helpers::{control, primary_button_style};
        use iced::widget::{button, column, container};

        // Make slot size responsive based on number of inputs
        let preview_size = match self.state.input_slots.len() {
            1 => SLOT_SIZE_SINGLE,
            2 => SLOT_SIZE_TWO,
            3 => SLOT_SIZE_THREE,
            4 => SLOT_SIZE_FOUR,
            _ => SLOT_SIZE_MANY,
        };

        // Create the image preview using cached handle or placeholder
        let image_widget = if let Some(cached_handle) = self.state.get_input_slot_handle(idx) {
            // Use cached handle - no conversion needed!
            container(
                iced::widget::image(cached_handle.clone())
                    .content_fit(iced::ContentFit::Cover)
                    .width(preview_size)
                    .height(preview_size),
            )
            .width(preview_size)
            .height(preview_size)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
        } else {
            // Show placeholder
            container(
                text("Drop file or Browse")
                    .size(14)
                    .align_x(iced::alignment::Horizontal::Center),
            )
            .width(preview_size)
            .height(preview_size)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
        };

        // Build the complete slot with label and browse button
        let browse_button = button("Browse...")
            .on_press(TextureSplitterMessage::BrowseInput(idx))
            .padding(8)
            .width(preview_size)
            .style(primary_button_style);

        let col = column![image_widget, browse_button]
            .spacing(8)
            .align_x(iced::Alignment::Center);

        control(text(&slot.label).size(13).into(), col.into()).into()
    }

    /// Handle GPU processing completion
    ///
    /// Updates state with processing results and converts output buffers to
    /// displayable image handles. Uses generation counter to ignore stale results.
    fn on_merge_completed(
        &mut self,
        result: Result<Vec<(ImageBuffer, String)>, String>,
        generation: u64,
    ) -> Task<Message> {
        if generation != self.state.merge_generation {
            return Task::none();
        }

        self.state.processing = false;

        match result {
            Ok(outputs) => {
                self.state.set_outputs(outputs);

                if self.state.outputs.len() > 1 {
                    self.state.status = StatusMessage::success(format!(
                        "Processing complete - {} outputs generated",
                        self.state.outputs.len()
                    ));
                } else {
                    self.state.status = StatusMessage::success("Processing complete");
                }
            }
            Err(e) => {
                self.state.status = StatusMessage::error(format!("Processing failed: {e}"));
            }
        }

        Task::none()
    }
}

impl Default for TextureSplitter {
    fn default() -> Self {
        Self::new()
    }
}
