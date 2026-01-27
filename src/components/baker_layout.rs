//! Baker Layout Module
//!
//! Provides a standardized layout for texture processing UIs with:
//! - Dynamic input slot arrangement
//! - Output preview section
//! - Parameter controls panel
//! - Action buttons

use iced::widget::{Column, Row, column, container, row, text};
use iced::{Alignment, Element, Length};

// Layout constants
const SPACING: f32 = 15.0;
const OUTPUT_SIZE: f32 = 280.0;

/// Configuration for a baker layout
pub struct BakerLayoutConfig<'a, Message: 'a> {
    /// Input slots (e.g., colour, specular, occlusion, detail, etc.)
    pub input_slots: Vec<Element<'a, Message>>,
    /// Output preview widget
    pub output_widget: Element<'a, Message>,
    /// Control widgets (sliders, etc.)
    pub controls: Vec<Element<'a, Message>>,
    /// Action buttons (save, clear, etc.)
    pub buttons: Vec<Element<'a, Message>>,
    /// Status message at the top
    pub status_bar: Element<'a, Message>,
}

/// Creates a baker layout with dynamic input scaling
///
/// # Layout Structure
/// - Top: Input slot previews (horizontally distributed)
/// - Bottom Left: Output preview
/// - Bottom Right: Parameter controls and action buttons
pub fn create_baker_layout<'a, Message: 'a + Clone>(
    config: BakerLayoutConfig<'a, Message>,
) -> Element<'a, Message> {
    // Top row: Input previews (automatically distributed)
    let mut input_row = Row::new().spacing(SPACING).width(Length::Fill);

    for input_slot in config.input_slots {
        input_row = input_row.push(input_slot);
    }

    // Wrap input row
    let input_container = container(input_row)
        .width(Length::Fill)
        .center_x(Length::Fill);

    // Bottom left: Output preview
    use crate::widget_helpers::control;

    let output_preview = control(
        text("Output").size(14).into(),
        column![
            container(config.output_widget)
                .width(OUTPUT_SIZE)
                .height(OUTPUT_SIZE)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
        ]
        .spacing(6)
        .align_x(Alignment::Center)
        .into(),
    );

    let output_container = container(output_preview).width(Length::FillPortion(1));

    // Bottom right: Controls panel
    let mut controls_column = Column::new()
        .spacing(20)
        .width(Length::Fill)
        .height(Length::Fill);

    // Add spacer at top
    controls_column = controls_column.push(container(text("")).height(Length::Fill));

    // Add all control widgets
    for control in config.controls {
        controls_column = controls_column.push(control);
    }

    // Add all button widgets
    for button in config.buttons {
        controls_column = controls_column.push(button);
    }

    let controls_panel = container(controls_column)
        .width(Length::FillPortion(1))
        .height(Length::Fill);

    // Bottom row: Output preview (left) + Controls (right)
    let bottom_row = row![output_container, controls_panel]
        .spacing(SPACING)
        .width(Length::Fill)
        .height(Length::Fill);

    // Combine all sections vertically
    let content = column![config.status_bar, input_container, bottom_row]
        .spacing(8)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill);

    content.into()
}

/// Create an output preview widget from an optional handle
///
/// Shows either the provided image or a placeholder with text.
pub fn create_output_preview<'a, Message: 'a>(
    output_handle: &'a Option<iced::widget::image::Handle>,
    placeholder_text: &'a str,
) -> Element<'a, Message> {
    use iced::widget::image;

    const OUTPUT_SIZE: f32 = 280.0;

    if let Some(handle) = output_handle {
        container(
            image(handle.clone())
                .content_fit(iced::ContentFit::Contain)
                .width(OUTPUT_SIZE)
                .height(OUTPUT_SIZE),
        )
        .width(OUTPUT_SIZE)
        .height(OUTPUT_SIZE)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    } else {
        create_placeholder(placeholder_text, OUTPUT_SIZE)
    }
}

/// Create a placeholder widget with centered text
pub fn create_placeholder<'a, Message: 'a>(msg: &'a str, size: f32) -> Element<'a, Message> {
    container(
        text(msg)
            .size(14)
            .align_x(iced::alignment::Horizontal::Center),
    )
    .width(size)
    .height(size)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

/// Create a slider control with label and value display
///
/// # Arguments
/// * `label` - Text label for the slider
/// * `value` - Current slider value
/// * `range` - Min and max values for the slider
/// * `on_change` - Callback when value changes
pub fn create_slider_control<'a, Message: 'a + Clone>(
    label: &str,
    value: f64,
    range: std::ops::RangeInclusive<f64>,
    on_change: impl Fn(f64) -> Message + 'a,
) -> Element<'a, Message> {
    use crate::widget_helpers::slider_style;
    use iced::widget::slider;

    column![
        text(format!("{label}: {value:.2}")).size(13),
        slider(range, value, on_change)
            .step(0.01)
            .width(Length::Fill)
            .style(slider_style)
    ]
    .spacing(8)
    .into()
}

/// Create a save all outputs button
///
/// Button is disabled when saving or no outputs are available.
pub fn create_save_all_button<'a, Message: 'a + Clone>(
    is_saving: bool,
    has_output: bool,
    on_press: Message,
) -> Element<'a, Message> {
    use crate::widget_helpers::success_button_style;
    use iced::widget::button;

    let save_all_button = button(text(if is_saving {
        "Saving..."
    } else {
        "Save Outputs"
    }))
    .padding(12)
    .width(Length::Fill)
    .style(success_button_style);

    if has_output && !is_saving {
        save_all_button.on_press(on_press).into()
    } else {
        save_all_button.into()
    }
}

/// Create a clear button for resetting all inputs and outputs
pub fn create_clear_button<'a, Message: 'a + Clone>(on_press: Message) -> Element<'a, Message> {
    use crate::widget_helpers::danger_button_style;
    use iced::widget::button;

    button(text("Clear"))
        .padding(12)
        .width(Length::Fill)
        .style(danger_button_style)
        .on_press(on_press)
        .into()
}
