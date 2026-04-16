use bincode::{Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Encode, Decode)]
pub enum AppTheme {
    Dark,
    Light,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    #[default]
    Dracula,
    Nord,
    SolarizedLight,
    SolarizedDark,
    GruvboxLight,
    GruvboxDark,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightLight,
    KanagawaWave,
    KanagawaDragon,
    KanagawaLotus,
    Moonfly,
    Nightfly,
    Oxocarbon,
    Ferra,
}

impl AppTheme {
    pub const ALL: &'static [AppTheme] = &[
        AppTheme::Dark,
        AppTheme::Light,
        AppTheme::CatppuccinLatte,
        AppTheme::CatppuccinFrappe,
        AppTheme::CatppuccinMacchiato,
        AppTheme::CatppuccinMocha,
        AppTheme::Dracula,
        AppTheme::Nord,
        AppTheme::SolarizedLight,
        AppTheme::SolarizedDark,
        AppTheme::GruvboxLight,
        AppTheme::GruvboxDark,
        AppTheme::TokyoNight,
        AppTheme::TokyoNightStorm,
        AppTheme::TokyoNightLight,
        AppTheme::KanagawaWave,
        AppTheme::KanagawaDragon,
        AppTheme::KanagawaLotus,
        AppTheme::Moonfly,
        AppTheme::Nightfly,
        AppTheme::Oxocarbon,
        AppTheme::Ferra,
    ];
}

impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AppTheme::Dark => "Dark",
                AppTheme::Light => "Light",
                AppTheme::CatppuccinLatte => "Catppuccin Latte",
                AppTheme::CatppuccinFrappe => "Catppuccin Frappé",
                AppTheme::CatppuccinMacchiato => "Catppuccin Macchiato",
                AppTheme::CatppuccinMocha => "Catppuccin Mocha",
                AppTheme::Dracula => "Dracula",
                AppTheme::Nord => "Nord",
                AppTheme::SolarizedLight => "Solarized Light",
                AppTheme::SolarizedDark => "Solarized Dark",
                AppTheme::GruvboxLight => "Gruvbox Light",
                AppTheme::GruvboxDark => "Gruvbox Dark",
                AppTheme::TokyoNight => "Tokyo Night",
                AppTheme::TokyoNightStorm => "Tokyo Night Storm",
                AppTheme::TokyoNightLight => "Tokyo Night Light",
                AppTheme::KanagawaWave => "Kanagawa Wave",
                AppTheme::KanagawaDragon => "Kanagawa Dragon",
                AppTheme::KanagawaLotus => "Kanagawa Lotus",
                AppTheme::Moonfly => "Moonfly",
                AppTheme::Nightfly => "Nightfly",
                AppTheme::Oxocarbon => "Oxocarbon",
                AppTheme::Ferra => "Ferra",
            }
        )
    }
}

impl AppTheme {
    pub fn to_iced_theme(self) -> iced::Theme {
        match self {
            AppTheme::Dark => iced::Theme::Dark,
            AppTheme::Light => iced::Theme::Light,
            AppTheme::CatppuccinLatte => iced::Theme::CatppuccinLatte,
            AppTheme::CatppuccinFrappe => iced::Theme::CatppuccinFrappe,
            AppTheme::CatppuccinMacchiato => iced::Theme::CatppuccinMacchiato,
            AppTheme::CatppuccinMocha => iced::Theme::CatppuccinMocha,
            AppTheme::Dracula => iced::Theme::Dracula,
            AppTheme::Nord => iced::Theme::Nord,
            AppTheme::SolarizedLight => iced::Theme::SolarizedLight,
            AppTheme::SolarizedDark => iced::Theme::SolarizedDark,
            AppTheme::GruvboxLight => iced::Theme::GruvboxLight,
            AppTheme::GruvboxDark => iced::Theme::GruvboxDark,
            AppTheme::TokyoNight => iced::Theme::TokyoNight,
            AppTheme::TokyoNightStorm => iced::Theme::TokyoNightStorm,
            AppTheme::TokyoNightLight => iced::Theme::TokyoNightLight,
            AppTheme::KanagawaWave => iced::Theme::KanagawaWave,
            AppTheme::KanagawaDragon => iced::Theme::KanagawaDragon,
            AppTheme::KanagawaLotus => iced::Theme::KanagawaLotus,
            AppTheme::Moonfly => iced::Theme::Moonfly,
            AppTheme::Nightfly => iced::Theme::Nightfly,
            AppTheme::Oxocarbon => iced::Theme::Oxocarbon,
            AppTheme::Ferra => iced::Theme::Ferra,
        }
    }
}
