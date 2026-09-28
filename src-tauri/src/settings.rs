use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub interval_minutes: u64,
    pub notifications: bool,
    pub geo_lookup: bool,
    pub autostart: bool,
    pub start_minimized: bool,
    pub track_ipv6: bool,
    pub paused: bool,
    pub theme: String,       // "system" | "light" | "dark"
    pub retention_days: u32, // 0 = keep forever
    pub webhook_url: String,
    pub hook_command: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            interval_minutes: 5,
            notifications: true,
            geo_lookup: true,
            autostart: false,
            start_minimized: false,
            track_ipv6: true,
            paused: false,
            theme: "system".into(),
            retention_days: 0,
            webhook_url: String::new(),
            hook_command: String::new(),
        }
    }
}

impl Settings {
    pub fn path(dir: &Path) -> PathBuf {
        dir.join("settings.json")
    }

    pub fn load(dir: &Path) -> Self {
        std::fs::read_to_string(Self::path(dir)).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    }

    pub fn save(&self, dir: &Path) -> Result<(), String> {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(Self::path(dir), json).map_err(|e| e.to_string())
    }

    pub fn interval_secs(&self) -> u64 {
        self.interval_minutes.clamp(1, 24 * 60) * 60
    }
}
