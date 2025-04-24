use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub struct FileSystemUtils {}

impl FileSystemUtils {
    /// Creates a uniquely named directory like "New Folder", "New Folder (1)", etc. within the given base directory.
    pub fn create_unique_folder(
        base: &Path,
        base_name: &str,
    ) -> io::Result<PathBuf> {
        let mut index = 0;

        loop {
            let folder_name = if index == 0 {
                base_name.to_string()
            } else {
                format!("{} ({})", base_name, index)
            };

            let new_path = base.join(&folder_name);
            if !new_path.exists() {
                fs::create_dir(&new_path)?;
                return Ok(new_path);
            }

            index += 1;
        }
    }
}
