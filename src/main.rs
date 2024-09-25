mod backend;
mod config;
mod file;
mod frontend;
mod state;

use frontend::MainApp;
use iced::{Settings, Size};

fn main() -> iced::Result {
    let config_path = config::get_config_path();
    config::create_config_if_not_exists(config_path).unwrap();

    let setting = Settings::default();

    iced::application("musica", MainApp::update, MainApp::view)
        .settings(setting)
        .resizable(false)
        .window_size(Size::new(300.0, 600.0))
        .theme(MainApp::theme)
        .subscription(MainApp::subscription)
        .run()
}
