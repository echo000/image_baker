use std::path::PathBuf;

use iced::Theme;
use iced::keyboard;
use iced::keyboard::Modifiers;

use iced::widget::container;
use iced::widget::{button, column, horizontal_rule, pick_list, row, text};

use iced::window;
use iced::window::Mode;
use iced::window::Position;

use iced::Alignment;
use iced::Background;
use iced::Element;
use iced::Event;
use iced::Length;
use iced::Size;
use iced::Task;

use crate::AppState;
use crate::Message;
use crate::components::about::About;
use crate::components::baker::{Baker, BakerMessage};
use crate::components::detail_mapper::{DetailMapper, DetailMapperMessage};
use crate::messages::BakingMode;

/// Main window handler.
pub struct MainWindow {
    pub id: window::Id,
    pub baker: Baker,
    pub detail_mapper: DetailMapper,
    pub about: About,
    pub current_mode: BakingMode,
}

/// Messages produced by the main window.
#[derive(Debug, Clone)]
pub enum MainMessage {
    UI(Event),
    Show,
    Baker(BakerMessage),
    DetailMapper(DetailMapperMessage),
    ModeChanged(BakingMode),
    FontLoaded(Result<(), iced::font::Error>),
}

impl MainWindow {
    /// Creates a new main window.
    pub fn create() -> (Self, Task<window::Id>) {
        // Load window icon
        let icon = {
            let icon_bytes = include_bytes!("../../baker.ico");
            image::load_from_memory_with_format(icon_bytes, image::ImageFormat::Ico)
                .ok()
                .and_then(|img| {
                    let rgba = img.to_rgba8();
                    let (width, height) = rgba.dimensions();
                    window::icon::from_rgba(rgba.into_raw(), width, height).ok()
                })
        };

        let (id, task) = window::open(window::Settings {
            size: Size::new(780.0, 720.0),
            position: Position::Centered,
            min_size: Some(Size::new(780.0, 720.0)),
            visible: false,
            resizable: false,
            icon,
            ..Default::default()
        });

        (
            Self {
                id,
                baker: Baker::new(),
                detail_mapper: DetailMapper::new(),
                about: About::new(),
                current_mode: BakingMode::SpecGlossPacker,
            },
            task,
        )
    }

    /// Handles the title of the main window.
    pub fn title(&self, state: &AppState) -> String {
        format!("{} v{}", state.name, state.version)
    }

    /// Handles updates for the main window.
    pub fn update(&mut self, state: &mut AppState, message: MainMessage) -> Task<Message> {
        use MainMessage::*;

        match message {
            UI(event) => self.on_ui(state, event),
            Show => self.on_show(),
            Baker(message) => self.baker.update(message),
            DetailMapper(message) => self.detail_mapper.update(message),
            ModeChanged(mode) => self.on_mode_changed(mode),
            FontLoaded(result) => {
                if let Err(e) = result {
                    tracing::error!("Failed to load font: {:#?}", e);
                }
                Task::none()
            }
        }
    }

    /// Handles rendering the main window.
    pub fn view<'a>(&'a self, state: &'a AppState) -> Element<'a, Message> {
        // Tab buttons
        let spec_gloss_tab = button(text("C/S/O Baker"))
            .on_press(Message::Main(MainMessage::ModeChanged(
                BakingMode::SpecGlossPacker,
            )))
            .style(if self.current_mode == BakingMode::SpecGlossPacker {
                crate::widget_helpers::primary_button_style
            } else {
                crate::widget_helpers::secondary_button_style
            })
            .width(Length::Fill);

        let detail_map_tab = button(text("Detail Baker"))
            .on_press(Message::Main(MainMessage::ModeChanged(
                BakingMode::DetailMapper,
            )))
            .style(if self.current_mode == BakingMode::DetailMapper {
                crate::widget_helpers::primary_button_style
            } else {
                crate::widget_helpers::secondary_button_style
            })
            .width(Length::Fill);

        let about_tab = button(text("About"))
            .on_press(Message::Main(MainMessage::ModeChanged(BakingMode::About)))
            .style(if self.current_mode == BakingMode::About {
                crate::widget_helpers::primary_button_style
            } else {
                crate::widget_helpers::secondary_button_style
            })
            .width(Length::Fill);

        let tab_row = row![spec_gloss_tab, detail_map_tab, about_tab]
            .spacing(5)
            .padding([10.0, 20.0]);

        // Content based on current mode
        let content_view = match self.current_mode {
            BakingMode::SpecGlossPacker => self
                .baker
                .view()
                .map(|msg| Message::Main(MainMessage::Baker(msg))),
            BakingMode::DetailMapper => self
                .detail_mapper
                .view()
                .map(|msg| Message::Main(MainMessage::DetailMapper(msg))),
            BakingMode::About => self.about.view(state),
        };

        // Footer with version and theme picker
        let theme = state.settings.theme.to_iced_theme();
        let version = text(format!("Image Baker v{}.", state.version))
            .color(theme.extended_palette().primary.base.color);

        let theme_picker = pick_list(
            crate::theme::AppTheme::ALL,
            Some(state.settings.theme),
            Message::ThemeChanged,
        )
        .placeholder("Theme")
        .style(crate::widget_helpers::pick_list_style);

        let footer = column![
            horizontal_rule(1),
            row![
                column![version]
                    .width(Length::Fill)
                    .align_x(Alignment::Start),
                column![theme_picker]
                    .width(Length::Fill)
                    .align_x(Alignment::End)
            ]
            .padding([5, 20])
            .align_y(Alignment::Center)
        ]
        .align_x(Alignment::Center);

        let content = column![
            tab_row,
            container(content_view)
                .width(Length::Fill)
                .height(Length::Fill),
            footer
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        container(content)
            .style(main_background_style)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Occurs when a ui event has fired.
    fn on_ui(&mut self, state: &mut AppState, event: Event) -> Task<Message> {
        match event {
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifier_keys)) => {
                self.on_modifiers_changed(state, modifier_keys)
            }
            Event::Window(window::Event::Opened { .. }) => self.on_opened(),
            Event::Window(window::Event::Closed) => self.on_closed(),
            Event::Window(window::Event::FileHovered(_)) => self.on_file_hovered(state),
            Event::Window(window::Event::FilesHoveredLeft) => self.on_files_hovered_left(state),
            Event::Window(window::Event::FileDropped(path)) => self.on_file_dropped(state, path),
            _ => Task::none(),
        }
    }

    /// Occurs when the modifier keys change.
    fn on_modifiers_changed(
        &mut self,
        state: &mut AppState,
        modifier_keys: Modifiers,
    ) -> Task<Message> {
        state.modifier_keys = modifier_keys;
        Task::none()
    }

    /// Occurs when the window has opened.
    fn on_opened(&mut self) -> Task<Message> {
        Task::done(Message::WindowOpened(self.id))
    }

    /// Occurs when the window is closed.
    fn on_closed(&mut self) -> Task<Message> {
        iced::exit()
    }

    /// Occurs when a file is hovered over the window.
    fn on_file_hovered(&mut self, state: &mut AppState) -> Task<Message> {
        state.file_hovered = true;
        Task::none()
    }

    /// Occurs when files stop being hovered over the window.
    fn on_files_hovered_left(&mut self, state: &mut AppState) -> Task<Message> {
        state.file_hovered = false;
        Task::none()
    }

    /// Occurs when a file has been dropped onto the window.
    fn on_file_dropped(&mut self, state: &mut AppState, path: PathBuf) -> Task<Message> {
        if state.is_busy() {
            return Task::none();
        }

        let result = match self.current_mode {
            BakingMode::SpecGlossPacker => {
                // Use filename suffix to determine image type
                if let Some(img_type) = crate::core_logic::identify_image_type(&path) {
                    self.baker.on_file_dropped(path, img_type)
                } else {
                    self.baker.on_unknown_file_dropped(path)
                }
            }
            BakingMode::DetailMapper => {
                // Use filename suffix to determine image type (_c for base colour, _d for detail)
                if let Some(img_type) = crate::core_logic::identify_image_type(&path) {
                    self.detail_mapper.on_file_dropped(path, img_type)
                } else {
                    self.detail_mapper.on_unknown_file_dropped(path)
                }
            }
            BakingMode::About => {
                // No file drops on About tab
                Task::none()
            }
        };

        // Clear file hovered state after processing drop
        state.file_hovered = false;

        result
    }

    /// Occurs when the mode is changed.
    fn on_mode_changed(&mut self, mode: BakingMode) -> Task<Message> {
        self.current_mode = mode;
        Task::none()
    }

    /// Shows the main window.
    fn on_show(&mut self) -> Task<Message> {
        window::set_mode(self.id, Mode::Windowed)
    }
}

/// Style for the main background.
fn main_background_style(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(
            theme.extended_palette().background.base.color,
        )),
        ..Default::default()
    }
}

impl From<BakerMessage> for MainMessage {
    fn from(message: BakerMessage) -> Self {
        MainMessage::Baker(message)
    }
}

impl From<DetailMapperMessage> for MainMessage {
    fn from(message: DetailMapperMessage) -> Self {
        MainMessage::DetailMapper(message)
    }
}
