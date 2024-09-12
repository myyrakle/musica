mod config;
mod file;
mod static_assets;

use iced::widget::{self, button, column, container, text, Column};
use iced::{alignment, Color, Element, Length, Sandbox, Settings, Size, Theme};

pub fn main() -> iced::Result {
    let app_data_path = config::get_app_data_path();
    println!("{:?}", app_data_path);

    let mut setting = Settings::default();

    setting.window.resizable = false;
    setting.window.size = Size::new(300.0, 600.0);

    Player::run(setting)
}

#[derive(Debug, Clone, Default)]
pub struct Music {
    title: String,
    pub file_path: String,
}

#[derive(Debug, Clone)]
pub struct MusicList {
    list: Vec<Music>,
}

impl Default for MusicList {
    fn default() -> Self {
        Self {
            list: vec![
                Music {
                    title: "test1".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test2".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test3".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test4".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test5".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test6".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test7".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test8".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test9".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test10".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test11".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test12".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test13".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test14".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test10".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test11".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test12".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test13".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test14".into(),
                    file_path: "test".into(),
                },
            ],
        }
    }
}

pub struct Player {
    title: String,
    value: i32,
    music_list: MusicList,
    on_play: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum PlayerMessage {
    ResumeOrPausePressed,
    NextPressed,
    PreviousPressed,
}

impl Sandbox for Player {
    type Message = PlayerMessage;

    fn new() -> Self {
        Self {
            title: "test name".into(),
            value: 0,
            music_list: MusicList::default(),
            on_play: false,
        }
    }

    fn title(&self) -> String {
        String::from("musica")
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dracula
    }

    fn update(&mut self, message: PlayerMessage) {
        match message {
            PlayerMessage::ResumeOrPausePressed => {
                self.on_play = !self.on_play;
            }
            PlayerMessage::NextPressed => {
                self.value += 1;
            }
            PlayerMessage::PreviousPressed => {
                self.value -= 1;
            }
        }
    }

    fn view(&self) -> Element<PlayerMessage> {
        container(
            column!(
                container(
                    container(column!(
                        container(text(self.title.as_str()).size(15))
                            .padding(15)
                            .align_x(alignment::Horizontal::Center)
                            .width(Length::Fill),
                        container(self.button_view())
                            .padding(5)
                            .align_x(alignment::Horizontal::Center)
                            .width(Length::Fill),
                    ),)
                    .style(|_: &Theme| {
                        let mut style = container::Appearance::default();
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
                .height(Length::Fixed(150_f32))
                .padding(10),
                container(self.items_list_view())
                    .height(Length::Fill)
                    .padding(10),
            )
            .align_items(iced::Alignment::Center),
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Top)
        .into()
    }
}

impl Player {
    fn items_list_view(&self) -> Element<'static, PlayerMessage> {
        let mut column = Column::new()
            .spacing(10)
            .align_items(iced::Alignment::Center)
            .width(Length::Fill);

        for value in self.music_list.list.iter() {
            column = column.push(text(value.title.as_str()));
        }

        widget::scrollable(container(column)).width(300).into()
    }

    fn button_view(&self) -> Element<'static, PlayerMessage> {
        let prev_button = button(
            text("<")
                .horizontal_alignment(alignment::Horizontal::Center)
                .vertical_alignment(alignment::Vertical::Center),
        )
        .on_press(PlayerMessage::PreviousPressed)
        .padding(10)
        .width(Length::Fixed(50_f32))
        .height(Length::Fixed(50_f32));

        let next_button = button(
            text(">")
                .horizontal_alignment(alignment::Horizontal::Center)
                .vertical_alignment(alignment::Vertical::Center),
        )
        .on_press(PlayerMessage::NextPressed)
        .padding(10)
        .width(Length::Fixed(50_f32))
        .height(Length::Fixed(50_f32));

        let resume_or_pause_button_text = if self.on_play { "||" } else { ">>" };

        let resume_or_pause_button = button(
            text(resume_or_pause_button_text)
                .horizontal_alignment(alignment::Horizontal::Center)
                .vertical_alignment(alignment::Vertical::Center),
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
