use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub directory_path: PathBuf, // 재생할 기본 경로
}

pub fn get_app_data_path() -> PathBuf {
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

pub fn create_config_if_not_exists(path: PathBuf) -> anyhow::Result<()> {
    if !path.exists() {
        let config = Config {
            directory_path: env::current_dir()?,
        };

        let config_str = serde_json::to_string(&config)?;

        fs::write(path, config_str)?;

        Ok(())
    } else {
        Ok(())
    }
}
