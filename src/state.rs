use std::path::PathBuf;

pub struct MainState {
    pub title: String,
    pub music_list: MusicList,
    pub on_play: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Music {
    pub title: String,
    pub file_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct MusicList {
    pub list: Vec<Music>,
}

impl Default for MusicList {
    fn default() -> Self {
        Self { list: vec![] }
    }
}
