pub mod state;

use std::{fs::File, io::BufReader, sync::mpsc::Receiver, thread, time::Duration};

use rodio::Decoder;
use state::{BackgroundLoopEvent, BackgroundState};

use crate::state::MusicList;

fn get_current_music_source(
    background_state: &mut BackgroundState,
    random_indices: &Vec<usize>,
    music_list: &MusicList,
) -> anyhow::Result<Decoder<BufReader<File>>> {
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
    let file = std::fs::File::open(&current_music.file_path)?;
    let buffer = std::io::BufReader::new(file);
    println!("file: {:?}", current_music.file_path);

    let source = rodio::Decoder::new(buffer)?;

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

        let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&handle).unwrap();

        // shuffled index list
        let mut random_indices = (0..music_list.list.len()).collect::<Vec<_>>();

        {
            use rand::seq::SliceRandom;
            random_indices.shuffle(&mut rand::thread_rng());
        }

        loop {
            if let Ok(event) = receiver.recv_timeout(Duration::from_millis(100)) {
                match event {
                    BackgroundLoopEvent::Play => {
                        if music_list.is_not_empty() {
                            if let Ok(source) = get_current_music_source(
                                &mut background_state,
                                &random_indices,
                                &music_list,
                            ) {
                                sink.play();
                                sink.append(source);
                            }
                        }
                    }
                    BackgroundLoopEvent::Pause => {
                        if !sink.is_paused() {
                            sink.pause();
                        }
                    }
                    BackgroundLoopEvent::Resume => {
                        if sink.is_paused() {
                            sink.play();
                        }
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

                        if music_list.is_not_empty() {
                            if let Ok(source) = get_current_music_source(
                                &mut background_state,
                                &random_indices,
                                &music_list,
                            ) {
                                sink.clear();
                                sink.play();
                                sink.append(source);
                            }
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

                        if music_list.is_not_empty() {
                            if let Ok(source) = get_current_music_source(
                                &mut background_state,
                                &random_indices,
                                &music_list,
                            ) {
                                sink.clear();
                                sink.play();
                                sink.append(source);
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Background Tick
            {
                if sink.empty() {
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

                    if music_list.is_not_empty() {
                        if let Ok(source) = get_current_music_source(
                            &mut background_state,
                            &random_indices,
                            &music_list,
                        ) {
                            sink.clear();
                            sink.play();
                            sink.append(source);
                        }
                    }
                }
            }
        }
    });
}
