use iced::widget::{Column, Container, Row, button, column, container, pick_list, slider, text};
use iced::{Alignment, Border, Color, Element, Length, Theme, alignment};

/// Creates a centered text widget
pub fn centered_text(input: impl Into<String>) -> iced::widget::Text<'static> {
    text(input.into())
        .align_x(alignment::Horizontal::Center)
        .width(Length::Fill)
}

/// Creates a container that centers its content both horizontally and vertically
pub fn centered_container<'a, Message>(content: Element<'a, Message>) -> Container<'a, Message> {
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
}

/// Creates a container that fills all available space
pub fn fill_container<'a, Message>(content: Element<'a, Message>) -> Container<'a, Message> {
    container(content).width(Length::Fill).height(Length::Fill)
}

/// A titled frame container
pub fn control<'a, Message: 'a>(
    title: Element<'a, Message>,
    content: Element<'a, Message>,
) -> Container<'a, Message> {
    container(
        column![
            title,
            container(content)
                .padding(8)
                .style(frame_style)
                .width(Length::Fill),
        ]
        .spacing(8),
    )
}

/// Control helper that fills all space
pub fn control_filled<'a, Message: 'a>(
    title: Element<'a, Message>,
    content: Element<'a, Message>,
) -> Container<'a, Message> {
    fill_container(
        column![
            title,
            container(content)
                .padding(8)
                .style(frame_style)
                .width(Length::Fill)
                .height(Length::Fill),
        ]
        .spacing(8)
        .into(),
    )
}

/// Creates a column with center alignment
pub fn centered_column<'a, Message>(col: Column<'a, Message>) -> Column<'a, Message> {
    col.spacing(5)
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill)
}

/// Creates a column with horizontal center alignment only
pub fn centered_column_x<'a, Message>(col: Column<'a, Message>) -> Column<'a, Message> {
    col.spacing(5)
        .align_x(Alignment::Center)
        .width(Length::Fill)
}

/// Creates a row with center alignment and spacing
pub fn spaced_row<'a, Message: 'a>(r: Row<'a, Message>) -> Row<'a, Message> {
    r.align_y(Alignment::Center).spacing(5)
}

/// Frame style for containers
pub fn frame_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style::default().border(Border {
        color: palette.background.strong.color,
        width: 1.0,
        radius: 5.0.into(),
    })
}

/// Dark background style
pub fn dark_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style::default()
        .background(palette.background.base.color)
        .border(Border {
            color: palette.background.weak.color,
            width: 1.5,
            radius: 5.0.into(),
        })
}

/// Hovered overlay style for drag-and-drop
pub fn hovered_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    let accent = palette.primary.base.color;

    container::Style::default()
        .background(Color { a: 0.3, ..accent })
        .border(Border {
            color: Color { a: 0.8, ..accent },
            width: 2.0,
            radius: 5.0.into(),
        })
}

/// Primary button style
pub fn primary_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let base = button::Style::default();

    match status {
        button::Status::Active | button::Status::Pressed | button::Status::Disabled => {
            button::Style {
                background: None,
                text_color: palette.primary.base.color,
                border: Border {
                    color: Color {
                        a: 0.5,
                        ..palette.primary.base.color
                    },
                    width: 1.5,
                    radius: 5.0.into(),
                },
                ..base
            }
        }
        button::Status::Hovered => button::Style {
            background: Some(
                Color {
                    a: 0.4,
                    ..palette.primary.base.color
                }
                .into(),
            ),
            text_color: palette.background.base.text,
            border: Border {
                color: Color {
                    a: 0.5,
                    ..palette.primary.base.color
                },
                width: 1.5,
                radius: 5.0.into(),
            },
            ..base
        },
    }
}

/// Success/Start button style (green)
pub fn success_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let base = button::Style::default();

    match status {
        button::Status::Active | button::Status::Pressed | button::Status::Disabled => {
            button::Style {
                background: None,
                text_color: palette.success.base.color,
                border: Border {
                    color: Color {
                        a: 0.5,
                        ..palette.success.base.color
                    },
                    width: 1.5,
                    radius: 5.0.into(),
                },
                ..base
            }
        }
        button::Status::Hovered => button::Style {
            background: Some(
                Color {
                    a: 0.4,
                    ..palette.success.base.color
                }
                .into(),
            ),
            text_color: palette.background.base.text,
            border: Border {
                color: Color {
                    a: 0.5,
                    ..palette.success.base.color
                },
                width: 1.5,
                radius: 5.0.into(),
            },
            ..base
        },
    }
}

/// Danger/Cancel button style (red)
pub fn danger_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let base = button::Style::default();

    match status {
        button::Status::Active | button::Status::Pressed | button::Status::Disabled => {
            button::Style {
                background: None,
                text_color: palette.danger.base.color,
                border: Border {
                    color: Color {
                        a: 0.5,
                        ..palette.danger.base.color
                    },
                    width: 1.5,
                    radius: 5.0.into(),
                },
                ..base
            }
        }
        button::Status::Hovered => button::Style {
            background: Some(
                Color {
                    a: 0.4,
                    ..palette.danger.base.color
                }
                .into(),
            ),
            text_color: palette.background.base.text,
            border: Border {
                color: Color {
                    a: 0.5,
                    ..palette.danger.base.color
                },
                width: 1.5,
                radius: 5.0.into(),
            },
            ..base
        },
    }
}

/// Pick list style
pub fn pick_list_style(theme: &Theme, _status: pick_list::Status) -> pick_list::Style {
    let palette = theme.extended_palette();

    pick_list::Style {
        text_color: palette.background.base.text,
        placeholder_color: palette.background.weak.text,
        handle_color: palette.background.base.text,
        background: palette.background.base.color.into(),
        border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: 2.0.into(),
        },
    }
}

/// Secondary button style (for inactive tabs)
pub fn secondary_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let base = button::Style::default();

    match status {
        button::Status::Active | button::Status::Pressed | button::Status::Disabled => {
            button::Style {
                background: None,
                text_color: palette.background.base.text,
                border: Border {
                    color: Color {
                        a: 0.3,
                        ..palette.background.strong.color
                    },
                    width: 1.0,
                    radius: 5.0.into(),
                },
                ..base
            }
        }
        button::Status::Hovered => button::Style {
            background: Some(
                Color {
                    a: 0.2,
                    ..palette.background.strong.color
                }
                .into(),
            ),
            text_color: palette.background.base.text,
            border: Border {
                color: Color {
                    a: 0.5,
                    ..palette.background.strong.color
                },
                width: 1.0,
                radius: 5.0.into(),
            },
            ..base
        },
    }
}

/// Slider style
pub fn slider_style(theme: &Theme, _status: slider::Status) -> slider::Style {
    let palette = theme.extended_palette();

    slider::Style {
        rail: slider::Rail {
            backgrounds: (
                palette.primary.base.color.into(),
                palette.background.strong.color.into(),
            ),
            width: 4.0,
            border: Border::default(),
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle { radius: 8.0 },
            background: palette.primary.base.color.into(),
            border_color: Color::TRANSPARENT,
            border_width: 0.0,
        },
    }
}
