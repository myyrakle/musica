use std::sync::{atomic::AtomicUsize, mpsc::Sender, Arc};

pub enum BackgroundLoopEvent {
    Play,
    Pause,
    Resume,
    Next,
    Previous,
}

pub struct MusicController {
    pub event_sender: Sender<BackgroundLoopEvent>,
    pub current_music_index: Arc<AtomicUsize>,
}
