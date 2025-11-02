use crate::theme::AppTheme;
use bincode::{Decode, Encode};
use directories::ProjectDirs;

#[derive(Debug, Clone, Encode, Decode, Default)]
pub struct Settings {
    pub theme: AppTheme,
}

impl Settings {
    /// Loads settings from disk
    pub fn load() -> Self {
        let Some(project_directory) = ProjectDirs::from("com", "echo000", "ImageBaker") else {
            return Default::default();
        };

        std::fs::read(project_directory.config_dir().join("settings.dat")).map_or(
            Default::default(),
            |buffer| {
                let config = bincode::config::standard();

                bincode::decode_from_slice(&buffer, config)
                    .unwrap_or_default()
                    .0
            },
        )
    }

    /// Saves settings to disk
    pub fn save(&self) {
        let Some(project_directory) = ProjectDirs::from("com", "echo000", "ImageBaker") else {
            return;
        };

        let config = bincode::config::standard();

        let Ok(result) = bincode::encode_to_vec(self, config) else {
            return;
        };

        let dirs = std::fs::create_dir_all(project_directory.config_dir());

        debug_assert!(dirs.is_ok());

        let result = std::fs::write(project_directory.config_dir().join("settings.dat"), result);
        debug_assert!(result.is_ok());
    }
}
