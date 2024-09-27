use std::sync::{
    atomic::{AtomicBool, AtomicUsize},
    Arc,
};

pub enum BackgroundLoopEvent {
    StartUp,
    Play,
    Pause,
    Resume,
    Next,
    Previous,
}

#[derive(Debug, Clone)]
pub struct BackgroundState {
    pub current_index: Arc<AtomicUsize>, // Random 인덱스를 거치지 않은 순수한 1-N 인덱스
    pub current_music_index: Arc<AtomicUsize>, // Random 인덱스를 거쳐서 실제 재생 대상을 가리키는 인덱스
    pub is_random_mode: Arc<AtomicBool>,
}
