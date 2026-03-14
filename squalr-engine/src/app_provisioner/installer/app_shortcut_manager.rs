use crate::app_provisioner::installer::install_shortcut_options::InstallShortcutOptions;
use anyhow::Result;
use std::path::Path;

pub struct AppShortcutManager {}

impl AppShortcutManager {
    pub fn sync_shortcuts(
        install_directory: &Path,
        install_shortcut_options: &InstallShortcutOptions,
    ) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            return windows_shortcut_manager::WindowsShortcutManager::sync_shortcuts(install_directory, install_shortcut_options);
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = (install_directory, install_shortcut_options);
            Ok(())
        }
    }
}

#[cfg(target_os = "windows")]
mod windows_shortcut_manager {
    use crate::app_provisioner::installer::install_shortcut_options::InstallShortcutOptions;
    use anyhow::{Context, Result, anyhow};
    use std::path::{Path, PathBuf};
    use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize, IPersistFile};
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
    use windows::core::{HSTRING, Interface};

    pub(super) struct WindowsShortcutManager {}

    impl WindowsShortcutManager {
        const SHORTCUT_DISPLAY_NAME: &'static str = "Squalr";
        const SHORTCUT_DESCRIPTION: &'static str = "Launch Squalr.";
        const GUI_EXECUTABLE_NAME: &'static str = "squalr.exe";

        pub(super) fn sync_shortcuts(
            install_directory: &Path,
            install_shortcut_options: &InstallShortcutOptions,
        ) -> Result<()> {
            let gui_executable_path = install_directory.join(Self::GUI_EXECUTABLE_NAME);
            if !gui_executable_path.is_file() {
                return Err(anyhow!("Installed GUI executable was not found at {}", gui_executable_path.display()));
            }

            let start_menu_shortcut_path = Self::resolve_start_menu_shortcut_path()?;
            let desktop_shortcut_path = Self::resolve_desktop_shortcut_path()?;

            Self::sync_shortcuts_to_paths(
                &gui_executable_path,
                install_shortcut_options,
                &start_menu_shortcut_path,
                &desktop_shortcut_path,
            )
        }

        fn sync_shortcuts_to_paths(
            gui_executable_path: &Path,
            install_shortcut_options: &InstallShortcutOptions,
            start_menu_shortcut_path: &Path,
            desktop_shortcut_path: &Path,
        ) -> Result<()> {
            Self::sync_shortcut_path(
                start_menu_shortcut_path,
                install_shortcut_options.register_start_menu_shortcut,
                gui_executable_path,
            )?;
            Self::sync_shortcut_path(desktop_shortcut_path, install_shortcut_options.create_desktop_shortcut, gui_executable_path)?;

            Ok(())
        }

        fn sync_shortcut_path(
            shortcut_path: &Path,
            should_exist: bool,
            gui_executable_path: &Path,
        ) -> Result<()> {
            if should_exist {
                Self::create_shortcut(shortcut_path, gui_executable_path)?;
            } else {
                Self::remove_shortcut(shortcut_path)?;
            }

            Ok(())
        }

        fn resolve_start_menu_shortcut_path() -> Result<PathBuf> {
            let start_menu_programs_directory = dirs::data_dir()
                .context("Failed to resolve the roaming application data directory for the Start Menu shortcut")?
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs");

            Ok(start_menu_programs_directory.join(Self::shortcut_file_name()))
        }

        fn resolve_desktop_shortcut_path() -> Result<PathBuf> {
            let desktop_directory = dirs::desktop_dir().context("Failed to resolve the current user desktop directory for the desktop shortcut")?;

            Ok(desktop_directory.join(Self::shortcut_file_name()))
        }

        fn shortcut_file_name() -> String {
            format!("{}.lnk", Self::SHORTCUT_DISPLAY_NAME)
        }

        fn create_shortcut(
            shortcut_path: &Path,
            gui_executable_path: &Path,
        ) -> Result<()> {
            if let Some(shortcut_parent_directory) = shortcut_path.parent() {
                std::fs::create_dir_all(shortcut_parent_directory)
                    .with_context(|| format!("Failed to create shortcut directory {}", shortcut_parent_directory.display()))?;
            }

            if shortcut_path.exists() {
                std::fs::remove_file(shortcut_path).with_context(|| format!("Failed to replace existing shortcut {}", shortcut_path.display()))?;
            }

            let _com_guard = ComGuard::new()?;
            let shell_link: IShellLinkW =
                unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) }.context("Failed to create the Windows shell link COM instance")?;
            let working_directory = gui_executable_path
                .parent()
                .ok_or_else(|| anyhow!("Shortcut target {} had no parent directory", gui_executable_path.display()))?;

            unsafe {
                shell_link
                    .SetPath(&HSTRING::from(gui_executable_path.to_string_lossy().into_owned()))
                    .context("Failed to set the shell link executable path")?;
                shell_link
                    .SetWorkingDirectory(&HSTRING::from(working_directory.to_string_lossy().into_owned()))
                    .context("Failed to set the shell link working directory")?;
                shell_link
                    .SetDescription(&HSTRING::from(Self::SHORTCUT_DESCRIPTION))
                    .context("Failed to set the shell link description")?;
                shell_link
                    .SetIconLocation(&HSTRING::from(gui_executable_path.to_string_lossy().into_owned()), 0)
                    .context("Failed to set the shell link icon")?;
            }

            let persist_file: IPersistFile = shell_link
                .cast()
                .context("Failed to acquire the shell link persist file interface")?;
            unsafe {
                persist_file
                    .Save(&HSTRING::from(shortcut_path.to_string_lossy().into_owned()), true)
                    .context("Failed to save the shell link shortcut file")?;
            }

            Ok(())
        }

        fn remove_shortcut(shortcut_path: &Path) -> Result<()> {
            if shortcut_path.exists() {
                std::fs::remove_file(shortcut_path).with_context(|| format!("Failed to remove shortcut {}", shortcut_path.display()))?;
            }

            Ok(())
        }
    }

    struct ComGuard;

    impl ComGuard {
        fn new() -> Result<Self> {
            unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }
                .ok()
                .context("Failed to initialize COM for shortcut creation")?;
            Ok(Self)
        }
    }

    impl Drop for ComGuard {
        fn drop(&mut self) {
            unsafe {
                CoUninitialize();
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::WindowsShortcutManager;
        use crate::app_provisioner::installer::install_shortcut_options::InstallShortcutOptions;
        use tempfile::TempDir;

        #[test]
        fn sync_shortcuts_to_paths_creates_requested_shortcuts() -> anyhow::Result<()> {
            let install_directory = TempDir::new()?;
            let shortcut_directory = TempDir::new()?;
            let gui_executable_path = install_directory.path().join("squalr.exe");
            let start_menu_shortcut_path = shortcut_directory.path().join("start-menu.lnk");
            let desktop_shortcut_path = shortcut_directory.path().join("desktop.lnk");
            std::fs::write(&gui_executable_path, b"stub-executable")?;

            WindowsShortcutManager::sync_shortcuts_to_paths(
                &gui_executable_path,
                &InstallShortcutOptions {
                    register_start_menu_shortcut: true,
                    create_desktop_shortcut: true,
                },
                &start_menu_shortcut_path,
                &desktop_shortcut_path,
            )?;

            assert!(start_menu_shortcut_path.exists());
            assert!(desktop_shortcut_path.exists());
            Ok(())
        }

        #[test]
        fn sync_shortcuts_to_paths_removes_unchecked_shortcuts() -> anyhow::Result<()> {
            let install_directory = TempDir::new()?;
            let shortcut_directory = TempDir::new()?;
            let gui_executable_path = install_directory.path().join("squalr.exe");
            let start_menu_shortcut_path = shortcut_directory.path().join("start-menu.lnk");
            let desktop_shortcut_path = shortcut_directory.path().join("desktop.lnk");
            std::fs::write(&gui_executable_path, b"stub-executable")?;

            WindowsShortcutManager::sync_shortcuts_to_paths(
                &gui_executable_path,
                &InstallShortcutOptions {
                    register_start_menu_shortcut: true,
                    create_desktop_shortcut: true,
                },
                &start_menu_shortcut_path,
                &desktop_shortcut_path,
            )?;

            WindowsShortcutManager::sync_shortcuts_to_paths(
                &gui_executable_path,
                &InstallShortcutOptions {
                    register_start_menu_shortcut: false,
                    create_desktop_shortcut: false,
                },
                &start_menu_shortcut_path,
                &desktop_shortcut_path,
            )?;

            assert!(!start_menu_shortcut_path.exists());
            assert!(!desktop_shortcut_path.exists());
            Ok(())
        }
    }
}
