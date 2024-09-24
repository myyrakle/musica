use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Config {
    #[serde(default)]
    pub is_random: bool, // 랜덤 재생 여부
    #[serde(default)]
    pub directory_path: PathBuf, // 재생할 기본 경로
}

impl Config {
    pub fn update_config_if_exists(&self, path: PathBuf) -> anyhow::Result<()> {
        let config_str = serde_json::to_string(self)?;

        fs::write(path, config_str)?;

        Ok(())
    }
}

fn get_app_data_path() -> PathBuf {
    let home_dir = env::var("HOME").unwrap_or_else(|_| String::from(""));

    #[cfg(target_os = "windows")]
    let app_data_path = PathBuf::from(format!(r"{}\AppData\Local\musica", home_dir));

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

    let config: Config = serde_json::from_str(&config_str)?;

    Ok(config)
}
