use iced::widget::{self, button, column, container, text, Column};
use iced::{alignment, Alignment, Element, Length, Sandbox, Settings, Size};

pub fn main() -> iced::Result {
    let mut setting = Settings::default();

    setting.window.size = Size::new(300.0, 600.0);

    Player::run(setting)
}

#[derive(Debug, Clone, Default)]
pub struct Music {
    title: String,
    file_path: String,
}

#[derive(Debug, Clone)]
pub struct MusicList {
    list: Vec<Music>,
    size: i32,
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
            size: 20,
        }
    }
}

pub struct Player {
    title: String,
    value: i32,
    music_list: MusicList,
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
                self.value += 1;
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
                container(column!(
                    container(text(self.title.as_str()).size(20))
                        .padding(20)
                        .align_x(alignment::Horizontal::Center)
                        .width(Length::Fill),
                    container(self.button_view())
                        .align_x(alignment::Horizontal::Center)
                        .width(Length::Fill),
                ))
                .width(Length::Fill)
                .height(Length::Fixed(110_f32)),
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
        let prev_button = button(text("<").horizontal_alignment(alignment::Horizontal::Center))
            .on_press(PlayerMessage::PreviousPressed)
            .padding(10)
            .width(Length::Fixed(50_f32))
            .height(Length::Fixed(50_f32));

        let next_button = button(text(">").horizontal_alignment(alignment::Horizontal::Center))
            .on_press(PlayerMessage::NextPressed)
            .padding(10)
            .width(Length::Fixed(50_f32))
            .height(Length::Fixed(50_f32));

        let resume_or_pause_button =
            button(text("||").horizontal_alignment(alignment::Horizontal::Center))
                .on_press(PlayerMessage::ResumeOrPausePressed)
                .padding(10)
                .width(Length::Fixed(50_f32))
                .height(Length::Fixed(50_f32));

        widget::row!(prev_button, resume_or_pause_button, next_button,)
            .spacing(10)
            .into()
    }
}
