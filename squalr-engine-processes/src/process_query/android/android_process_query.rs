use crate::process_info::{Bitness, OpenedProcessInfo, ProcessIcon, ProcessInfo};
use crate::process_query::android::android_process_monitor::AndroidProcessMonitor;
use crate::process_query::process_query_options::ProcessQueryOptions;
use crate::process_query::process_queryer::ProcessQueryer;
use image::ImageReader;
use once_cell::sync::Lazy;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::fs;
use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use std::path::Path;
use std::sync::RwLock;
use zip::ZipArchive;

pub(crate) static PROCESS_MONITOR: Lazy<RwLock<AndroidProcessMonitor>> = Lazy::new(|| RwLock::new(AndroidProcessMonitor::new()));

/// Minimum UID for user-installed apps.
const MIN_USER_UID: u32 = 10000;
const ICON_PATHS: [&str; 5] = [
    "res/mipmap-mdpi-v4/ic_launcher.png",
    "res/mipmap-hdpi-v4/ic_launcher.png",
    "res/mipmap-xhdpi-v4/ic_launcher.png",
    "res/mipmap-mdpi/ic_launcher.png",
    "res/mipmap-hdpi/ic_launcher.png",
];

pub struct AndroidProcessQuery {}

impl AndroidProcessQuery {
    /// Checks if a process belongs to a user app (UID ≥ 10000).
    fn is_user_app(process_id: u32) -> bool {
        let status_path = format!("/proc/{}/status", process_id);
        if let Ok(status) = fs::read_to_string(status_path) {
            for line in status.lines() {
                if line.starts_with("Uid:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() > 1 {
                        if let Ok(uid) = parts[1].parse::<u32>() {
                            return uid >= MIN_USER_UID;
                        }
                    }
                }
            }
        }

        false
    }

    /// Searches for the APK path under `/data/app/` for a given package name.
    /// Handles obfuscated directories (Android 11+).
    pub fn find_apk_path(package_name: &str) -> Option<String> {
        let data_app_path = Path::new("/data/app/");

        if !data_app_path.exists() {
            return None;
        }

        // Iterate over top-level directories in `/data/app/`
        if let Ok(entries) = fs::read_dir(data_app_path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();

                // If it's a directory, search inside it
                if entry_path.is_dir() {
                    if let Some(apk_path) = Self::search_package_dir(&entry_path, package_name) {
                        return Some(apk_path);
                    }
                }
            }
        }

        None
    }

    /// Recursively searches a given directory for a matching package name subdirectory and base.apk.
    fn search_package_dir(
        parent_dir: &Path,
        package_name: &str,
    ) -> Option<String> {
        if let Ok(entries) = fs::read_dir(parent_dir) {
            for entry in entries.flatten() {
                let entry_path = entry.path();

                // Check if this is a package directory (com.example.app-XYZ)
                if entry_path.is_dir() {
                    if let Some(dir_name) = entry_path.file_name().and_then(|n| n.to_str()) {
                        if dir_name.starts_with(package_name) {
                            let apk_path = entry_path.join("base.apk");

                            if apk_path.exists() {
                                return Some(apk_path.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn get_icon_from_apk(apk_path: &str) -> Option<ProcessIcon> {
        let file = File::open(apk_path).ok()?;
        let mut archive = ZipArchive::new(file).ok()?;

        for icon_path in ICON_PATHS {
            if let Ok(mut icon_file) = archive.by_name(icon_path) {
                let mut icon_data = Vec::new();
                if icon_file.read_to_end(&mut icon_data).is_ok() {
                    let reader = ImageReader::new(Cursor::new(icon_data))
                        .with_guessed_format()
                        .ok()?;

                    if let Ok(img) = reader.decode() {
                        let rgba_img = img.to_rgba8();
                        let (width, height) = rgba_img.dimensions();
                        let bytes_rgba = rgba_img.into_raw();

                        return Some(ProcessIcon { bytes_rgba, width, height });
                    }
                }
            }
        }

        None
    }
}

impl ProcessQueryer for AndroidProcessQuery {
    fn start_monitoring() -> Result<(), String> {
        let mut monitor = PROCESS_MONITOR
            .write()
            .map_err(|err| format!("Failed to acquire process monitor lock: {}", err))?;

        Logger::get_instance().log(LogLevel::Error, "Monitoring system processes...", None);
        monitor.start_monitoring();

        Ok(())
    }

    fn stop_monitoring() -> Result<(), String> {
        let mut monitor = PROCESS_MONITOR
            .write()
            .map_err(|err| format!("Failed to acquire process monitor lock: {}", err))?;

        monitor.stop_monitoring();

        Ok(())
    }

    // Android has no concept of opening a process -- do nothing, return 0 for handle.
    fn open_process(process_info: &ProcessInfo) -> Result<OpenedProcessInfo, String> {
        Ok(OpenedProcessInfo {
            process_id: process_info.process_id,
            name: process_info.name.clone(),
            handle: 0,
            bitness: Bitness::Bit64,
            icon: process_info.icon.clone(),
        })
    }

    // Android has no concept of closing a process -- do nothing.
    fn close_process(_handle: u64) -> Result<(), String> {
        Ok(())
    }

    fn get_processes(options: ProcessQueryOptions) -> Vec<ProcessInfo> {
        let process_monitor_guard = match PROCESS_MONITOR.read() {
            Ok(guard) => guard,
            Err(err) => {
                Logger::get_instance().log(LogLevel::Error, &format!("Failed to acquire process monitor lock: {}", err), None);
                return Vec::new();
            }
        };

        let all_processes_lock = process_monitor_guard.get_all_processes();
        let zygote_processes_lock = process_monitor_guard.get_zygote_processes();

        let all_processes_guard = match all_processes_lock.read() {
            Ok(guard) => guard,
            Err(err) => {
                Logger::get_instance().log(LogLevel::Error, &format!("Failed to acquire process read lock: {}", err), None);
                return Vec::new();
            }
        };

        let zygote_processes_guard = match zygote_processes_lock.read() {
            Ok(guard) => guard,
            Err(err) => {
                Logger::get_instance().log(LogLevel::Error, &format!("Failed to acquire zygote process read lock: {}", err), None);
                return Vec::new();
            }
        };

        let all_processes = all_processes_guard.clone();
        let zygote_processes = zygote_processes_guard.clone();
        let mut results = Vec::new();

        for android_process_info in all_processes.values() {
            let apk_path = Self::find_apk_path(&android_process_info.package_name);
            let is_windowed = apk_path.is_some()
                && zygote_processes.contains_key(&android_process_info.parent_process_id)
                && Self::is_user_app(android_process_info.process_id);

            Logger::get_instance().log(LogLevel::Info, &format!("APK: {:?}", apk_path), None);

            let icon = if let Some(apk_path) = apk_path {
                Self::get_icon_from_apk(&apk_path)
            } else {
                None
            };

            let process_info = ProcessInfo {
                process_id: android_process_info.process_id,
                name: android_process_info.package_name.clone(),
                is_windowed,
                icon: icon,
            };
            let mut matches = true;

            if let Some(ref term) = options.search_name {
                if options.match_case {
                    matches &= process_info.name.contains(term);
                } else {
                    matches &= process_info.name.to_lowercase().contains(&term.to_lowercase());
                }
            }

            if let Some(required_process_id) = options.required_process_id {
                matches &= process_info.process_id == required_process_id.as_u32();
            }

            if options.require_windowed {
                matches &= process_info.is_windowed;
            }

            if matches {
                results.push(process_info);
            }

            if let Some(limit) = options.limit {
                if results.len() >= limit as usize {
                    break;
                }
            }
        }

        results
    }
}
