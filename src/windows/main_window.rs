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
use crate::components::texture_converter::{TextureSplitter, TextureSplitterMessage};

/// Main window handler.
pub struct MainWindow {
    pub id: window::Id,
    pub texture_splitter: TextureSplitter,
    pub about: About,
    pub show_about: bool,
}

/// Messages produced by the main window.
#[derive(Debug, Clone)]
pub enum MainMessage {
    UI(Event),
    Show,
    TextureSplitter(TextureSplitterMessage),
    ShowAbout,
    HideAbout,
    FontLoaded(Result<(), iced::font::Error>),
}

impl MainWindow {
    /// Creates a new main window.
    pub fn create() -> (Self, Task<window::Id>) {
        // Load window icon

        let (id, task) = window::open(window::Settings {
            size: Size::new(780.0, 720.0),
            position: Position::Centered,
            min_size: Some(Size::new(780.0, 720.0)),
            visible: false,
            resizable: false,
            //icon,
            ..Default::default()
        });

        (
            Self {
                id,
                texture_splitter: TextureSplitter::new(),
                about: About::new(),
                show_about: false,
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
            TextureSplitter(message) => self.texture_splitter.update(message),
            ShowAbout => {
                self.show_about = true;
                Task::none()
            }
            HideAbout => {
                self.show_about = false;
                Task::none()
            }
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
        // Content - show either texture converter or about page
        let content_view = if self.show_about {
            self.about.view(state)
        } else {
            self.texture_splitter
                .view()
                .map(|msg| Message::Main(MainMessage::TextureSplitter(msg)))
        };

        // Footer with about button and theme picker
        let about_button = if self.show_about {
            button(text("Back to Converter"))
                .on_press(Message::Main(MainMessage::HideAbout))
                .style(crate::widget_helpers::primary_button_style)
        } else {
            button(text("About"))
                .on_press(Message::Main(MainMessage::ShowAbout))
                .style(crate::widget_helpers::primary_button_style)
        };

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
                column![about_button]
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
        Task::batch([
            Task::done(Message::WindowOpened(self.id)),
            TextureSplitter::initialize(),
        ])
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

        // Only handle drops if not showing about page
        let result = if self.show_about {
            Task::none()
        } else {
            // Texture converter accepts any image file
            self.texture_splitter.on_file_dropped(path)
        };

        // Clear file hovered state after processing drop
        state.file_hovered = false;

        result
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

impl From<TextureSplitterMessage> for MainMessage {
    fn from(message: TextureSplitterMessage) -> Self {
        MainMessage::TextureSplitter(message)
    }
}
