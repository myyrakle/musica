use std::{sync::mpsc::Receiver, thread};

use crate::{controller::BackgroundLoopEvent, state::MusicList};

pub fn background_loop(receiver: Receiver<BackgroundLoopEvent>, music_list: MusicList) {
    thread::spawn(move || {
        let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&handle).unwrap();

        loop {
            if let Ok(event) = receiver.recv() {
                match event {
                    BackgroundLoopEvent::Play => {
                        if music_list.is_not_empty() {
                            let current_music = music_list.list[0].clone();
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
                        todo!();
                    }
                    BackgroundLoopEvent::Resume => {
                        todo!();
                    }
                    BackgroundLoopEvent::Next => {
                        todo!();
                    }
                    BackgroundLoopEvent::Previous => {
                        todo!();
                    }
                    BackgroundLoopEvent::Tick => {
                        todo!();
                    }
                }
            }
        }
    });
}
