use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileInfo {
    pub filename: String,
    pub filepath: PathBuf,
}

impl FileInfo {
    pub fn is_music_file(&self) -> bool {
        let ext = self.filepath.extension().unwrap_or_default();
        ext == "mp3" || ext == "ogg"
    }
}

pub fn read_file_list(path: &Path) -> anyhow::Result<Vec<FileInfo>> {
    let read_dir_result = fs::read_dir(path)?;

    let mut file_list = vec![];

    for read_dir_entry in read_dir_result {
        if let Ok(read_dir_entry) = read_dir_entry {
            if let Ok(metadata) = read_dir_entry.metadata() {
                if metadata.is_file() {
                    let filepath = read_dir_entry.path();
                    let filename = read_dir_entry
                        .file_name()
                        .to_str()
                        .unwrap_or("error")
                        .to_owned();
                    file_list.push(FileInfo { filename, filepath })
                }
            }
        }
    }

    Ok(file_list)
}
