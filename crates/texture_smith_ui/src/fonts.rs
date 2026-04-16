use iced::Task;
use iced::font::{self, Font};

pub static JETBRAINS_MONO: Font = Font::with_name("JetBrains Mono NL");

pub mod bytes {
    pub static JETBRAINS_MONO: &[u8] = include_bytes!("./fonts/JetBrainsMonoNL-Regular.ttf");
}

pub fn load() -> Task<Result<(), font::Error>> {
    Task::batch([font::load(bytes::JETBRAINS_MONO)])
}
