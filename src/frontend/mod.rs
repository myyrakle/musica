mod dialog;
mod modal;

use std::sync::mpsc::Sender;
use std::sync::{Arc, LazyLock, mpsc};
use std::time::{Duration, Instant};

use crate::backend::background_loop;
use crate::backend::state::{BackgroundLoopEvent, BackgroundState};
use crate::state::{MainState, Music, MusicList};
use config::Config;
use iced::widget::{self, Column, button, column, container, text, text_input, toggler};
use iced::{Color, Element, Length, Subscription, Theme, advanced, alignment};

use crate::{config, file};

static TEXT_INPUT_ID: LazyLock<widget::Id> = LazyLock::new(widget::Id::unique);

pub struct MainApp {
    main_state: MainState,
    config_data: Config,
    show_setting_modal: bool,

    background_event_sender: Sender<BackgroundLoopEvent>,
    background_state: BackgroundState,

    position_secs: f32,        // 현재 재생 위치 (초)
    total_secs: f32,           // 현재 곡 전체 길이 (초, 알 수 없으면 0)
    seek_preview: Option<f32>, // 재생 막대를 드래그하는 중일 때의 위치 (초)
}

#[derive(Debug, Clone)]
pub enum ForegroundEvent {
    ResumeOrPausePressed,
    NextPressed,
    PreviousPressed,

    DirectPlayMusic(usize),

    OpenSettingModal,
    CloseSettingModal,
    MusicDirectoryInputChanged(String),
    ChooseMusicDirectory,

    RandomToggled(bool),
    VolumeChanged(f32),

    SeekChanged(f32),
    SeekReleased,

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
            is_paused: Arc::new(false.into()),
            position_millis: Default::default(),
            total_duration_millis: Default::default(),
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
            position_secs: 0.0,
            total_secs: 0.0,
            seek_preview: None,
        };

        app.update_music_list_from_config();

        app.config_data.volume = Config::normalize_volume(app.config_data.volume);

        let music_list = app.main_state.music_list.clone();

        background_loop(
            receiver,
            app.background_state.clone(),
            music_list,
            app.config_data.volume,
        );

        // 백그라운드 스레드가 OutputStream/Sink를 생성한 뒤 첫 곡을 자동 재생하도록 트리거
        app.background_event_sender
            .send(BackgroundLoopEvent::StartUp)
            .unwrap();

        app
    }
    /// 창 제목에도 현재 곡 제목을 표시해 작업 표시줄 등에서 전체 제목을 확인할 수 있게 한다.
    pub fn window_title(&self) -> String {
        if self.main_state.music_list.is_not_empty() {
            format!("{} - musica", self.main_state.title)
        } else {
            "musica".into()
        }
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

                    self.refresh_music_list();
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

                self.refresh_music_list();
            }
            ForegroundEvent::Tick(_) => {
                // 백엔드의 실제 일시정지 상태를 프론트엔드 on_play와 동기화
                let is_paused = self
                    .background_state
                    .is_paused
                    .load(std::sync::atomic::Ordering::Acquire);
                self.main_state.on_play = !is_paused;

                self.total_secs =
                    self.background_state
                        .total_duration_millis
                        .load(std::sync::atomic::Ordering::Acquire) as f32
                        / 1000.0;
                let position_secs =
                    self.background_state
                        .position_millis
                        .load(std::sync::atomic::Ordering::Acquire) as f32
                        / 1000.0;
                self.position_secs = if self.total_secs > 0.0 {
                    position_secs.min(self.total_secs)
                } else {
                    position_secs
                };

                let current_music_index = self
                    .background_state
                    .current_music_index
                    .load(std::sync::atomic::Ordering::Acquire);

                if self.main_state.music_list.list.is_empty() {
                    self.main_state.title = "no music".into();
                    return;
                }

                // 목록이 바뀐 직후에는 백그라운드가 아직 이전 목록 기준의 인덱스를 가지고 있을 수 있다.
                if let Some(current_music) =
                    self.main_state.music_list.list.get(current_music_index)
                {
                    self.main_state.title = current_music.title.clone();
                }
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
            ForegroundEvent::VolumeChanged(volume) => {
                let volume = Config::normalize_volume(volume);
                self.config_data.volume = volume;

                if let Err(err) = self
                    .config_data
                    .update_config_if_exists(config::get_config_path())
                {
                    println!("Failed to update config: {:?}", err);
                }

                if let Err(error) = self
                    .background_event_sender
                    .send(BackgroundLoopEvent::VolumeChanged(volume))
                {
                    println!("Failed to send event: {:?}", error);
                }
            }
            ForegroundEvent::SeekChanged(position) => {
                // 전체 길이를 알 수 없는 곡은 탐색하지 않는다
                if self.total_secs > 0.0 {
                    self.seek_preview = Some(position.clamp(0.0, self.total_secs));
                }
            }
            ForegroundEvent::SeekReleased => {
                if let Some(position) = self.seek_preview.take() {
                    self.position_secs = position;

                    if let Err(error) = self
                        .background_event_sender
                        .send(BackgroundLoopEvent::Seek(Duration::from_secs_f32(position)))
                    {
                        println!("Failed to send event: {:?}", error);
                    }
                }
            }
            ForegroundEvent::DirectPlayMusic(index) => {
                if let Err(error) = self
                    .background_event_sender
                    .send(BackgroundLoopEvent::DirectPlayMusic(index))
                {
                    println!("Failed to send event: {:?}", error);
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, ForegroundEvent> {
        let content = container(
            column!(
                container(
                    container(column!(
                        container(self.setting_button()).padding(0),
                        // 제목은 최대 두 줄(15px 글자 기준)까지만 보여주고 넘치는 부분은 잘라낸다.
                        // 전체 제목은 툴팁으로 확인할 수 있다. 한 줄짜리 제목은 세로 가운데 정렬한다.
                        container(
                            container(self.title_view())
                                .max_height(40)
                                .clip(true)
                        )
                        .padding([0, 10])
                        .height(Length::Fixed(40_f32))
                        .align_x(alignment::Horizontal::Center)
                        .align_y(alignment::Vertical::Center)
                        .width(Length::Fill),
                        container(self.seek_view())
                            .padding([0, 5])
                            .width(Length::Fill),
                        container(self.button_view())
                            .align_x(alignment::Horizontal::Center)
                            .width(Length::Fill),
                        container(self.volume_view())
                            .padding([0, 5])
                            .width(Length::Fill),
                    )
                    .spacing(10),)
                    .style(|_: &Theme| {
                        container::Style {
                            text_color: Some(Color::BLACK),
                            border: iced::Border {
                                width: 1.0,
                                radius: 10.0.into(),
                                ..Default::default()
                            },
                            background: Some(iced::Background::Color(Color::from_rgb8(
                                0x44, 0x47, 0x5a,
                            ))),
                            ..Default::default()
                        }
                    })
                    .padding(10),
                )
                .width(Length::Fill)
                .height(Length::Shrink)
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
        let tick = iced::time::every(Duration::from_secs(1)).map(ForegroundEvent::Tick);

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
        .style(|_, _| iced::widget::button::Style {
            background: Some(iced::Background::Color(Color::from_rgba8(
                0xff, 0xff, 0xff, 0.5,
            ))),
            border: iced::Border {
                radius: 10.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        setting_button.into()
    }

    /// 현재 재생 중인 곡 제목.
    /// 상단 영역 높이가 고정되어 있어 긴 제목은 잘리므로, 마우스를 올리면 전체 제목을 툴팁으로 보여준다.
    fn title_view(&self) -> Element<'_, ForegroundEvent> {
        let title = self.main_state.title.as_str();

        let title_text = text(title)
            .size(15)
            .align_x(alignment::Horizontal::Center)
            .shaping(advanced::text::Shaping::Advanced);

        let full_title = container(
            text(title)
                .size(13)
                .shaping(advanced::text::Shaping::Advanced),
        )
        .max_width(260)
        .padding(8)
        .style(|_: &Theme| container::Style {
            text_color: Some(Color::WHITE),
            background: Some(iced::Background::Color(Color::from_rgba8(
                0x0, 0x0, 0x0, 0.85,
            ))),
            border: iced::Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        widget::tooltip(title_text, full_title, widget::tooltip::Position::Top)
            .gap(4)
            .delay(Duration::from_millis(300))
            .into()
    }

    fn items_list_view(&self) -> Element<'_, ForegroundEvent> {
        let mut column = Column::new()
            .spacing(5)
            .align_x(iced::Alignment::Start)
            .width(Length::Fill);

        for (i, value) in self.main_state.music_list.list.iter().enumerate() {
            let text_widget = text(value.title.as_str())
                .size(12)
                .shaping(advanced::text::Shaping::Advanced)
                .color(Color::WHITE);

            let button_widget = button(text_widget)
                .on_press_with(move || ForegroundEvent::DirectPlayMusic(i))
                .padding(5)
                .style(|_, _| iced::widget::button::Style {
                    background: Some(iced::Background::Color(Color::from_rgba8(
                        0x0, 0x0, 0x0, 0.5,
                    ))),
                    border: iced::Border {
                        radius: 0.0.into(),
                        width: 0.1,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .width(Length::Fill);

            column = column.push(button_widget);
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

    fn seek_view(&self) -> Element<'_, ForegroundEvent> {
        // 드래그 중에는 드래그 위치를, 아니면 실제 재생 위치를 표시
        let position = self.seek_preview.unwrap_or(self.position_secs);
        let max = if self.total_secs > 0.0 {
            self.total_secs
        } else {
            1.0
        };

        let time_text = if self.total_secs > 0.0 {
            format!(
                "{} / {}",
                format_time(position),
                format_time(self.total_secs)
            )
        } else {
            format!("{} / --:--", format_time(position))
        };

        widget::row![
            widget::slider(0.0..=max, position.min(max), ForegroundEvent::SeekChanged)
                .on_release(ForegroundEvent::SeekReleased)
                .step(0.1_f32),
            text(time_text).size(12),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .width(Length::Fill)
        .into()
    }

    fn volume_view(&self) -> Element<'_, ForegroundEvent> {
        let volume_percent = (self.config_data.volume * 100.0).round() as u8;

        widget::row![
            text("Volume").size(12),
            widget::slider(
                0.0..=1.0,
                self.config_data.volume,
                ForegroundEvent::VolumeChanged,
            )
            .step(0.01_f32),
            text(format!("{volume_percent}%"))
                .size(12)
                .width(Length::Fixed(36.0)),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .width(Length::Fill)
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

    // 곡 목록이 바뀌면 백그라운드 루프에도 전달해 같은 목록을 기준으로 재생하도록 한다.
    fn refresh_music_list(&mut self) {
        let previous_music_list = self.main_state.music_list.clone();

        self.update_music_list_from_config();

        if self.main_state.music_list != previous_music_list
            && let Err(error) =
                self.background_event_sender
                    .send(BackgroundLoopEvent::MusicListChanged(
                        self.main_state.music_list.clone(),
                    ))
        {
            println!("Failed to send event: {:?}", error);
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

        let directory_error_messasge = if !directory_path_text.is_empty() {
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
            .style(|_| text::Style {
                color: Some(Color::from_rgb8(u8::MAX, 0, 0)),
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

/// 초 단위 시간을 mm:ss (1시간 이상이면 h:mm:ss) 형식으로 변환
fn format_time(secs: f32) -> String {
    let total = secs.max(0.0) as u64;
    let (hours, minutes, seconds) = (total / 3600, (total % 3600) / 60, total % 60);

    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::format_time;

    #[test]
    fn format_time_works() {
        assert_eq!(format_time(0.0), "00:00");
        assert_eq!(format_time(65.9), "01:05");
        assert_eq!(format_time(3725.0), "1:02:05");
        assert_eq!(format_time(-3.0), "00:00");
    }
}
