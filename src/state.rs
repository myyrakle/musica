use std::path::PathBuf;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Default)]
pub struct MusicList {
    pub list: Vec<Music>,
}

impl MusicList {
    pub fn is_not_empty(&self) -> bool {
        !self.list.is_empty()
    }
}
