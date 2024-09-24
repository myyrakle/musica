pub mod state;

use std::{sync::mpsc::Receiver, thread};

use state::{BackgroundLoopEvent, BackgroundState};

use crate::state::MusicList;

pub fn background_loop(
    receiver: Receiver<BackgroundLoopEvent>,
    background_state: BackgroundState,
    music_list: MusicList,
) {
    thread::spawn(move || {
        let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&handle).unwrap();

        loop {
            if let Ok(event) = receiver.recv() {
                match event {
                    BackgroundLoopEvent::Play => {
                        if music_list.is_not_empty() {
                            let index = background_state
                                .current_music_index
                                .load(std::sync::atomic::Ordering::Relaxed);

                            let current_music = music_list.list[index].clone();
                            let file = std::fs::File::open(&current_music.file_path).unwrap();
                            let buffer = std::io::BufReader::new(file);
                            println!("file: {:?}", current_music.file_path);

                            let source = rodio::Decoder::new(buffer).unwrap();

                            {
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
                        todo!();
                    }
                    BackgroundLoopEvent::Previous => {
                        todo!();
                    }
                    BackgroundLoopEvent::Tick => {}
                }
            }
        }
    });
}
