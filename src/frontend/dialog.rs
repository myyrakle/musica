use std::path::PathBuf;

use native_dialog::DialogBuilder;

pub fn open_directory_dialog() -> anyhow::Result<PathBuf> {
    let path = DialogBuilder::file()
        .open_single_dir()
        .show()?;

    let path = match path {
        Some(path) => path,
        None => {
            return Err(anyhow::anyhow!("No directory selected"));
        }
    };

    println!("Selected file: {:?}", path);

    Ok(path)
}
