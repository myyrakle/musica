use std::sync::{atomic::AtomicUsize, Arc};

pub enum BackgroundLoopEvent {
    Play,
    Pause,
    Resume,
    Next,
    Previous,
    Tick,
    RandomToggled(bool),
}

#[derive(Debug, Clone)]
pub struct BackgroundState {
    pub current_music_index: Arc<AtomicUsize>,
}
