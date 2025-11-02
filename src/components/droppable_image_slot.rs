use iced::widget::{Container, button, column, container, text};
use iced::{Alignment, Element, Length};
use image::DynamicImage;

/// A self-contained droppable image slot component
#[derive(Debug, Clone)]
pub struct DroppableImageSlot {
    /// Label for this slot
    pub label: String,
    /// The currently loaded image (if any)
    pub image: Option<DynamicImage>,
}

impl DroppableImageSlot {
    /// Create a new droppable image slot
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            image: None,
        }
    }

    /// Load an image into this slot
    pub fn load_image(&mut self, image: DynamicImage) {
        self.image = Some(image);
    }

    /// Clear the image from this slot
    pub fn clear(&mut self) {
        self.image = None;
    }

    /// Render this slot
    pub fn view<'a, Message: 'a + Clone>(
        &'a self,
        browse_message: Option<Message>,
        placeholder_text: &'a str,
    ) -> Element<'a, Message> {
        use crate::widget_helpers::{control, primary_button_style};
        use image::GenericImageView;

        const PREVIEW_SIZE: f32 = 220.0;

        // Create the image preview or placeholder
        let image_widget = if let Some(img) = &self.image {
            let (w, h) = img.dimensions();
            if w > 0 && h > 0 && w <= 8192 && h <= 8192 {
                let handle = crate::core_logic::image_to_handle(img);
                container(
                    iced::widget::image(handle)
                        .content_fit(iced::ContentFit::Cover)
                        .width(PREVIEW_SIZE)
                        .height(PREVIEW_SIZE),
                )
                .width(PREVIEW_SIZE)
                .height(PREVIEW_SIZE)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
            } else {
                Self::create_placeholder(placeholder_text, PREVIEW_SIZE)
            }
        } else {
            Self::create_placeholder(placeholder_text, PREVIEW_SIZE)
        };

        // Build the complete slot with label and browse button
        let mut col = column![image_widget].spacing(8).align_x(Alignment::Center);

        if let Some(browse_msg) = browse_message {
            col = col.push(
                button("Browse...")
                    .on_press(browse_msg)
                    .padding(8)
                    .width(PREVIEW_SIZE)
                    .style(primary_button_style),
            );
        }

        control(text(&self.label).size(13).into(), col.into()).into()
    }

    /// Create a placeholder widget
    fn create_placeholder<'a, Message: 'a>(msg: &'a str, size: f32) -> Container<'a, Message> {
        container(
            text(msg)
                .size(14)
                .align_x(iced::alignment::Horizontal::Center),
        )
        .width(size)
        .height(size)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
    }
}
