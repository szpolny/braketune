use std::{fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::scaling::{MAX_PERCENT, MIN_PERCENT};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Settings {
    pub brake_force: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            brake_force: MIN_PERCENT,
        }
    }
}

impl Settings {
    pub fn load(path: &PathBuf) -> Self {
        let mut settings: Self = fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default();
        settings.brake_force = settings.brake_force.clamp(MIN_PERCENT, MAX_PERCENT);
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
        let saved = Settings { brake_force: 70 };
        saved.save(&path).unwrap();
        assert_eq!(Settings::load(&path), saved);

        fs::write(&path, r#"{"brake_force": 250}"#).unwrap();
        assert_eq!(Settings::load(&path).brake_force, MAX_PERCENT);
        let _ = fs::remove_dir_all(dir);
    }
}
