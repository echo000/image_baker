use iced::keyboard::Modifiers;
use std::path::PathBuf;

use crate::{Controller, Settings};

/// Global application state shared across windows and components.
pub struct AppState {
    pub name: &'static str,
    pub version: &'static str,
    pub controller: Controller,
    pub settings: Settings,
    pub modifier_keys: Modifiers,
    pub files_dropped: Vec<PathBuf>,
    pub loading: bool,
    pub file_hovered: bool,
}

impl AppState {
    /// Constructs a new application state.
    pub fn new() -> Self {
        AppState {
            name: "Image Baker",
            version: env!("CARGO_PKG_VERSION"),
            controller: Controller::new(),
            settings: Settings::default(),
            modifier_keys: Modifiers::default(),
            files_dropped: Vec::new(),
            loading: false,
            file_hovered: false,
        }
    }

    /// Whether the application is busy (loading, etc).
    pub fn is_busy(&self) -> bool {
        self.loading
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
