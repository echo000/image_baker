use crate::components::baker_layout::*;
use crate::components::droppable_image_slot::DroppableImageSlot;
use crate::components::image_operations;
use crate::core_logic;
use crate::messages::{ImageType, Message};
use crate::status::StatusMessage;
use ::image::{DynamicImage, ImageBuffer, Rgba};
use iced::widget::text;
use iced::{Element, Task};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Baker component state
pub struct Baker {
    pub colour_slot: DroppableImageSlot,
    pub specular_slot: DroppableImageSlot,
    pub occlusion_slot: DroppableImageSlot,
    pub output: Option<iced::widget::image::Handle>,
    pub output_buffer: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    pub is_saving: bool,
    pub status: StatusMessage,
    pub last_merge_time: Option<Instant>,
    pub pending_merge: bool,
    pub ao_contrast_power: f64,
    pub specular_intensity: f64,
}

/// Messages produced by the baker component.
#[derive(Debug, Clone)]
pub enum BakerMessage {
    SavePressed,
    ClearPressed,
    AoChanged(f64),
    SpecularChanged(f64),
    BrowseColour,
    BrowseSpecular,
    BrowseOcclusion,
    FileSelected(ImageType, Option<PathBuf>),
    ImageLoaded(ImageType, Result<DynamicImage, String>),
    MergeCompleted(Result<ImageBuffer<Rgba<u8>, Vec<u8>>, String>),
    ImageSaved(Result<PathBuf, String>),
    Tick,
}

impl Baker {
    /// Creates a new baker component.
    pub fn new() -> Self {
        Self {
            colour_slot: DroppableImageSlot::new("Colour Map"),
            specular_slot: DroppableImageSlot::new("Specular Map"),
            occlusion_slot: DroppableImageSlot::new("Occlusion Map"),
            output: None,
            output_buffer: None,
            is_saving: false,
            status: StatusMessage::info("Ready. Drop or browse for images."),
            last_merge_time: None,
            pending_merge: false,
            ao_contrast_power: 1.0,
            specular_intensity: 0.5,
        }
    }

    /// Handles updates for the baker component.
    pub fn update(&mut self, message: BakerMessage) -> Task<Message> {
        use BakerMessage::*;

        match message {
            ClearPressed => self.on_clear(),
            SavePressed => self.on_save(),
            AoChanged(val) => self.on_ao_changed(val),
            SpecularChanged(val) => self.on_specular_changed(val),
            BrowseColour => self.on_browse(ImageType::Colour),
            BrowseSpecular => self.on_browse(ImageType::Specular),
            BrowseOcclusion => self.on_browse(ImageType::Occlusion),
            FileSelected(img_type, path_opt) => self.on_file_selected(img_type, path_opt),
            ImageLoaded(img_type, result) => {
                let task = self.on_image_loaded(img_type, result);
                // After loading, trigger merge if we have a colour image
                if self.colour_slot.image.is_some() {
                    return Task::batch([
                        task,
                        self.trigger_merge_with_settings(
                            self.ao_contrast_power,
                            self.specular_intensity,
                        ),
                    ]);
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
                            return self.trigger_merge_with_settings(
                                self.ao_contrast_power,
                                self.specular_intensity,
                            );
                        }
                    } else {
                        self.pending_merge = false;
                    }
                }
                Task::none()
            }
        }
    }

    /// Handles rendering the baker component.
    pub fn view<'a>(&'a self) -> Element<'a, BakerMessage> {
        // Render each droppable image slot
        let colour_slot = self
            .colour_slot
            .view(Some(BakerMessage::BrowseColour), "Drop file or Browse");

        let specular_slot = self
            .specular_slot
            .view(Some(BakerMessage::BrowseSpecular), "Drop file or Browse");

        let occlusion_slot = self
            .occlusion_slot
            .view(Some(BakerMessage::BrowseOcclusion), "Drop file or Browse");

        // Output preview
        let output_widget = create_output_preview(&self.output, "Output will appear here");

        // Controls
        let specular_slider = create_slider_control(
            "Specular Intensity",
            self.specular_intensity,
            0.0..=1.0,
            BakerMessage::SpecularChanged,
        );

        let ao_slider = create_slider_control(
            "AO Contrast",
            self.ao_contrast_power,
            0.0..=1.0,
            BakerMessage::AoChanged,
        );

        let save_button = create_save_button(
            self.is_saving,
            self.output_buffer.is_some(),
            BakerMessage::SavePressed,
        );

        let clear_button = create_clear_button(BakerMessage::ClearPressed);

        // Status bar
        let status_bar = text(&self.status.message)
            .size(12)
            .color(self.status.colour());

        // Build the layout - overlays are now shown per-slot!
        create_baker_layout(BakerLayoutConfig {
            input_slots: vec![colour_slot, specular_slot, occlusion_slot],
            output_widget,
            controls: vec![specular_slider, ao_slider],
            buttons: vec![save_button, clear_button],
            status_bar: status_bar.into(),
        })
    }

    /// Occurs when a file has been dropped.
    /// Uses hovered_slot to determine which slot to load the image into.
    pub fn on_file_dropped(&mut self, path: PathBuf, img_type: ImageType) -> Task<Message> {
        self.status = image_operations::loading_status_message(&img_type, &path);

        Task::perform(
            image_operations::load_image_async(path, img_type),
            |(img_type, result)| Message::Main(BakerMessage::ImageLoaded(img_type, result).into()),
        )
    }

    /// Occurs when an unknown file has been dropped.
    pub fn on_unknown_file_dropped(&mut self, path: PathBuf) -> Task<Message> {
        self.status = image_operations::unknown_file_status_message(&path, "_c, _s, or _o");
        Task::none()
    }

    fn on_clear(&mut self) -> Task<Message> {
        self.colour_slot.clear();
        self.specular_slot.clear();
        self.occlusion_slot.clear();
        self.output = None;
        self.output_buffer = None;
        self.status = StatusMessage::info("Cleared all images.");
        tracing::info!("Cleared all loaded images");
        Task::none()
    }

    fn on_save(&mut self) -> Task<Message> {
        if !self.is_saving {
            self.is_saving = true;
            self.status = StatusMessage::info("Saving...");
            if let Some(buffer) = self.output_buffer.clone() {
                return Task::perform(image_operations::save_image_async(buffer), |result| {
                    Message::Main(BakerMessage::ImageSaved(result).into())
                });
            }
        }
        Task::none()
    }

    fn on_ao_changed(&mut self, val: f64) -> Task<Message> {
        self.ao_contrast_power = val;
        self.trigger_debounced_merge()
    }

    fn on_specular_changed(&mut self, val: f64) -> Task<Message> {
        self.specular_intensity = val;
        self.trigger_debounced_merge()
    }

    fn on_browse(&mut self, img_type: ImageType) -> Task<Message> {
        Task::perform(
            image_operations::browse_for_image_async(img_type),
            |(img_type, path)| Message::Main(BakerMessage::FileSelected(img_type, path).into()),
        )
    }

    fn on_file_selected(
        &mut self,
        img_type: ImageType,
        path_opt: Option<PathBuf>,
    ) -> Task<Message> {
        if let Some(path) = path_opt {
            self.status = image_operations::loading_status_message(&img_type, &path);
            return Task::perform(
                image_operations::load_image_async(path, img_type),
                |(img_type, result)| {
                    Message::Main(BakerMessage::ImageLoaded(img_type, result).into())
                },
            );
        }
        Task::none()
    }

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

                self.status = image_operations::loaded_status_message(&img_type);
                match img_type {
                    ImageType::Colour => self.colour_slot.load_image(img),
                    ImageType::Specular => self.specular_slot.load_image(img),
                    ImageType::Occlusion => self.occlusion_slot.load_image(img),
                    ImageType::Detail => {
                        // DetailMap is not used in baker, only in detail mapper
                    }
                }
                Task::none()
            }
            Err(e) => {
                self.status = image_operations::load_error_status_message(&e);
                Task::none()
            }
        }
    }

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
            }
            Err(e) => {
                self.status = image_operations::merge_error_status_message(&e);
            }
        }
        Task::none()
    }

    fn on_image_saved(&mut self, result: Result<PathBuf, String>) -> Task<Message> {
        self.is_saving = false;
        match result {
            Ok(path) => {
                self.status = image_operations::save_success_status_message(&path);
            }
            Err(e) => self.status = image_operations::save_error_status_message(&e),
        }
        Task::none()
    }

    fn trigger_debounced_merge(&mut self) -> Task<Message> {
        self.pending_merge = true;
        self.last_merge_time = Some(Instant::now());
        Task::none()
    }

    /// Triggers a merge with the given settings.
    pub fn trigger_merge_with_settings(
        &self,
        ao_contrast_power: f64,
        specular_intensity: f64,
    ) -> Task<Message> {
        if let Some(c) = &self.colour_slot.image {
            let c = c.clone();
            let s = self.specular_slot.image.clone();
            let o = self.occlusion_slot.image.clone();

            Task::perform(
                merge_async(c, s, o, ao_contrast_power, specular_intensity),
                |result| Message::Main(BakerMessage::MergeCompleted(result).into()),
            )
        } else {
            Task::none()
        }
    }
}

async fn merge_async(
    colour_map: DynamicImage,
    specular_map: Option<DynamicImage>,
    occlusion_map: Option<DynamicImage>,
    ao_contrast_power: f64,
    specular_intensity: f64,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, String> {
    let result = core_logic::merge_images(
        &colour_map,
        specular_map.as_ref(),
        occlusion_map.as_ref(),
        ao_contrast_power,
        specular_intensity,
    );

    Ok(result)
}
