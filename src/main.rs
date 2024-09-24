mod backend;
mod config;
mod file;
mod frontend;
mod state;

use frontend::Player;
use iced::{Settings, Size};

fn main() -> iced::Result {
    let config_path = config::get_config_path();
    config::create_config_if_not_exists(config_path).unwrap();

    let setting = Settings::default();

    iced::application("musica", Player::update, Player::view)
        .settings(setting)
        .resizable(false)
        .window_size(Size::new(300.0, 600.0))
        .theme(Player::theme)
        .subscription(Player::subscription)
        .run()
}
