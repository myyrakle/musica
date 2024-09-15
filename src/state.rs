pub struct MainState {
    pub title: String,
    pub value: i32,
    pub music_list: MusicList,
    pub on_play: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Music {
    pub title: String,
    pub file_path: String,
}

#[derive(Debug, Clone)]
pub struct MusicList {
    pub list: Vec<Music>,
}

impl Default for MusicList {
    fn default() -> Self {
        Self {
            list: vec![
                Music {
                    title: "test1".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test2".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test3".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test4".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test5".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test6".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test7".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test8".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test9".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test10".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test11".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test12".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test13".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test14".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test10".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test11".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test12".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test13".into(),
                    file_path: "test".into(),
                },
                Music {
                    title: "test14".into(),
                    file_path: "test".into(),
                },
            ],
        }
    }
}
