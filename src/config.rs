use std::env;
use std::fs;
use std::path::PathBuf;

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
