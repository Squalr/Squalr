use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

pub struct FileSystemUtils {}

impl FileSystemUtils {
    /// Determines whether a path should be treated as absolute across platforms.
    pub fn is_cross_platform_absolute_path(path: &Path) -> bool {
        if path.is_absolute() {
            return true;
        }

        let path_string = path.to_string_lossy();
        let path_bytes = path_string.as_bytes();

        path_bytes.len() >= 3 && path_bytes[0].is_ascii_alphabetic() && path_bytes[1] == b':' && (path_bytes[2] == b'/' || path_bytes[2] == b'\\')
    }

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
                fs::create_dir_all(&new_path)?;
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

#[cfg(test)]
mod tests {
    use super::FileSystemUtils;
    use std::path::Path;

    #[test]
    fn is_cross_platform_absolute_path_detects_unix_absolute_paths() {
        assert!(FileSystemUtils::is_cross_platform_absolute_path(Path::new("/tmp/test")));
    }

    #[test]
    fn is_cross_platform_absolute_path_detects_windows_drive_absolute_paths() {
        assert!(FileSystemUtils::is_cross_platform_absolute_path(Path::new("C:/Projects/TestProject")));
        assert!(FileSystemUtils::is_cross_platform_absolute_path(Path::new("D:\\Projects\\TestProject")));
    }

    #[test]
    fn is_cross_platform_absolute_path_rejects_relative_paths() {
        assert!(!FileSystemUtils::is_cross_platform_absolute_path(Path::new("Projects/TestProject")));
    }
}
