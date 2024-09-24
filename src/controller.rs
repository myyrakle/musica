use std::sync::{atomic::AtomicUsize, Arc};

pub enum BackgroundLoopEvent {
    Play,
    Pause,
    Resume,
    Next,
    Previous,
    Tick,
}

pub struct MusicController {
    pub current_music_index: Arc<AtomicUsize>,
}
