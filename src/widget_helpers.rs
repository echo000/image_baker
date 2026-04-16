use iced::widget::{Column, Container, Row, button, column, container, pick_list, scrollable, slider, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme, alignment};

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
        radius: 6.0.into(),
    })
}

/// Dark background style
pub fn dark_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style::default()
        .background(palette.background.base.color)
        .border(Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: 6.0.into(),
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
            radius: 6.0.into(),
        })
}

/// Drop zone style for empty input slots — subtle inset with a dimmer border
pub fn drop_zone_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style::default()
        .background(Color {
            a: 0.25,
            ..palette.background.strong.color
        })
        .border(Border {
            color: Color {
                a: 0.45,
                ..palette.background.strong.color
            },
            width: 1.5,
            radius: 6.0.into(),
        })
}

/// Header style — slightly elevated surface for section headers
pub fn header_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style::default()
        .background(Color {
            a: 0.4,
            ..palette.background.strong.color
        })
        .border(Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: Border::default().radius,
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
                        a: 0.55,
                        ..palette.primary.base.color
                    },
                    width: 1.5,
                    radius: 6.0.into(),
                },
                ..base
            }
        }
        button::Status::Hovered => button::Style {
            background: Some(
                Color {
                    a: 0.18,
                    ..palette.primary.base.color
                }
                .into(),
            ),
            text_color: palette.primary.base.color,
            border: Border {
                color: Color {
                    a: 0.8,
                    ..palette.primary.base.color
                },
                width: 1.5,
                radius: 6.0.into(),
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
        button::Status::Active | button::Status::Pressed => button::Style {
            background: Some(
                Color {
                    a: 0.12,
                    ..palette.success.base.color
                }
                .into(),
            ),
            text_color: palette.success.base.color,
            border: Border {
                color: Color {
                    a: 0.55,
                    ..palette.success.base.color
                },
                width: 1.5,
                radius: 6.0.into(),
            },
            ..base
        },
        button::Status::Disabled => button::Style {
            background: None,
            text_color: Color {
                a: 0.3,
                ..palette.success.base.color
            },
            border: Border {
                color: Color {
                    a: 0.2,
                    ..palette.success.base.color
                },
                width: 1.5,
                radius: 6.0.into(),
            },
            ..base
        },
        button::Status::Hovered => button::Style {
            background: Some(
                Color {
                    a: 0.25,
                    ..palette.success.base.color
                }
                .into(),
            ),
            text_color: palette.success.base.color,
            border: Border {
                color: Color {
                    a: 0.85,
                    ..palette.success.base.color
                },
                width: 1.5,
                radius: 6.0.into(),
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
                text_color: Color {
                    a: 0.7,
                    ..palette.danger.base.color
                },
                border: Border {
                    color: Color {
                        a: 0.35,
                        ..palette.danger.base.color
                    },
                    width: 1.5,
                    radius: 6.0.into(),
                },
                ..base
            }
        }
        button::Status::Hovered => button::Style {
            background: Some(
                Color {
                    a: 0.15,
                    ..palette.danger.base.color
                }
                .into(),
            ),
            text_color: palette.danger.base.color,
            border: Border {
                color: Color {
                    a: 0.7,
                    ..palette.danger.base.color
                },
                width: 1.5,
                radius: 6.0.into(),
            },
            ..base
        },
    }
}

/// Pick list style
pub fn pick_list_style(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let palette = theme.extended_palette();

    let border_color = match status {
        pick_list::Status::Opened { .. } => palette.primary.base.color,
        _ => palette.background.strong.color,
    };

    pick_list::Style {
        text_color: palette.background.base.text,
        placeholder_color: Color {
            a: 0.5,
            ..palette.background.base.text
        },
        handle_color: Color {
            a: 0.7,
            ..palette.background.base.text
        },
        background: Color {
            a: 0.6,
            ..palette.background.strong.color
        }
        .into(),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 6.0.into(),
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
                    radius: 6.0.into(),
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
                radius: 6.0.into(),
            },
            ..base
        },
    }
}

/// Bottom bar for slot cards — slightly elevated tint, no explicit border needed
/// (the outer frame_style container provides the card border)
pub fn slot_bar_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(Color {
            a: 0.55,
            ..palette.background.strong.color
        })),
        border: Border {
            color: Color {
                a: 0.4,
                ..palette.background.strong.color
            },
            width: 1.0,
            radius: iced::border::Radius {
                top_left: 0.0,
                top_right: 0.0,
                bottom_right: 6.0,
                bottom_left: 6.0,
            },
        },
        ..Default::default()
    }
}

/// Thin bar at the bottom of the output panel (filename / nav) — same tint as slot bar
pub fn panel_bar_style(theme: &Theme) -> container::Style {
    slot_bar_style(theme)
}

/// Scrollable style — thin, subtle rail that matches the theme
pub fn scrollable_style(
    theme: &Theme,
    _status: scrollable::Status,
) -> scrollable::Style {
    let palette = theme.extended_palette();

    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: scrollable::Rail {
            background: Some(Background::Color(Color {
                a: 0.08,
                ..palette.background.strong.color
            })),
            border: Border {
                radius: 3.0.into(),
                ..Border::default()
            },
            scroller: scrollable::Scroller {
                color: Color {
                    a: 0.35,
                    ..palette.primary.base.color
                },
                border: Border {
                    radius: 3.0.into(),
                    ..Border::default()
                },
            },
        },
        horizontal_rail: scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller {
                color: Color::TRANSPARENT,
                border: Border::default(),
            },
        },
        gap: None,
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
