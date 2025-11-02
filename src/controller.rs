use crate::Message;
use iced::futures::channel::mpsc;

/// Controller for sending messages from background tasks to the UI.
#[derive(Debug, Clone)]
pub struct Controller {
    tx: Option<mpsc::UnboundedSender<Message>>,
}

impl Controller {
    /// Creates a new controller without a channel.
    pub fn new() -> Self {
        Self { tx: None }
    }

    /// Creates a new controller with the given message channel.
    pub fn with_channel(tx: mpsc::UnboundedSender<Message>) -> Self {
        Self { tx: Some(tx) }
    }

    /// Sends a message to the UI thread.
    pub fn send(&self, message: Message) {
        if let Some(tx) = &self.tx {
            let _ = tx.unbounded_send(message);
        }
    }

    /// Notifies that settings have been loaded.
    pub fn settings_loaded(&self, settings: crate::Settings) {
        self.send(Message::SettingsLoaded(settings));
    }

    /// Notifies that settings have been saved.
    pub fn settings_saved(&self) {
        self.send(Message::SettingsSaved);
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}
