use crate::app_state::AppState;
use crate::messages::Message;
use iced::widget::{Space, column, container, scrollable, text};
use iced::{Alignment, Color, Element, Length};

/// About component
pub struct About;

impl About {
    /// Creates a new About component
    pub fn new() -> Self {
        Self
    }

    /// Renders the About view
    pub fn view(&self, state: &AppState) -> Element<'_, Message> {
        let title = text("Image Baker")
            .size(32)
            .align_x(iced::alignment::Horizontal::Center);

        let version = text(format!("Version {}", state.version))
            .size(16)
            .align_x(iced::alignment::Horizontal::Center);

        let features_title = text("Features:")
            .size(18)
            .align_x(iced::alignment::Horizontal::Left);

        let spec_gloss_feature = text("• C/S/O Baker: Combine colour, specular, and occlusion maps into a single texture. Pack RGB albedo with specular in alpha, and optionally blend ambient occlusion.")
            .size(14);

        let detail_mapper_feature = text("• Detail Baker: Blend detail maps with base color textures to add fine surface detail and variation.")
            .size(14);

        let drag_drop_feature = text("• Drag & Drop: Supports filename-based automatic routing (_c for colour, _s for specular, _o for occlusion, _d for detail map).")
            .size(14);

        let realtime_feature = text(
            "• Real-time Preview: See your results instantly with adjustable intensity controls.",
        )
        .size(14);

        let author_title = text("Created By:")
            .size(18)
            .align_x(iced::alignment::Horizontal::Left);

        let author = text("echo000")
            .color(Color::from_rgb8(0xEC, 0x34, 0xCA))
            .size(16)
            .align_x(iced::alignment::Horizontal::Left);

        let github = text("Open source - Built with Rust and Iced")
            .size(12)
            .color(iced::Theme::default().extended_palette().primary.base.color)
            .align_x(iced::alignment::Horizontal::Center);

        let content = column![
            title,
            Space::with_height(10),
            version,
            Space::with_height(20),
            features_title,
            Space::with_height(10),
            spec_gloss_feature,
            Space::with_height(10),
            detail_mapper_feature,
            Space::with_height(10),
            drag_drop_feature,
            Space::with_height(10),
            realtime_feature,
            Space::with_height(40),
            author_title,
            Space::with_height(10),
            author,
            Space::with_height(40),
            github,
        ]
        .spacing(5)
        .padding(40)
        .width(Length::Fill)
        .align_x(Alignment::Center);

        scrollable(
            container(content)
                .width(Length::Fill)
                .center_x(Length::Fill),
        )
        .into()
    }
}

impl Default for About {
    fn default() -> Self {
        Self::new()
    }
}
