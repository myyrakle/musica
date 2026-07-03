use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    #[serde(default)]
    pub is_random: bool, // 랜덤 재생 여부
    #[serde(default)]
    pub directory_path: PathBuf, // 재생할 기본 경로
    #[serde(default = "Config::default_volume")]
    pub volume: f32, // 볼륨 (0.0 - 1.0)
}

impl Config {
    pub const DEFAULT_VOLUME: f32 = 1.0;

    fn default_volume() -> f32 {
        Self::DEFAULT_VOLUME
    }

    pub fn normalize_volume(volume: f32) -> f32 {
        volume.clamp(0.0, 1.0)
    }

    pub fn update_config_if_exists(&self, path: PathBuf) -> anyhow::Result<()> {
        let mut config = self.clone();
        config.volume = Self::normalize_volume(config.volume);

        let config_str = serde_json::to_string(&config)?;

        fs::write(path, config_str)?;

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            is_random: false,
            directory_path: PathBuf::default(),
            volume: Self::DEFAULT_VOLUME,
        }
    }
}

fn get_app_data_path() -> PathBuf {
    // ------- Windows Only
    #[cfg(target_os = "windows")]
    let app_data_path = PathBuf::from(r"\AppData\Local\musica");
    // Windows Only -------

    #[cfg(not(any(target_os = "windows")))]
    let home_dir = env::var("HOME").unwrap_or_else(|_| String::from(""));

    #[cfg(target_os = "macos")]
    let app_data_path = PathBuf::from(format!("{}/Library/Application Support/musica", home_dir));

    #[cfg(target_os = "linux")]
    let app_data_path = PathBuf::from(format!("{}/.local/share/musica", home_dir));

    // Ensure the directory exists
    if !app_data_path.exists() {
        fs::create_dir_all(&app_data_path).expect("Failed to create app data directory");
    }

    app_data_path
}

pub fn get_config_path() -> PathBuf {
    let app_data_path = get_app_data_path();

    app_data_path.join("config.json")
}

pub fn create_config_if_not_exists(path: PathBuf) -> anyhow::Result<()> {
    if !path.exists() {
        let config = Config {
            directory_path: env::current_dir()?,
            is_random: false,
            volume: Config::DEFAULT_VOLUME,
        };

        let config_str = serde_json::to_string(&config)?;

        fs::write(path, config_str)?;

        Ok(())
    } else {
        Ok(())
    }
}

pub fn read_config_if_exists(path: PathBuf) -> anyhow::Result<Config> {
    let config_str = fs::read_to_string(path)?;

    let mut config: Config = serde_json::from_str(&config_str)?;
    config.volume = Config::normalize_volume(config.volume);

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_legacy_config_with_default_volume() {
        let config: Config =
            serde_json::from_str(r#"{"is_random":true,"directory_path":"/tmp/music"}"#).unwrap();

        assert_eq!(Config::DEFAULT_VOLUME, config.volume);
    }

    #[test]
    fn default_config_uses_default_volume() {
        assert_eq!(Config::DEFAULT_VOLUME, Config::default().volume);
    }

    #[test]
    fn clamps_volume_to_supported_range() {
        assert_eq!(0.0, Config::normalize_volume(-0.5));
        assert_eq!(0.5, Config::normalize_volume(0.5));
        assert_eq!(1.0, Config::normalize_volume(1.5));
    }
}
