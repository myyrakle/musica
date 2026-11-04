pub mod state;

use std::{
    fs::File,
    sync::mpsc::Receiver,
    thread,
    time::{Duration, Instant},
};

use rodio::{Decoder, MixerDeviceSink, Player, Source};
use state::{BackgroundLoopEvent, BackgroundState};

use crate::state::{Music, MusicList};

// 재생 중인데 이 시간 동안 재생 위치가 그대로면 오디오 출력이 죽은 것으로 판단
const STALL_TIMEOUT: Duration = Duration::from_secs(3);
const AUDIO_OUTPUT_RETRY_INTERVAL: Duration = Duration::from_secs(1);

// 오디오 스트림이 죽어도 cpal은 복구하지 않고, 경우에 따라 오류 콜백조차 호출하지 않는다.
// (예: PipeWire 스트림이 끊긴 경우) 그래서 재생 위치의 진행 여부로 직접 감지한다.
struct StallDetector {
    last_position: Duration,
    last_progress_at: Instant,
}

impl StallDetector {
    fn new(now: Instant) -> Self {
        Self {
            last_position: Duration::ZERO,
            last_progress_at: now,
        }
    }

    fn is_stalled(&mut self, position: Duration, is_playing: bool, now: Instant) -> bool {
        if !is_playing || position != self.last_position {
            self.last_position = position;
            self.last_progress_at = now;
            return false;
        }

        now.duration_since(self.last_progress_at) >= STALL_TIMEOUT
    }
}

fn open_audio_output() -> anyhow::Result<MixerDeviceSink> {
    let mut stream = rodio::DeviceSinkBuilder::open_default_sink()?;
    stream.log_on_drop(false);

    Ok(stream)
}

// Player::clear()는 오디오 스레드가 곡을 넘겨줄 때까지 타임아웃 없이 대기하므로,
// 스트림이 죽어 있으면 백그라운드 루프 전체가 멈춘다. 대신 Player를 새로 만들어 교체한다.
fn replace_player(
    sink: &mut Player,
    start_offset: &mut Duration,
    stream: &MixerDeviceSink,
    source: Decoder<std::io::BufReader<File>>,
) {
    let player = Player::connect_new(stream.mixer());
    player.set_volume(sink.volume());
    player.append(source);

    *sink = player;
    *start_offset = Duration::ZERO;
}

fn get_current_music_source(
    background_state: &mut BackgroundState,
    random_indices: &[usize],
    music_list: &MusicList,
) -> anyhow::Result<Decoder<std::io::BufReader<File>>> {
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
    let source = get_source_from_music(&current_music)?;

    Ok(source)
}

fn get_source_from_music(music: &Music) -> anyhow::Result<Decoder<std::io::BufReader<File>>> {
    let file = std::fs::File::open(&music.file_path)?;
    println!("file: {:?}", music.file_path);

    let source = rodio::Decoder::try_from(file)?;

    Ok(source)
}

pub fn background_loop(
    receiver: Receiver<BackgroundLoopEvent>,
    mut background_state: BackgroundState,
    music_list: MusicList,
    initial_volume: f32,
) {
    thread::spawn(move || {
        // StartUp 이벤트가 들어올때까지 대기
        loop {
            if let Ok(BackgroundLoopEvent::StartUp) = receiver.recv() {
                break;
            }
        }

        let mut stream = open_audio_output().expect("Failed to open default audio output device");
        let mut sink = Player::connect_new(stream.mixer());
        sink.set_volume(initial_volume);

        // Player::get_pos()는 append 이후 진행된 시간이므로, 곡 중간부터 이어서 재생한 경우
        // 곡 안에서의 실제 위치는 start_offset + get_pos()가 된다.
        let mut start_offset = Duration::ZERO;

        let mut stall_detector = StallDetector::new(Instant::now());

        // shuffled index list
        let mut random_indices = (0..music_list.list.len()).collect::<Vec<_>>();

        {
            use rand::seq::SliceRandom;
            random_indices.shuffle(&mut rand::rng());
        }

        // 시작 시 첫 곡 자동 재생
        if music_list.is_not_empty()
            && let Ok(source) =
                get_current_music_source(&mut background_state, &random_indices, &music_list)
        {
            sink.play();
            sink.append(source);
        }

        loop {
            if let Ok(event) = receiver.recv_timeout(Duration::from_millis(100)) {
                match event {
                    BackgroundLoopEvent::Pause => {
                        if !sink.is_paused() {
                            sink.pause();
                        }
                        background_state
                            .is_paused
                            .store(true, std::sync::atomic::Ordering::Relaxed);
                    }
                    BackgroundLoopEvent::Resume => {
                        if sink.is_paused() {
                            sink.play();
                        }
                        background_state
                            .is_paused
                            .store(false, std::sync::atomic::Ordering::Relaxed);
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

                        if music_list.is_not_empty()
                            && let Ok(source) = get_current_music_source(
                                &mut background_state,
                                &random_indices,
                                &music_list,
                            )
                        {
                            replace_player(&mut sink, &mut start_offset, &stream, source);
                            background_state
                                .is_paused
                                .store(false, std::sync::atomic::Ordering::Relaxed);
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

                        if music_list.is_not_empty()
                            && let Ok(source) = get_current_music_source(
                                &mut background_state,
                                &random_indices,
                                &music_list,
                            )
                        {
                            replace_player(&mut sink, &mut start_offset, &stream, source);
                            background_state
                                .is_paused
                                .store(false, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                    BackgroundLoopEvent::DirectPlayMusic(index) => {
                        if index >= music_list.list.len() {
                            // 범위를 벗어난 인덱스는 무시 (리스트 변경 등으로 stale된 클로저 방지)
                            continue;
                        }

                        // current_index를 직접 재생한 곡의 위치로 맞춰
                        // 이후 Next/Previous 및 자동 다음 곡 연결이 올바르게 이어지도록 함
                        if background_state
                            .is_random_mode
                            .load(std::sync::atomic::Ordering::Acquire)
                        {
                            // 랜덤 모드: random_indices에서 index가 위치한 슬롯을 역추적
                            if let Some(slot) = random_indices.iter().position(|&i| i == index) {
                                background_state
                                    .current_index
                                    .store(slot, std::sync::atomic::Ordering::Relaxed);
                            }
                        } else {
                            background_state
                                .current_index
                                .store(index, std::sync::atomic::Ordering::Relaxed);
                        }

                        background_state
                            .current_music_index
                            .store(index, std::sync::atomic::Ordering::Relaxed);

                        let music = music_list.list[index].clone();

                        if let Ok(source) = get_source_from_music(&music) {
                            replace_player(&mut sink, &mut start_offset, &stream, source);
                            background_state
                                .is_paused
                                .store(false, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                    BackgroundLoopEvent::VolumeChanged(volume) => {
                        sink.set_volume(volume);
                    }
                    _ => {}
                }
            }

            // Background Tick
            {
                let is_playing = !sink.empty() && !sink.is_paused();

                if stall_detector.is_stalled(sink.get_pos(), is_playing, Instant::now()) {
                    println!("audio output stalled, reopening");

                    let position = start_offset + sink.get_pos();

                    drop(stream);
                    stream = loop {
                        match open_audio_output() {
                            Ok(stream) => break stream,
                            Err(error) => {
                                println!("Failed to reopen audio output: {:?}", error);
                                thread::sleep(AUDIO_OUTPUT_RETRY_INTERVAL);
                            }
                        }
                    };

                    let index = background_state
                        .current_music_index
                        .load(std::sync::atomic::Ordering::Acquire);

                    // 파일을 열 수 없으면 sink가 비므로 아래에서 다음 곡으로 넘어간다.
                    let player = Player::connect_new(stream.mixer());
                    player.set_volume(sink.volume());
                    sink = player;

                    start_offset = Duration::ZERO;

                    if let Some(music) = music_list.list.get(index)
                        && let Ok(mut source) = get_source_from_music(music)
                    {
                        // 끊긴 위치부터 이어서 재생. 탐색에 실패하면 곡의 처음부터 재생한다.
                        if source.try_seek(position).is_ok() {
                            start_offset = position;
                        }
                        sink.append(source);
                    }

                    stall_detector = StallDetector::new(Instant::now());
                }

                // 일시정지 중에는 자동 다음 곡 재생을 하지 않는다.
                // (일시정지 상태에서 sink가 비었다고 해서 임의로 다음 곡을 재생하면
                //  사용자가 일시정지했음에도 재생이 시작되는 버그가 발생함)
                if sink.empty() && !sink.is_paused() {
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

                    if music_list.is_not_empty()
                        && let Ok(source) = get_current_music_source(
                            &mut background_state,
                            &random_indices,
                            &music_list,
                        )
                    {
                        replace_player(&mut sink, &mut start_offset, &stream, source);
                        background_state
                            .is_paused
                            .store(false, std::sync::atomic::Ordering::Relaxed);
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const POSITION: Duration = Duration::from_secs(10);

    #[test]
    fn detects_stall_when_position_stops_while_playing() {
        let start = Instant::now();
        let mut detector = StallDetector::new(start);

        assert!(!detector.is_stalled(POSITION, true, start));
        assert!(!detector.is_stalled(POSITION, true, start + STALL_TIMEOUT / 2));
        assert!(detector.is_stalled(POSITION, true, start + STALL_TIMEOUT));
    }

    #[test]
    fn progress_resets_stall_timer() {
        let start = Instant::now();
        let mut detector = StallDetector::new(start);

        assert!(!detector.is_stalled(POSITION, true, start));

        let moved_at = start + STALL_TIMEOUT - Duration::from_millis(1);
        let moved = POSITION + Duration::from_millis(5);
        assert!(!detector.is_stalled(moved, true, moved_at));
        assert!(!detector.is_stalled(moved, true, start + STALL_TIMEOUT));
    }

    #[test]
    fn paused_or_empty_player_is_not_stalled() {
        let start = Instant::now();
        let mut detector = StallDetector::new(start);

        assert!(!detector.is_stalled(POSITION, false, start));
        assert!(!detector.is_stalled(POSITION, false, start + STALL_TIMEOUT * 10));

        // 재생을 재개한 직후에는 일시정지 시간을 stall로 치지 않는다.
        assert!(!detector.is_stalled(POSITION, true, start + STALL_TIMEOUT * 10));
    }
}
