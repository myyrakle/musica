mod dialog;
mod modal;

use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, LazyLock};
use std::time::{Duration, Instant};
use std::u8;

use crate::backend::background_loop;
use crate::backend::state::{BackgroundLoopEvent, BackgroundState};
use crate::state::{MainState, Music, MusicList};
use config::Config;
use iced::widget::{self, button, column, container, text, text_input, toggler, Column};
use iced::{advanced, alignment, Color, Element, Length, Subscription, Theme};

use crate::{config, file};

static TEXT_INPUT_ID: LazyLock<text_input::Id> = LazyLock::new(text_input::Id::unique);

pub struct MainApp {
    main_state: MainState,
    config_data: Config,
    show_setting_modal: bool,

    background_event_sender: Sender<BackgroundLoopEvent>,
    background_state: BackgroundState,
}

#[derive(Debug, Clone)]
pub enum ForegroundEvent {
    ResumeOrPausePressed,
    NextPressed,
    PreviousPressed,

    OpenSettingModal,
    CloseSettingModal,
    MusicDirectoryInputChanged(String),
    ChooseMusicDirectory,

    RandomToggled(bool),

    #[allow(dead_code)]
    Tick(Instant),
}

impl MainApp {
    pub fn new() -> Self {
        let config_path = config::get_config_path();
        let config_data = config::read_config_if_exists(config_path).unwrap_or_default();

        let (sender, receiver) = mpsc::channel::<BackgroundLoopEvent>();

        let background_state = BackgroundState {
            current_music_index: Default::default(),
            current_index: Default::default(),
            is_random_mode: Arc::new(config_data.is_random.into()),
        };

        let mut app = Self {
            main_state: MainState {
                title: "no music".into(),
                music_list: MusicList::default(),
                on_play: true,
            },
            config_data,
            show_setting_modal: false,
            background_state,
            background_event_sender: sender,
        };

        app.update_music_list_from_config();

        app.background_event_sender
            .send(BackgroundLoopEvent::Play)
            .unwrap();

        let music_list = app.main_state.music_list.clone();

        background_loop(receiver, app.background_state.clone(), music_list);

        app
    }
    pub fn theme(&self) -> iced::Theme {
        iced::Theme::Dracula
    }

    pub fn update(&mut self, message: ForegroundEvent) {
        match message {
            ForegroundEvent::ResumeOrPausePressed => {
                if self.main_state.on_play {
                    if let Err(error) = self
                        .background_event_sender
                        .send(BackgroundLoopEvent::Pause)
                    {
                        println!("Failed to send event: {:?}", error);
                    }

                    self.main_state.on_play = false;
                } else {
                    if let Err(error) = self
                        .background_event_sender
                        .send(BackgroundLoopEvent::Resume)
                    {
                        println!("Failed to send event: {:?}", error);
                    }

                    self.main_state.on_play = true;
                }
            }
            ForegroundEvent::NextPressed => {
                if let Err(error) = self.background_event_sender.send(BackgroundLoopEvent::Next) {
                    println!("Failed to send event: {:?}", error);
                }
            }
            ForegroundEvent::PreviousPressed => {
                if let Err(error) = self
                    .background_event_sender
                    .send(BackgroundLoopEvent::Previous)
                {
                    println!("Failed to send event: {:?}", error);
                }
            }
            ForegroundEvent::OpenSettingModal => {
                self.show_setting_modal = true;
            }
            ForegroundEvent::CloseSettingModal => {
                self.show_setting_modal = false;
            }
            ForegroundEvent::ChooseMusicDirectory => {
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
            ForegroundEvent::MusicDirectoryInputChanged(text) => {
                self.config_data.directory_path = text.clone().into();

                if let Err(err) = self
                    .config_data
                    .update_config_if_exists(config::get_config_path())
                {
                    println!("Failed to update config: {:?}", err);
                }

                self.update_music_list_from_config();
            }
            ForegroundEvent::Tick(_) => {
                let current_music_index = self
                    .background_state
                    .current_music_index
                    .load(std::sync::atomic::Ordering::Acquire);

                let current_music = &self.main_state.music_list.list[current_music_index];
                self.main_state.title = current_music.title.clone();
            }
            ForegroundEvent::RandomToggled(flag) => {
                self.config_data.is_random = flag;

                if let Err(err) = self
                    .config_data
                    .update_config_if_exists(config::get_config_path())
                {
                    println!("Failed to update config: {:?}", err);
                }

                self.background_state
                    .is_random_mode
                    .store(flag, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }

    pub fn view(&self) -> Element<ForegroundEvent> {
        let content = container(
            column!(
                container(
                    container(column!(
                        container(self.setting_button()).padding(0),
                        container(
                            text(self.main_state.title.as_str())
                                .size(15)
                                .height(Length::Fill)
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

            modal::create_modal(content, modal_view, ForegroundEvent::CloseSettingModal)
        } else {
            content
        }
    }

    pub fn subscription(&self) -> iced::Subscription<ForegroundEvent> {
        let tick = iced::time::every(Duration::from_millis(500)).map(ForegroundEvent::Tick);

        Subscription::batch(vec![tick])
    }
}

impl Default for MainApp {
    fn default() -> Self {
        Self::new()
    }
}

impl MainApp {
    pub fn setting_button(&self) -> Element<'static, ForegroundEvent> {
        let setting_button = button(
            text("setting")
                .size(12)
                .align_x(alignment::Horizontal::Right)
                .align_y(alignment::Vertical::Center),
        )
        .on_press(ForegroundEvent::OpenSettingModal)
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

    fn items_list_view(&self) -> Element<'_, ForegroundEvent> {
        let mut column = Column::new()
            .spacing(5)
            .align_x(iced::Alignment::Start)
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

    fn button_view(&self) -> Element<'static, ForegroundEvent> {
        let prev_button = button(
            text("<")
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center),
        )
        .on_press(ForegroundEvent::PreviousPressed)
        .padding(10)
        .width(Length::Fixed(50_f32))
        .height(Length::Fixed(50_f32));

        let next_button = button(
            text(">")
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center),
        )
        .on_press(ForegroundEvent::NextPressed)
        .padding(10)
        .width(Length::Fixed(50_f32))
        .height(Length::Fixed(50_f32));

        let resume_or_pause_button_text = if self.main_state.on_play { "||" } else { ">>" };

        let resume_or_pause_button = button(
            text(resume_or_pause_button_text)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center),
        )
        .on_press(ForegroundEvent::ResumeOrPausePressed)
        .padding(10)
        .width(Length::Fixed(50_f32))
        .height(Length::Fixed(50_f32));

        widget::row!(prev_button, resume_or_pause_button, next_button,)
            .spacing(10)
            .into()
    }
}

impl MainApp {
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

impl MainApp {
    fn setting_modal_view(&self) -> Element<'_, ForegroundEvent> {
        let toggler = toggler(self.config_data.is_random)
            .label("Random Mode")
            .on_toggle(ForegroundEvent::RandomToggled)
            .spacing(15);

        let directory_path = self.config_data.directory_path.clone();
        let directory_path_text = directory_path.as_os_str().to_str().unwrap_or_default();

        let directory_text_input = text_input("Music Directory Path", directory_path_text)
            .id(TEXT_INPUT_ID.clone())
            .on_input(ForegroundEvent::MusicDirectoryInputChanged)
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

        let choose_directory_button = button(text("Choose Directory").size(12))
            .on_press(ForegroundEvent::ChooseMusicDirectory);

        let content = container(
            column![
                text("Setting").size(24),
                column![toggler,].spacing(10),
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
