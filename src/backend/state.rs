use std::sync::{
    atomic::{AtomicBool, AtomicUsize},
    Arc,
};

pub enum BackgroundLoopEvent {
    Play,
    Pause,
    Resume,
    Next,
    Previous,
    Tick,
}

#[derive(Debug, Clone)]
pub struct BackgroundState {
    pub current_music_index: Arc<AtomicUsize>,
    pub is_random_mode: Arc<AtomicBool>,
}
