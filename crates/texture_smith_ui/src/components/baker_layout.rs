//! Baker Layout Module
//!
//! Two-column layout:
//!   Left sidebar (FillPortion 3): section labels, compact input slots,
//!                                  options/controls, pinned save+clear buttons
//!   Right output  (FillPortion 5): large output preview, optional nav row
//!
//! Section labels replace nested title-bar boxes — the hierarchy comes from
//! spacing and typography, not borders inside borders.

use iced::widget::{Column, column, container, horizontal_rule, row, scrollable, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

const OUTER_PADDING: iced::Padding = iced::Padding {
    top: 10.0,
    right: 16.0,
    bottom: 6.0,
    left: 16.0,
};
const PANEL_GAP: f32 = 10.0;
const SIDEBAR_PAD: f32 = 12.0;

/// Configuration for the baker layout.
pub struct BakerLayoutConfig<'a, Message: 'a> {
    /// Top bar — shader picker + description + reload button.
    pub top_bar: Element<'a, Message>,
    /// Compact input slot rows (stacked in the sidebar under "INPUTS").
    pub input_slots: Vec<Element<'a, Message>>,
    /// Output preview (fills the right panel entirely).
    pub output_widget: Element<'a, Message>,
    /// Parameter sliders + format pickers (under "OPTIONS" in sidebar).
    pub controls: Vec<Element<'a, Message>>,
    /// Save / Clear buttons — pinned to the bottom of the sidebar.
    pub buttons: Vec<Element<'a, Message>>,
    /// Thin status line at the very bottom of the window.
    pub status_bar: Element<'a, Message>,
}

/// Build the full baker layout.
pub fn create_baker_layout<'a, Message: 'a + Clone>(
    config: BakerLayoutConfig<'a, Message>,
) -> Element<'a, Message> {
    let has_inputs = !config.input_slots.is_empty();
    let has_controls = !config.controls.is_empty();
    let has_buttons = !config.buttons.is_empty();

    // ── Scrollable sidebar content (inputs + controls) ─────────────────────
    let mut scroll_col = Column::new().spacing(0).width(Length::Fill);

    if has_inputs {
        scroll_col = scroll_col.push(section_label("INPUTS"));
        let mut slots = Column::new().spacing(6).width(Length::Fill);
        for slot in config.input_slots {
            slots = slots.push(slot);
        }
        scroll_col = scroll_col.push(slots);
    }

    if has_controls {
        if has_inputs {
            scroll_col = scroll_col.push(
                container(horizontal_rule(1))
                    .padding(iced::Padding {
                        top: 12.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                    })
                    .width(Length::Fill),
            );
        }
        scroll_col = scroll_col.push(section_label("OPTIONS"));
        let mut ctrls = Column::new().spacing(14).width(Length::Fill);
        for ctrl in config.controls {
            ctrls = ctrls.push(ctrl);
        }
        scroll_col = scroll_col.push(ctrls);
    }

    let scroll_area = scrollable(
        container(scroll_col).padding(iced::Padding {
            top: 4.0,
            right: SIDEBAR_PAD,
            bottom: 8.0,
            left: SIDEBAR_PAD,
        }),
    )
    .height(Length::Fill)
    .style(crate::widget_helpers::scrollable_style);

    // ── Sidebar: scrollable area above, buttons pinned below ───────────────
    let mut sidebar_col = Column::new()
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill);

    sidebar_col = sidebar_col.push(scroll_area);

    if has_buttons {
        sidebar_col = sidebar_col.push(
            container(horizontal_rule(1))
                .padding(iced::Padding {
                    top: 0.0,
                    right: SIDEBAR_PAD,
                    bottom: 0.0,
                    left: SIDEBAR_PAD,
                })
                .width(Length::Fill),
        );
        let mut btn_col = Column::new().spacing(8).width(Length::Fill);
        for btn in config.buttons {
            btn_col = btn_col.push(btn);
        }
        sidebar_col = sidebar_col.push(
            container(btn_col).padding(iced::Padding {
                top: 8.0,
                right: SIDEBAR_PAD,
                bottom: 10.0,
                left: SIDEBAR_PAD,
            }),
        );
    }

    let left_sidebar = container(sidebar_col)
        .style(sidebar_style)
        .width(Length::FillPortion(3))
        .height(Length::Fill);

    // ── Output area ────────────────────────────────────────────────────────
    let right_output = container(
        container(config.output_widget)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(8),
    )
    .style(crate::widget_helpers::frame_style)
    .width(Length::FillPortion(5))
    .height(Length::Fill);

    // ── Main row ───────────────────────────────────────────────────────────
    let main_row = row![left_sidebar, right_output]
        .spacing(PANEL_GAP)
        .width(Length::Fill)
        .height(Length::Fill);

    // ── Full layout ────────────────────────────────────────────────────────
    column![config.top_bar, main_row, config.status_bar]
        .spacing(8)
        .padding(OUTER_PADDING)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Small ALL-CAPS accent label used as a section divider.
fn section_label<'a, Message: 'a>(label: &'a str) -> Element<'a, Message> {
    container(
        text(label).size(10).style(|theme: &Theme| iced::widget::text::Style {
            color: Some(Color {
                a: 0.55,
                ..theme.extended_palette().primary.base.color
            }),
        }),
    )
    .padding(iced::Padding {
        top: 10.0,
        right: 0.0,
        bottom: 6.0,
        left: 0.0,
    })
    .into()
}

fn sidebar_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(Color {
            a: 0.35,
            ..palette.background.strong.color
        })),
        border: Border {
            color: Color {
                a: 0.3,
                ..palette.background.strong.color
            },
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

// ── Helpers used by texture_converter ─────────────────────────────────────────

/// Output preview — image if available, placeholder text otherwise.
pub fn create_output_preview<'a, Message: 'a>(
    output_handle: &'a Option<iced::widget::image::Handle>,
    placeholder_text: &'a str,
) -> Element<'a, Message> {
    if let Some(handle) = output_handle {
        container(
            iced::widget::image(handle.clone())
                .content_fit(iced::ContentFit::Contain)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        container(
            text(placeholder_text)
                .size(13)
                .align_x(iced::alignment::Horizontal::Center)
                .style(|theme: &Theme| iced::widget::text::Style {
                    color: Some(Color {
                        a: 0.3,
                        ..theme.extended_palette().background.base.text
                    }),
                }),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }
}

/// Labeled slider: description on left, current value on right, slider below.
pub fn create_slider_control<'a, Message: 'a + Clone>(
    label: String,
    value: f64,
    range: std::ops::RangeInclusive<f64>,
    on_change: impl Fn(f64) -> Message + 'a,
) -> Element<'a, Message> {
    use crate::widget_helpers::slider_style;
    use iced::widget::{row, slider};

    column![
        row![
            text(label)
                .size(12)
                .width(Length::Fill)
                .style(|theme: &Theme| iced::widget::text::Style {
                    color: Some(theme.extended_palette().background.base.text),
                }),
            text(format!("{value:.2}"))
                .size(12)
                .style(|theme: &Theme| iced::widget::text::Style {
                    color: Some(Color {
                        a: 0.55,
                        ..theme.extended_palette().background.base.text
                    }),
                }),
        ]
        .align_y(Alignment::Center),
        slider(range, value, on_change)
            .step(0.01)
            .width(Length::Fill)
            .style(slider_style),
    ]
    .spacing(5)
    .into()
}

/// Green save button — disabled when there is nothing to save or already saving.
pub fn create_save_all_button<'a, Message: 'a + Clone>(
    is_saving: bool,
    has_output: bool,
    on_press: Message,
) -> Element<'a, Message> {
    use crate::widget_helpers::success_button_style;
    use iced::widget::button;

    let label = if is_saving { "Saving..." } else { "Save Outputs" };
    let btn = button(
        text(label)
            .align_x(iced::alignment::Horizontal::Center)
            .width(Length::Fill),
    )
    .padding([10, 16])
    .width(Length::Fill)
    .style(success_button_style);

    if has_output && !is_saving {
        btn.on_press(on_press).into()
    } else {
        btn.into()
    }
}

/// Red clear button — always enabled.
pub fn create_clear_button<'a, Message: 'a + Clone>(on_press: Message) -> Element<'a, Message> {
    use crate::widget_helpers::danger_button_style;
    use iced::widget::button;

    button(
        text("Clear")
            .align_x(iced::alignment::Horizontal::Center)
            .width(Length::Fill),
    )
    .padding([8, 16])
    .width(Length::Fill)
    .style(danger_button_style)
    .on_press(on_press)
    .into()
}
