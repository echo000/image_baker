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
        let title = text("Texture Smith")
            .size(32)
            .align_x(iced::alignment::Horizontal::Center);

        let subtitle = text("GPU-Accelerated Texture Processing")
            .size(16)
            .color(iced::Theme::default().extended_palette().primary.base.color)
            .align_x(iced::alignment::Horizontal::Center);

        let version = text(format!("Version {}", state.version))
            .size(14)
            .align_x(iced::alignment::Horizontal::Center);

        let features_title = text("Features:")
            .size(20)
            .align_x(iced::alignment::Horizontal::Left);

        let shader_feature = text("• Custom Shader System: Process textures with WGSL shaders for unlimited creative possibilities")
            .size(14);

        let multi_output_feature =
            text("• Multiple Outputs: Generate multiple texture maps from a single shader pass")
                .size(14);

        let realtime_feature = text("• Real-time Preview: Instant visual feedback with smooth, debounced parameter controls")
            .size(14);

        let drag_drop_feature = text("• Smart Loading: Drag & drop images or use file browser with automatic slot assignment")
            .size(14);

        let format_feature = text(
            "• Multi-Format Support: Save as PNG, TGA, TIFF, or DDS with one-click batch export",
        )
        .size(14);

        let author_title = text("Created By:")
            .size(20)
            .align_x(iced::alignment::Horizontal::Left);

        let author = text("echo000")
            .color(Color::from_rgb8(0xEC, 0x34, 0xCA))
            .size(16)
            .align_x(iced::alignment::Horizontal::Left);

        let github = text("Open source - Built with Rust, Iced, and WGPU")
            .size(12)
            .color(iced::Theme::default().extended_palette().primary.base.color)
            .align_x(iced::alignment::Horizontal::Center);

        let content = column![
            title,
            Space::with_height(5),
            subtitle,
            Space::with_height(10),
            version,
            Space::with_height(30),
            features_title,
            Space::with_height(10),
            shader_feature,
            Space::with_height(8),
            multi_output_feature,
            Space::with_height(8),
            realtime_feature,
            Space::with_height(8),
            drag_drop_feature,
            Space::with_height(8),
            format_feature,
            Space::with_height(30),
            author_title,
            Space::with_height(10),
            author,
            Space::with_height(30),
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
