use iced::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Success,
    Warning,
    Error,
}

impl Severity {
    /// Get the colour for this severity level
    pub fn colour(&self) -> Color {
        match self {
            Severity::Info => Color::from_rgb(0.7, 0.7, 0.7), // Light gray
            Severity::Success => Color::from_rgb(0.4, 0.8, 0.4), // Green
            Severity::Warning => Color::from_rgb(1.0, 0.7, 0.0), // Orange/Yellow
            Severity::Error => Color::from_rgb(0.9, 0.3, 0.3), // Red
        }
    }
}

#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub message: String,
    pub severity: Severity,
}

impl StatusMessage {
    pub fn new(message: impl Into<String>, severity: Severity) -> Self {
        Self {
            message: message.into(),
            severity,
        }
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new(message, Severity::Info)
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self::new(message, Severity::Success)
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(message, Severity::Warning)
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(message, Severity::Error)
    }

    pub fn colour(&self) -> Color {
        self.severity.colour()
    }
}

impl Default for StatusMessage {
    fn default() -> Self {
        Self::info("Image Baker Initialized.")
    }
}
