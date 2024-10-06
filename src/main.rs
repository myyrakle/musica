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

    let mut setting = Settings::default();
    setting.id = Some("musica".into());

    let mut window_setting = iced::window::Settings::default();
    window_setting.platform_specific.application_id = "musica".into();

    iced::application("musica", MainApp::update, MainApp::view)
        .settings(setting)
        .window(window_setting)
        .resizable(false)
        .window_size(Size::new(300.0, 600.0))
        .theme(MainApp::theme)
        .subscription(MainApp::subscription)
        .run()
}
