use std::path::PathBuf;

use native_dialog::FileDialog;

pub fn open_directory_dialog() -> anyhow::Result<PathBuf> {
    let path = FileDialog::new().show_open_single_dir()?;

    let path = match path {
        Some(path) => path,
        None => {
            return Err(anyhow::anyhow!("No directory selected"));
        }
    };

    println!("Selected file: {:?}", path);

    Ok(path)
}
