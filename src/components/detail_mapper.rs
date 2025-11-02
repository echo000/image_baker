use crate::components::baker_layout::*;
use crate::components::droppable_image_slot::DroppableImageSlot;
use crate::components::image_operations;
use crate::messages::{ImageType, Message};
use crate::status::StatusMessage;
use ::image::{DynamicImage, ImageBuffer, Rgba};
use iced::widget::text;
use iced::{Element, Task};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Detail mapper component state
pub struct DetailMapper {
    pub base_colour_slot: DroppableImageSlot,
    pub detail_map_slot: DroppableImageSlot,
    pub output: Option<iced::widget::image::Handle>,
    pub output_buffer: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    pub is_saving: bool,
    pub status: StatusMessage,
    pub detail_intensity: f64,
    pub last_merge_time: Option<Instant>,
    pub pending_merge: bool,
}

/// Messages produced by the detail mapper component.
#[derive(Debug, Clone)]
pub enum DetailMapperMessage {
    SavePressed,
    ClearPressed,
    IntensityChanged(f64),
    BrowseBaseColour,
    BrowseDetailMap,
    FileSelected(ImageType, Option<PathBuf>),
    ImageLoaded(ImageType, Result<DynamicImage, String>),
    MergeCompleted(Result<ImageBuffer<Rgba<u8>, Vec<u8>>, String>),
    ImageSaved(Result<PathBuf, String>),
    Tick,
}

impl DetailMapper {
    /// Creates a new detail mapper component.
    pub fn new() -> Self {
        Self {
            base_colour_slot: DroppableImageSlot::new("Base Colour"),
            detail_map_slot: DroppableImageSlot::new("Detail Map"),
            output: None,
            output_buffer: None,
            is_saving: false,
            status: StatusMessage::info("Ready. Drop or browse for images."),
            detail_intensity: 0.5,
            last_merge_time: None,
            pending_merge: false,
        }
    }

    /// Handles updates for the detail mapper component.
    pub fn update(&mut self, message: DetailMapperMessage) -> Task<Message> {
        use DetailMapperMessage::*;

        match message {
            ClearPressed => self.on_clear(),
            SavePressed => self.on_save(),
            IntensityChanged(val) => self.on_intensity_changed(val),
            BrowseBaseColour => self.on_browse(ImageType::Colour),
            BrowseDetailMap => self.on_browse(ImageType::Detail),
            FileSelected(img_type, path_opt) => self.on_file_selected(img_type, path_opt),
            ImageLoaded(img_type, result) => {
                let task = self.on_image_loaded(img_type, result);
                // After loading, trigger merge if we have both images
                if self.base_colour_slot.image.is_some() && self.detail_map_slot.image.is_some() {
                    return Task::batch([task, self.trigger_debounced_merge()]);
                }
                task
            }
            MergeCompleted(result) => self.on_merge_completed(result),
            ImageSaved(result) => self.on_image_saved(result),
            Tick => {
                if self.pending_merge {
                    if let Some(last_merge) = self.last_merge_time {
                        if last_merge.elapsed() >= Duration::from_millis(150) {
                            self.pending_merge = false;
                            self.last_merge_time = None;
                            return self.trigger_merge(self.detail_intensity);
                        }
                    } else {
                        self.pending_merge = false;
                    }
                }
                Task::none()
            }
        }
    }

    /// Renders the detail mapper UI.
    pub fn view(&self) -> Element<DetailMapperMessage> {
        // Render each droppable image slot
        let base_slot = self.base_colour_slot.view(
            Some(DetailMapperMessage::BrowseBaseColour),
            "Drop file or Browse",
        );

        let detail_slot = self.detail_map_slot.view(
            Some(DetailMapperMessage::BrowseDetailMap),
            "Drop file or Browse",
        );

        // Output preview
        let output_widget = create_output_preview(&self.output, "Output will appear here");

        // Controls
        let intensity_slider = create_slider_control(
            "Detail Intensity",
            self.detail_intensity,
            0.0..=1.0,
            DetailMapperMessage::IntensityChanged,
        );

        let save_button = create_save_button(
            self.is_saving,
            self.output_buffer.is_some(),
            DetailMapperMessage::SavePressed,
        );

        let clear_button = create_clear_button(DetailMapperMessage::ClearPressed);

        // Status bar
        let status_bar = text(&self.status.message)
            .size(12)
            .color(self.status.colour());

        // Build the layout - overlays are now shown per-slot!
        create_baker_layout(BakerLayoutConfig {
            input_slots: vec![base_slot, detail_slot],
            output_widget,
            controls: vec![intensity_slider],
            buttons: vec![save_button, clear_button],
            status_bar: status_bar.into(),
        })
    }

    /// Occurs when an unknown file has been dropped.
    pub fn on_unknown_file_dropped(&mut self, path: PathBuf) -> Task<Message> {
        self.status = image_operations::unknown_file_status_message(&path, "_c or _d");
        Task::none()
    }

    /// Handles clearing all images.
    fn on_clear(&mut self) -> Task<Message> {
        self.base_colour_slot.clear();
        self.detail_map_slot.clear();
        self.output = None;
        self.output_buffer = None;
        self.status = StatusMessage::info("Cleared all images.");
        Task::none()
    }

    /// Handles save button press.
    fn on_save(&mut self) -> Task<Message> {
        if self.output_buffer.is_none() {
            self.status = StatusMessage::error("No output to save");
            return Task::none();
        }

        self.is_saving = true;
        self.status = StatusMessage::info("Saving...");

        Task::perform(
            image_operations::save_image_async(self.output_buffer.clone().unwrap()),
            |result| {
                Message::Main(crate::windows::MainMessage::DetailMapper(
                    DetailMapperMessage::ImageSaved(result),
                ))
            },
        )
    }

    /// Handles intensity change.
    fn on_intensity_changed(&mut self, value: f64) -> Task<Message> {
        self.detail_intensity = value;

        if self.base_colour_slot.image.is_some() && self.detail_map_slot.image.is_some() {
            self.trigger_debounced_merge()
        } else {
            Task::none()
        }
    }

    /// Handles browse button.
    fn on_browse(&mut self, img_type: ImageType) -> Task<Message> {
        Task::perform(
            image_operations::browse_for_image_async(img_type),
            |(img_type, path)| {
                Message::Main(DetailMapperMessage::FileSelected(img_type, path).into())
            },
        )
    }

    /// Handles file selection.
    fn on_file_selected(
        &mut self,
        img_type: ImageType,
        path_opt: Option<PathBuf>,
    ) -> Task<Message> {
        if let Some(path) = path_opt {
            self.status = image_operations::loading_status_message(&img_type, &path);
            Task::perform(
                image_operations::load_image_async(path, img_type),
                |(img_type, result)| {
                    Message::Main(crate::windows::MainMessage::DetailMapper(
                        DetailMapperMessage::ImageLoaded(img_type, result),
                    ))
                },
            )
        } else {
            Task::none()
        }
    }

    /// Handles image loaded.
    fn on_image_loaded(
        &mut self,
        img_type: ImageType,
        result: Result<DynamicImage, String>,
    ) -> Task<Message> {
        match result {
            Ok(img) => {
                if !image_operations::validate_image_dimensions(&img) {
                    self.status = image_operations::invalid_dimensions_status_message(&img_type);
                    return Task::none();
                }

                match img_type {
                    ImageType::Colour => {
                        self.base_colour_slot.load_image(img);
                    }
                    ImageType::Detail => {
                        self.detail_map_slot.load_image(img);
                    }
                    _ => {}
                }
                self.status = image_operations::loaded_status_message(&img_type);
                Task::none()
            }
            Err(e) => {
                self.status = image_operations::load_error_status_message(&e);
                Task::none()
            }
        }
    }

    /// Trigger a debounced merge operation.
    fn trigger_debounced_merge(&mut self) -> Task<Message> {
        self.pending_merge = true;
        self.last_merge_time = Some(Instant::now());
        Task::none()
    }

    /// Trigger a merge operation.
    fn trigger_merge(&self, intensity: f64) -> Task<Message> {
        let base = self.base_colour_slot.image.clone();
        let detail = self.detail_map_slot.image.clone();

        if base.is_none() || detail.is_none() {
            return Task::none();
        }

        Task::perform(
            merge_detail_maps(base.unwrap(), detail.unwrap(), intensity),
            |result| {
                Message::Main(crate::windows::MainMessage::DetailMapper(
                    DetailMapperMessage::MergeCompleted(result),
                ))
            },
        )
    }

    /// Handles merge completion.
    fn on_merge_completed(
        &mut self,
        result: Result<ImageBuffer<Rgba<u8>, Vec<u8>>, String>,
    ) -> Task<Message> {
        match result {
            Ok(buffer) => {
                if let Some(handle) = image_operations::buffer_to_display_handle(&buffer) {
                    self.status = image_operations::merge_complete_status_message();
                    self.output = Some(handle);
                    self.output_buffer = Some(buffer);
                } else {
                    self.status = image_operations::merge_error_status_message(
                        "Invalid output buffer dimensions",
                    );
                }
                Task::none()
            }
            Err(e) => {
                self.status = image_operations::merge_error_status_message(&e);
                Task::none()
            }
        }
    }

    /// Handles image saved.
    fn on_image_saved(&mut self, result: Result<PathBuf, String>) -> Task<Message> {
        self.is_saving = false;
        match result {
            Ok(path) => {
                self.status = image_operations::save_success_status_message(&path);
            }
            Err(e) => {
                self.status = image_operations::save_error_status_message(&e);
            }
        }
        Task::none()
    }

    /// Helper to handle file drops.
    pub fn on_file_dropped(&mut self, path: PathBuf, img_type: ImageType) -> Task<Message> {
        self.status = image_operations::loading_status_message(&img_type, &path);
        Task::perform(
            image_operations::load_image_async(path, img_type),
            |(img_type, result)| {
                Message::Main(crate::windows::MainMessage::DetailMapper(
                    DetailMapperMessage::ImageLoaded(img_type, result),
                ))
            },
        )
    }
}

/// Async function to merge detail maps.
async fn merge_detail_maps(
    base_colour: DynamicImage,
    detail_map: DynamicImage,
    intensity: f64,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, String> {
    Ok(crate::core_logic::apply_detail_map(
        &base_colour,
        &detail_map,
        intensity,
    ))
}
