mod dialog;
mod modal;

use std::sync::LazyLock;
use std::time::{Duration, Instant};
use std::u8;

use crate::state::{MainState, Music, MusicList};
use config::Config;
use iced::widget::{self, button, column, container, text, text_input, Column};
use iced::{advanced, alignment, time, Color, Element, Length, Subscription, Theme};

use crate::{config, file};

static TEXT_INPUT_ID: LazyLock<text_input::Id> = LazyLock::new(text_input::Id::unique);

pub struct Player {
    main_state: MainState,
    config_data: Config,
    show_setting_modal: bool,
}

#[derive(Debug, Clone)]
pub enum PlayerMessage {
    ResumeOrPausePressed,
    NextPressed,
    PreviousPressed,

    OpenSettingModal,
    CloseSettingModal,
    MusicDirectoryInputChanged(String),
    ChooseMusicDirectory,

    MusicEnds,
    Tick(Instant),
}

impl Player {
    pub fn new() -> Self {
        let config_path = config::get_config_path();
        let config_data = config::read_config_if_exists(config_path).unwrap_or_default();

        let mut app = Self {
            main_state: MainState {
                title: "no music".into(),
                music_list: MusicList::default(),
                on_play: false,
            },
            config_data,
            show_setting_modal: false,
        };

        app.update_music_list_from_config();

        app
    }

    pub fn theme(&self) -> iced::Theme {
        iced::Theme::Dracula
    }

    pub fn update(&mut self, message: PlayerMessage) {
        match message {
            PlayerMessage::ResumeOrPausePressed => {
                self.main_state.on_play = !self.main_state.on_play;
            }
            PlayerMessage::NextPressed => {}
            PlayerMessage::PreviousPressed => {}
            PlayerMessage::OpenSettingModal => {
                self.show_setting_modal = true;
            }
            PlayerMessage::CloseSettingModal => {
                self.show_setting_modal = false;
            }
            PlayerMessage::ChooseMusicDirectory => {
                let path = dialog::open_directory_dialog();

                if let Ok(path) = path {
                    self.config_data.directory_path = path;

                    if let Err(err) = self
                        .config_data
                        .update_config_if_exists(config::get_config_path())
                    {
                        println!("Failed to update config: {:?}", err);
                    }

                    self.update_music_list_from_config();
                }
            }
            PlayerMessage::MusicDirectoryInputChanged(text) => {
                self.config_data.directory_path = text.clone().into();

                if let Err(err) = self
                    .config_data
                    .update_config_if_exists(config::get_config_path())
                {
                    println!("Failed to update config: {:?}", err);
                }

                self.update_music_list_from_config();
            }
            PlayerMessage::Tick(_) => {
                println!("tick");
            }
            PlayerMessage::MusicEnds => {}
        }
    }

    pub fn view(&self) -> Element<PlayerMessage> {
        let content = container(
            column!(
                container(
                    container(column!(
                        container(self.setting_button()).padding(0),
                        container(
                            text(self.main_state.title.as_str())
                                .size(15)
                                .shaping(advanced::text::Shaping::Advanced)
                        )
                        .padding(10)
                        .align_x(alignment::Horizontal::Center)
                        .width(Length::Fill),
                        container(self.button_view())
                            .padding(5)
                            .align_x(alignment::Horizontal::Center)
                            .width(Length::Fill),
                    ),)
                    .style(|_: &Theme| {
                        let mut style = container::Style::default();
                        style.background =
                            Some(iced::Background::Color(Color::from_rgb8(0x44, 0x47, 0x5a)));
                        style.text_color = Some(Color::BLACK);
                        style.border.width = 1.0;
                        style.border.radius = 10.0.into();

                        style
                    })
                    .padding(10),
                )
                .width(Length::Fill)
                .height(Length::Fixed(160_f32))
                .padding(10),
                container(self.items_list_view())
                    .height(Length::Fill)
                    .padding(10),
            )
            .align_x(alignment::Horizontal::Center),
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Top)
        .into();

        if self.show_setting_modal {
            let modal_view = self.setting_modal_view();

            modal::create_modal(content, modal_view, PlayerMessage::CloseSettingModal)
        } else {
            content
        }
    }

    pub fn subscription(&self) -> iced::Subscription<PlayerMessage> {
        let tick = iced::time::every(Duration::from_millis(500)).map(PlayerMessage::Tick);

        Subscription::batch(vec![tick])
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

impl Player {
    pub fn setting_button(&self) -> Element<'static, PlayerMessage> {
        let setting_button = button(
            text("setting")
                .size(12)
                .align_x(alignment::Horizontal::Right)
                .align_y(alignment::Vertical::Center),
        )
        .on_press(PlayerMessage::OpenSettingModal)
        .padding(3)
        .style(|_, _| {
            let mut style = iced::widget::button::Style::default();
            style.background = Some(iced::Background::Color(Color::from_rgba8(
                0xff, 0xff, 0xff, 0.5,
            )));
            style.border.radius = 10.0.into();
            style
        });

        setting_button.into()
    }

    fn items_list_view(&self) -> Element<'_, PlayerMessage> {
        let mut column = Column::new()
            .spacing(5)
            .align_x(iced::Alignment::Center)
            .width(Length::Fill);

        for value in self.main_state.music_list.list.iter() {
            column = column.push(
                text(value.title.as_str())
                    .size(12)
                    .shaping(advanced::text::Shaping::Advanced),
            );
        }

        widget::scrollable(container(column)).width(300).into()
    }

    fn button_view(&self) -> Element<'static, PlayerMessage> {
        let prev_button = button(
            text("<")
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center),
        )
        .on_press(PlayerMessage::PreviousPressed)
        .padding(10)
        .width(Length::Fixed(50_f32))
        .height(Length::Fixed(50_f32));

        let next_button = button(
            text(">")
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center),
        )
        .on_press(PlayerMessage::NextPressed)
        .padding(10)
        .width(Length::Fixed(50_f32))
        .height(Length::Fixed(50_f32));

        let resume_or_pause_button_text = if self.main_state.on_play { "||" } else { ">>" };

        let resume_or_pause_button = button(
            text(resume_or_pause_button_text)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center),
        )
        .on_press(PlayerMessage::ResumeOrPausePressed)
        .padding(10)
        .width(Length::Fixed(50_f32))
        .height(Length::Fixed(50_f32));

        widget::row!(prev_button, resume_or_pause_button, next_button,)
            .spacing(10)
            .into()
    }
}

impl Player {
    fn update_music_list_from_config(&mut self) {
        let music_directory_path = self.config_data.directory_path.clone();

        if let Ok(file_info_list) = file::read_file_list(&music_directory_path) {
            self.main_state.music_list.list = file_info_list
                .iter()
                .filter(|x| x.is_music_file())
                .map(|x| Music {
                    title: x.filename.clone(),
                    file_path: x.filepath.clone(),
                })
                .collect();
        }
    }
}

impl Player {
    fn setting_modal_view(&self) -> Element<'_, PlayerMessage> {
        let directory_path = self.config_data.directory_path.clone();
        let directory_path_text = directory_path.as_os_str().to_str().unwrap_or_default();

        let directory_text_input = text_input("Music Directory Path", directory_path_text)
            .id(TEXT_INPUT_ID.clone())
            .on_input(PlayerMessage::MusicDirectoryInputChanged)
            .padding(15)
            .size(13);

        let directory_error_messasge = if directory_path_text != "" {
            if !directory_path.exists() {
                "path is not exists"
            } else if !directory_path.is_dir() {
                "path is not directory"
            } else {
                ""
            }
        } else {
            ""
        };

        let directory_error_text = text(directory_error_messasge)
            .style(|_| {
                let mut style = text::Style::default();

                style.color = Some(Color::from_rgb8(u8::MAX, 0, 0));

                style
            })
            .size(9);

        let choose_directory_button =
            button(text("Choose Directory").size(12)).on_press(PlayerMessage::ChooseMusicDirectory);

        let content = container(
            column![
                text("Setting").size(24),
                column![
                    directory_text_input,
                    directory_error_text,
                    choose_directory_button,
                ]
                .spacing(2)
            ]
            .spacing(20),
        )
        .width(250)
        .padding(10)
        .style(container::rounded_box);

        content.into()
    }
}
