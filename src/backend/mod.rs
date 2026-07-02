pub mod state;

use std::{fs::File, sync::mpsc::Receiver, thread, time::Duration};

use rodio::Decoder;
use state::{BackgroundLoopEvent, BackgroundState};

use crate::state::{Music, MusicList};

fn get_current_music_source(
    background_state: &mut BackgroundState,
    random_indices: &[usize],
    music_list: &MusicList,
) -> anyhow::Result<Decoder<std::io::BufReader<File>>> {
    let mut index = background_state
        .current_index
        .load(std::sync::atomic::Ordering::Acquire);

    if background_state
        .is_random_mode
        .load(std::sync::atomic::Ordering::Acquire)
    {
        index = random_indices[index];
    }

    background_state
        .current_music_index
        .store(index, std::sync::atomic::Ordering::Release);

    let current_music = music_list.list[index].clone();
    let source = get_source_from_music(&current_music)?;

    Ok(source)
}

fn get_source_from_music(music: &Music) -> anyhow::Result<Decoder<std::io::BufReader<File>>> {
    let file = std::fs::File::open(&music.file_path)?;
    println!("file: {:?}", music.file_path);

    let source = rodio::Decoder::try_from(file)?;

    Ok(source)
}

pub fn background_loop(
    receiver: Receiver<BackgroundLoopEvent>,
    mut background_state: BackgroundState,
    music_list: MusicList,
) {
    thread::spawn(move || {
        // StartUp 이벤트가 들어올때까지 대기
        loop {
            if let Ok(BackgroundLoopEvent::StartUp) = receiver.recv() {
                break;
            }
        }

        let _stream = rodio::DeviceSinkBuilder::open_default_sink()
            .expect("Failed to open default audio output device");
        let sink = rodio::Player::connect_new(_stream.mixer());

        // shuffled index list
        let mut random_indices = (0..music_list.list.len()).collect::<Vec<_>>();

        {
            use rand::seq::SliceRandom;
            random_indices.shuffle(&mut rand::rng());
        }

        // 시작 시 첫 곡 자동 재생
        if music_list.is_not_empty()
            && let Ok(source) =
                get_current_music_source(&mut background_state, &random_indices, &music_list)
        {
            sink.play();
            sink.append(source);
        }

        loop {
            if let Ok(event) = receiver.recv_timeout(Duration::from_millis(100)) {
                match event {
                    BackgroundLoopEvent::Pause => {
                        if !sink.is_paused() {
                            sink.pause();
                        }
                        background_state
                            .is_paused
                            .store(true, std::sync::atomic::Ordering::Relaxed);
                    }
                    BackgroundLoopEvent::Resume => {
                        if sink.is_paused() {
                            sink.play();
                        }
                        background_state
                            .is_paused
                            .store(false, std::sync::atomic::Ordering::Relaxed);
                    }
                    BackgroundLoopEvent::Next => {
                        let mut index = background_state
                            .current_index
                            .load(std::sync::atomic::Ordering::Acquire);

                        index += 1;

                        if index >= music_list.list.len() {
                            index = 0;
                        }

                        background_state
                            .current_index
                            .store(index, std::sync::atomic::Ordering::Relaxed);

                        if music_list.is_not_empty()
                            && let Ok(source) = get_current_music_source(
                                &mut background_state,
                                &random_indices,
                                &music_list,
                            )
                        {
                            sink.clear();
                            sink.play();
                            sink.append(source);
                            background_state
                                .is_paused
                                .store(false, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                    BackgroundLoopEvent::Previous => {
                        let mut index = background_state
                            .current_index
                            .load(std::sync::atomic::Ordering::Acquire);

                        if index == 0 {
                            index = music_list.list.len() - 1;
                        } else {
                            index -= 1;
                        }

                        background_state
                            .current_index
                            .store(index, std::sync::atomic::Ordering::Relaxed);

                        if music_list.is_not_empty()
                            && let Ok(source) = get_current_music_source(
                                &mut background_state,
                                &random_indices,
                                &music_list,
                            )
                        {
                            sink.clear();
                            sink.play();
                            sink.append(source);
                            background_state
                                .is_paused
                                .store(false, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                    BackgroundLoopEvent::DirectPlayMusic(index) => {
                        if index >= music_list.list.len() {
                            // 범위를 벗어난 인덱스는 무시 (리스트 변경 등으로 stale된 클로저 방지)
                            continue;
                        }

                        // current_index를 직접 재생한 곡의 위치로 맞춰
                        // 이후 Next/Previous 및 자동 다음 곡 연결이 올바르게 이어지도록 함
                        if background_state
                            .is_random_mode
                            .load(std::sync::atomic::Ordering::Acquire)
                        {
                            // 랜덤 모드: random_indices에서 index가 위치한 슬롯을 역추적
                            if let Some(slot) = random_indices.iter().position(|&i| i == index) {
                                background_state
                                    .current_index
                                    .store(slot, std::sync::atomic::Ordering::Relaxed);
                            }
                        } else {
                            background_state
                                .current_index
                                .store(index, std::sync::atomic::Ordering::Relaxed);
                        }

                        background_state
                            .current_music_index
                            .store(index, std::sync::atomic::Ordering::Relaxed);

                        let music = music_list.list[index].clone();

                        if let Ok(source) = get_source_from_music(&music) {
                            sink.clear();
                            sink.play();
                            sink.append(source);
                            background_state
                                .is_paused
                                .store(false, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                    _ => {}
                }
            }

            // Background Tick
            {
                // 일시정지 중에는 자동 다음 곡 재생을 하지 않는다.
                // (일시정지 상태에서 sink가 비었다고 해서 임의로 다음 곡을 재생하면
                //  사용자가 일시정지했음에도 재생이 시작되는 버그가 발생함)
                if sink.empty() && !sink.is_paused() {
                    let mut index = background_state
                        .current_index
                        .load(std::sync::atomic::Ordering::Acquire);

                    index += 1;

                    if index >= music_list.list.len() {
                        index = 0;
                    }

                    background_state
                        .current_index
                        .store(index, std::sync::atomic::Ordering::Relaxed);

                    if music_list.is_not_empty()
                        && let Ok(source) = get_current_music_source(
                            &mut background_state,
                            &random_indices,
                            &music_list,
                        )
                    {
                        sink.clear();
                        sink.play();
                        sink.append(source);
                        background_state
                            .is_paused
                            .store(false, std::sync::atomic::Ordering::Relaxed);
                    }
                }
            }
        }
    });
}
