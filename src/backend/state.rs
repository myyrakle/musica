use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, AtomicUsize},
    },
    time::Duration,
};

use crate::state::MusicList;

pub enum BackgroundLoopEvent {
    StartUp,
    Pause,
    Resume,
    Next,
    Previous,
    DirectPlayMusic(usize),
    VolumeChanged(f32),
    MusicListChanged(MusicList),
    Seek(Duration),
}

#[derive(Debug, Clone)]
pub struct BackgroundState {
    pub current_index: Arc<AtomicUsize>, // Random 인덱스를 거치지 않은 순수한 1-N 인덱스
    pub current_music_index: Arc<AtomicUsize>, // Random 인덱스를 거쳐서 실제 재생 대상을 가리키는 인덱스
    pub is_random_mode: Arc<AtomicBool>,
    pub is_paused: Arc<AtomicBool>,
    pub position_millis: Arc<AtomicU64>, // 현재 곡의 재생 위치 (ms)
    pub total_duration_millis: Arc<AtomicU64>, // 현재 곡의 전체 길이 (ms, 알 수 없으면 0)
}
