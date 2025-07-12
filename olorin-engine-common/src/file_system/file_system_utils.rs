use std::{
    env, fs, io,
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

    /// Gets the path to the current running executable.
    pub fn get_executable_path() -> PathBuf {
        match env::current_exe() {
            Ok(exe_path) => {
                return exe_path;
            }
            Err(error) => {
                log::error!("Failed to get executable directory: {}", error);
                return PathBuf::new();
            }
        }
    }

    /// Copies all elements from the source folder to the destination folder. Note that files in the destination are not pre-cleared.
    pub fn copy_dir_all(
        source_folder: impl AsRef<Path>,
        destination_folder: impl AsRef<Path>,
    ) -> io::Result<()> {
        std::fs::create_dir_all(&destination_folder)?;
        for entry in std::fs::read_dir(source_folder)? {
            let entry = entry?;
            let file_type = entry.file_type()?;

            if file_type.is_dir() {
                Self::copy_dir_all(entry.path(), destination_folder.as_ref().join(entry.file_name()))?;
            } else {
                std::fs::copy(entry.path(), destination_folder.as_ref().join(entry.file_name()))?;
            }
        }
        Ok(())
    }
}
