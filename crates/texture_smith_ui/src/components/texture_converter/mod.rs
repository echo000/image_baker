//! Texture Converter Module
//!
//! A modular texture processing component that uses GPU shaders to transform images.
//! Core processing logic is provided by texture_smith_core.
//! - `state`: Component state management with caching

mod state;

// Re-export state
pub use state::TextureConverterState;

// Re-export from core for backward compat
pub use texture_smith_core::{DdsSaveFormat, ImageFormat};

// Import types from core crate
use texture_smith_core::gpu_processor::process_images;
use texture_smith_core::shader_manager::load_shaders;
use texture_smith_core::{ImageBuffer, PorterImage, ShaderConfig};

// Keep original UI imports
use crate::components::droppable_image_slot::DroppableImageSlot;
use crate::messages::Message;
use crate::status::StatusMessage;
use iced::widget::{button, column, container, pick_list, row, text};
use iced::{Element, Length, Task};
use std::path::PathBuf;
use std::sync::Arc;

// Constants for UI sizing and timing
/// Debounce delay for parameter changes (milliseconds)
const PARAMETER_DEBOUNCE_MS: u64 = 150;

/// Maximum image dimension supported
#[allow(dead_code)]
const MAX_IMAGE_DIMENSION: u32 = 8192;

/// Texture converter component
pub struct TextureSplitter {
    state: TextureConverterState,
    selected_format: ImageFormat,
    selected_dds_format: DdsSaveFormat,
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
    InputImageLoaded(usize, Result<PorterImage, String>, Option<PathBuf>), // Input slot index, image, source path
    MergeCompleted(Result<Vec<(ImageBuffer, String)>, String>, u64), // Result (outputs with descriptions), generation
    SaveAllPressed,
    FormatSelected(ImageFormat),
    DdsFormatSelected(DdsSaveFormat),
    ClearPressed,
    AllImagesSaved(Result<Vec<PathBuf>, String>),
    NextOutput,
    PreviousOutput,
    ReloadShaders,
}

impl TextureSplitter {
    /// Creates a new texture splitter component
    pub fn new() -> Self {
        Self {
            state: TextureConverterState::new(),
            selected_format: ImageFormat::default(),
            selected_dds_format: DdsSaveFormat::default(),
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
            TextureSplitterMessage::InputImageLoaded(slot_idx, result, path) => {
                self.on_input_image_loaded(slot_idx, result, path)
            }
            TextureSplitterMessage::MergeCompleted(result, generation) => {
                self.on_merge_completed(result, generation)
            }
            TextureSplitterMessage::SaveAllPressed => self.on_save_all(),
            TextureSplitterMessage::FormatSelected(format) => {
                self.selected_format = format;
                Task::none()
            }
            TextureSplitterMessage::DdsFormatSelected(dds_format) => {
                self.selected_dds_format = dds_format;
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

        // ── Status bar ────────────────────────────────────────────────────────
        let status_bar = row![
            text(&self.state.status.message)
                .size(13)
                .color(self.state.status.colour())
        ]
        .padding([2, 4]);

        // ── Top bar: shader picker + description + reload ──────────────────────
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
            .width(Length::Fill)
            .style(pick_list_style)
        } else {
            pick_list(
                &[] as &[ShaderConfig],
                None::<&ShaderConfig>,
                |_: ShaderConfig| TextureSplitterMessage::ShaderSelected(String::new()),
            )
            .placeholder("No shaders available")
            .width(Length::Fill)
            .style(pick_list_style)
        };

        let shader_description: String = self
            .state
            .get_selected_shader()
            .map(|s| s.shader.description.clone())
            .unwrap_or_default();

        let reload_button = button("Reload")
            .on_press(TextureSplitterMessage::ReloadShaders)
            .padding([6, 12])
            .style(crate::widget_helpers::secondary_button_style);

        let top_bar = container(
            row![
                column![
                    shader_picker,
                    text(shader_description)
                        .size(11)
                        .style(|theme: &iced::Theme| iced::widget::text::Style {
                            color: Some(iced::Color {
                                a: 0.55,
                                ..theme.extended_palette().background.base.text
                            }),
                        }),
                ]
                .spacing(4)
                .width(Length::Fill),
                container(reload_button).align_x(iced::alignment::Horizontal::Right),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center),
        )
        .padding([8, 12])
        .width(Length::Fill)
        .style(crate::widget_helpers::dark_style);

        // ── Input slot views ──────────────────────────────────────────────────
        let input_slot_views: Vec<_> = self
            .state
            .input_slots
            .iter()
            .enumerate()
            .map(|(idx, slot)| self.view_compact_slot(idx, slot))
            .collect();

        // ── Output preview with navigation ────────────────────────────────────
        let output_widget = if !self.state.outputs.is_empty() {
            let current_handle = &self.state.outputs[self.state.current_output_index];
            let current_desc = &self.state.output_descriptions[self.state.current_output_index];

            let preview = container(
                iced::widget::image(current_handle.clone())
                    .content_fit(iced::ContentFit::Contain)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(crate::widget_helpers::dark_style);

            if self.state.outputs.len() > 1 {
                let prev_button = button("\u{2039}")
                    .on_press_maybe(if self.state.current_output_index > 0 {
                        Some(TextureSplitterMessage::PreviousOutput)
                    } else {
                        None
                    })
                    .padding([4, 10])
                    .style(crate::widget_helpers::primary_button_style);

                let next_button = button("\u{203a}")
                    .on_press_maybe(
                        if self.state.current_output_index < self.state.outputs.len() - 1 {
                            Some(TextureSplitterMessage::NextOutput)
                        } else {
                            None
                        },
                    )
                    .padding([4, 10])
                    .style(crate::widget_helpers::primary_button_style);

                let nav_row = row![
                    prev_button,
                    text(format!(
                        "{} ({}/{})",
                        current_desc,
                        self.state.current_output_index + 1,
                        self.state.outputs.len()
                    ))
                    .size(12)
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center),
                    next_button,
                ]
                .spacing(6)
                .align_y(iced::Alignment::Center)
                .width(Length::Fill);

                column![preview, nav_row]
                    .spacing(6)
                    .align_x(iced::Alignment::Center)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            } else {
                column![
                    preview,
                    text(current_desc.clone())
                        .size(12)
                        .align_x(iced::alignment::Horizontal::Center)
                        .width(Length::Fill),
                ]
                .spacing(6)
                .align_x(iced::Alignment::Center)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
        } else {
            create_output_preview(&None, "Output will appear here")
        };

        // ── Parameter sliders ─────────────────────────────────────────────────
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
                controls.push(create_slider_control(
                    param.description.clone(),
                    current_value as f64,
                    param.min as f64..=param.max as f64,
                    move |val| {
                        TextureSplitterMessage::ParameterChanged(param_name.clone(), val as f32)
                    },
                ));
            }
        }

        // Output format picker
        let format_selector = pick_list(
            &ImageFormat::ALL[..],
            Some(self.selected_format),
            TextureSplitterMessage::FormatSelected,
        )
        .padding([6, 10])
        .width(Length::Fill)
        .placeholder("Format")
        .style(pick_list_style);

        controls.push(
            column![
                text("Output Format").size(12).style(|theme: &iced::Theme| {
                    iced::widget::text::Style {
                        color: Some(iced::Color {
                            a: 0.6,
                            ..theme.extended_palette().background.base.text
                        }),
                    }
                }),
                format_selector,
            ]
            .spacing(4)
            .into(),
        );

        // DDS sub-format — only visible when DDS is selected
        if self.selected_format == ImageFormat::Dds {
            let dds_format_selector = pick_list(
                &DdsSaveFormat::ALL[..],
                Some(self.selected_dds_format),
                TextureSplitterMessage::DdsFormatSelected,
            )
            .padding([6, 10])
            .width(Length::Fill)
            .placeholder("DDS Format")
            .style(pick_list_style);

            controls.push(
                column![
                    text("DDS Pixel Format")
                        .size(12)
                        .style(|theme: &iced::Theme| iced::widget::text::Style {
                            color: Some(iced::Color {
                                a: 0.6,
                                ..theme.extended_palette().background.base.text
                            }),
                        }),
                    dds_format_selector,
                ]
                .spacing(4)
                .into(),
            );
        }

        // ── Buttons ───────────────────────────────────────────────────────────
        let buttons = vec![
            create_save_all_button(
                self.state.is_saving,
                !self.state.output_buffers.is_empty(),
                TextureSplitterMessage::SaveAllPressed,
            ),
            create_clear_button(TextureSplitterMessage::ClearPressed),
        ];

        // ── Assemble ──────────────────────────────────────────────────────────
        create_baker_layout(BakerLayoutConfig {
            top_bar: top_bar.into(),
            input_slots: input_slot_views,
            output_widget,
            controls,
            buttons,
            status_bar: status_bar.into(),
        })
    }

    /// Handle file dropped onto the window.
    ///
    /// Loads the dropped file into the first available empty slot.
    /// If all slots are filled, displays a warning message.
    pub fn on_file_dropped(&mut self, path: PathBuf) -> Task<Message> {
        for (idx, slot) in self.state.input_slots.iter().enumerate() {
            if slot.image.is_none() {
                return self.on_input_file_selected(idx, Some(path));
            }
        }

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
                            "{} shader{} loaded, {} failed. Check texture_smith.log for details.",
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

            let path_clone = path.clone();
            Task::perform(
                async move {
                    match PorterImage::open(&path) {
                        Ok(img) => Ok(img),
                        Err(e) => Err(format!("Failed to load image: {e}")),
                    }
                },
                move |result| {
                    Message::Main(crate::windows::MainMessage::TextureSplitter(
                        TextureSplitterMessage::InputImageLoaded(
                            slot_idx,
                            result,
                            Some(path_clone),
                        ),
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
        path: Option<PathBuf>,
    ) -> Task<Message> {
        match result {
            Ok(img) => {
                if slot_idx < self.state.input_slots.len() {
                    self.state.input_slots[slot_idx].load_image(img, path);
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
        // Check if we can process based on shader requirements
        let can_process = if let Some(shader) = self.state.get_selected_shader() {
            // For shaders with no inputs, we can always process
            if shader.inputs.is_empty() {
                true
            } else {
                // For shaders with inputs, check if required slots are filled
                self.state.all_required_slots_filled()
            }
        } else {
            false
        };

        if !can_process {
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

    /// Save all output images to the first input slot's directory
    ///
    /// Saves all outputs with auto-generated filenames to the same directory
    /// as the first input image, using the currently selected image format.
    fn on_save_all(&mut self) -> Task<Message> {
        if !self.state.is_saving && !self.state.output_buffers.is_empty() {
            // Get the directory from the first input slot
            let folder_path = match self
                .state
                .input_slots
                .first()
                .and_then(|slot| slot.get_directory())
            {
                Some(dir) => dir,
                None => {
                    self.state.status =
                        StatusMessage::error("No input image path available. Load an image first.");
                    return Task::none();
                }
            };

            self.state.is_saving = true;
            self.state.status = StatusMessage::info(format!(
                "Saving all outputs to {}...",
                folder_path.display()
            ));

            // Clone all output buffers and descriptions
            let outputs: Vec<(ImageBuffer, String)> = self
                .state
                .output_buffers
                .iter()
                .zip(self.state.output_descriptions.iter())
                .map(|(buffer, desc)| (buffer.clone(), desc.clone()))
                .collect();

            let format = self.selected_format;
            let dds_format = self.selected_dds_format;
            let folder_path_clone = folder_path.clone();

            Task::perform(
                async move {
                    texture_smith_core::save::save_outputs(
                        &outputs,
                        &folder_path_clone,
                        format,
                        dds_format,
                    )
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

    /// Compact horizontal slot row: [thumbnail] label [Browse]
    fn view_compact_slot<'a>(
        &'a self,
        idx: usize,
        slot: &'a DroppableImageSlot,
    ) -> Element<'a, TextureSplitterMessage> {
        use crate::widget_helpers::{dark_style, drop_zone_style, primary_button_style};
        use iced::widget::{button, container, row, text};

        const THUMB: f32 = 44.0;

        let thumbnail: Element<_> = if let Some(handle) = self.state.get_input_slot_handle(idx) {
            container(
                iced::widget::image(handle.clone())
                    .content_fit(iced::ContentFit::Cover)
                    .width(THUMB)
                    .height(THUMB),
            )
            .width(THUMB)
            .height(THUMB)
            .style(dark_style)
            .into()
        } else {
            container(text("\u{00b7}").size(20).style(|theme: &iced::Theme| {
                iced::widget::text::Style {
                    color: Some(iced::Color {
                        a: 0.2,
                        ..theme.extended_palette().background.base.text
                    }),
                }
            }))
            .width(THUMB)
            .height(THUMB)
            .style(drop_zone_style)
            .center_x(THUMB)
            .center_y(THUMB)
            .into()
        };

        let browse_btn = button(text("Browse").size(11))
            .on_press(TextureSplitterMessage::BrowseInput(idx))
            .padding([5, 10])
            .style(primary_button_style);

        container(
            row![
                thumbnail,
                text(&slot.label)
                    .size(12)
                    .width(Length::Fill)
                    .style(|theme: &iced::Theme| iced::widget::text::Style {
                        color: Some(theme.extended_palette().background.base.text),
                    }),
                browse_btn,
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center),
        )
        .padding([6, 8])
        .width(Length::Fill)
        .style(crate::widget_helpers::frame_style)
        .into()
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
                // Generate proper output descriptions using input filename with suffix
                let corrected_outputs = if let Some(shader) = self.state.get_selected_shader() {
                    // Get base filename from first input slot
                    let base_filename = self
                        .state
                        .input_slots
                        .first()
                        .and_then(|slot| slot.path.as_ref())
                        .and_then(|path| path.file_stem())
                        .map(|stem| stem.to_string_lossy().to_string())
                        .unwrap_or_else(|| "output".to_string());

                    // Create new outputs with proper descriptions
                    outputs
                        .into_iter()
                        .zip(shader.outputs.iter())
                        .map(|((buffer, _desc), output_config)| {
                            let description = format!("{}{}", base_filename, output_config.suffix);
                            (buffer, description)
                        })
                        .collect()
                } else {
                    outputs
                };

                self.state.set_outputs(corrected_outputs);

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
