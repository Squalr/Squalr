use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use std::process::Stdio;

pub struct UpdateOperationLaunch {}

impl UpdateOperationLaunch {
    pub fn launch_app(app_executable_path: &Path) -> Result<()> {
        if !app_executable_path.exists() {
            let error = Error::new(ErrorKind::NotFound, format!("App executable not found at: {}", app_executable_path.display()));
            log::error!("{}", error);
            return Err(error);
        }

        let mut launch_command = std::process::Command::new(app_executable_path);
        if let Some(executable_directory) = app_executable_path.parent() {
            launch_command.current_dir(executable_directory);
        }
        launch_command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|_| {
                log::info!("Successfully launched Squalr");
            })
            .map_err(|error| {
                log::error!("Failed to launch Squalr: {}", error);
                error
            })
    }
}

#[cfg(test)]
mod tests {
    use super::UpdateOperationLaunch;
    use std::io::Result;
    use tempfile::TempDir;

    #[test]
    fn launch_app_returns_not_found_for_missing_executable() -> Result<()> {
        let temporary_directory = TempDir::new()?;
        let missing_executable_path = temporary_directory.path().join("missing.exe");

        let launch_result = UpdateOperationLaunch::launch_app(&missing_executable_path);

        assert!(launch_result.is_err());
        if let Err(error) = launch_result {
            assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn launch_app_spawns_existing_windows_executable() -> Result<()> {
        let windows_directory = std::env::var("WINDIR").map_err(|error| std::io::Error::other(format!("Failed to resolve WINDIR: {}", error)))?;
        let whoami_executable_path = std::path::Path::new(&windows_directory)
            .join("System32")
            .join("whoami.exe");

        let launch_result = UpdateOperationLaunch::launch_app(&whoami_executable_path);

        assert!(launch_result.is_ok());
        Ok(())
    }
}
