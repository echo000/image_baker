use crate::windows::MainMessage;

/// Top-level application messages.
#[derive(Debug, Clone)]
pub enum Message {
    Noop,
    UI(iced::Event, iced::window::Id),
    WindowOpened(iced::window::Id),
    Controller(crate::Controller),
    Main(MainMessage),
    ThemeChanged(crate::theme::AppTheme),
    SettingsLoaded(crate::Settings),
    SettingsSaved,
}

impl From<MainMessage> for Message {
    fn from(message: MainMessage) -> Self {
        Message::Main(message)
    }
}
