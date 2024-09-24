use std::{
    collections::VecDeque,
    sync::{atomic::AtomicUsize, mpsc::Sender, Arc},
};

pub enum MusicSinkReceiveEvent {
    Play,
    Pause,
    Resume,
    Next,
    Previous,
}

pub struct MusicController {
    pub event_sender: Sender<MusicSinkReceiveEvent>,
    pub current_music_index: Arc<AtomicUsize>,
}
