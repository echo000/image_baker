#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub(crate) mod components;
pub(crate) mod fonts;

mod app;
mod app_state;
mod controller;
mod core_logic;
mod executor;
mod logger;
mod messages;
mod panic_hook;
mod porter_image;
mod settings;
mod status;
mod system;
mod theme;
#[allow(dead_code)]
mod widget_helpers;
mod windows;

pub(crate) use app::*;
pub(crate) use messages::*;
pub(crate) use windows::*;

pub use app_state::*;
pub use controller::*;
pub use settings::*;

fn main() -> iced::Result {
    App::launch()
}
