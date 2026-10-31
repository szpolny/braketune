use std::{fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::protocol::{MAX_BRAKE_PERCENT, MIN_BRAKE_PERCENT};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Settings {
    pub brake_force: u8,
    pub apply_on_connect: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            brake_force: 50,
            apply_on_connect: true,
        }
    }
}

impl Settings {
    pub fn load(path: &PathBuf) -> Self {
        let mut settings: Self = fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default();
        settings.brake_force = settings
            .brake_force
            .clamp(MIN_BRAKE_PERCENT, MAX_BRAKE_PERCENT);
        settings
    }

    pub fn save(&self, path: &PathBuf) -> io::Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let raw = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        fs::write(path, raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_yields_defaults() {
        let path = PathBuf::from("/nonexistent/braketune/settings.json");
        assert_eq!(Settings::load(&path), Settings::default());
    }

    #[test]
    fn roundtrips_and_clamps() {
        let dir = std::env::temp_dir().join(format!("braketune-test-{}", std::process::id()));
        let path = dir.join("settings.json");
        let saved = Settings {
            brake_force: 70,
            apply_on_connect: false,
        };
        saved.save(&path).unwrap();
        assert_eq!(Settings::load(&path), saved);

        fs::write(&path, r#"{"brake_force": 250}"#).unwrap();
        assert_eq!(Settings::load(&path).brake_force, MAX_BRAKE_PERCENT);
        let _ = fs::remove_dir_all(dir);
    }
}
