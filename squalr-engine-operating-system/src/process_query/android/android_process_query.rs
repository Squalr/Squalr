use crate::process_info::{Bitness, OpenedProcessInfo, ProcessIcon, ProcessInfo};
use crate::process_query::android::android_process_info::AndroidProcessInfo;
use crate::process_query::process_query_error::ProcessQueryError;
use crate::process_query::process_query_options::ProcessQueryOptions;
use crate::process_query::process_queryer::ProcessQueryer;
use image::ImageReader;
use regex::bytes::Regex;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::Cursor;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

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

    /// Parses `/data/system/packages.xml` (ABX Binary XML) and extracts package → APK path mapping.
    /// This is done with a greedy binary regex solution, to avoid the need to write a complex binary XML parser.
    fn parse_packages_xml() -> HashMap<String, String> {
        let mut package_map = HashMap::new();
        Logger::log(LogLevel::Info, "Scanning packages.xml...", None);
        let file = match File::open("/data/system/packages.xml") {
            Ok(file) => file,
            Err(error) => {
                Logger::log(LogLevel::Error, &format!("Error opening packages.xml: {}", error), None);
                return package_map;
            }
        };

        let mut buffer = vec![];
        if let Err(error) = BufReader::new(file).read_to_end(&mut buffer) {
            Logger::log(LogLevel::Error, &format!("Error reading packages.xml: {}", error), None);
            return package_map;
        }

        let regex = match Regex::new(r"/data/app(?:/[^/]+)?/(?P<package_name>[a-zA-Z0-9_.]+)-[^/]+/") {
            Ok(regex) => regex,
            Err(error) => {
                Logger::log(LogLevel::Error, &format!("Failed to compile regex: {}", error), None);
                return package_map;
            }
        };

        for capture_result in regex.captures_iter(&buffer) {
            if let Some(path_match) = capture_result.get(0) {
                if let Some(package_name_match) = capture_result.name("package_name") {
                    let package_name = String::from_utf8_lossy(package_name_match.as_bytes()).to_string();
                    let package_path = String::from_utf8_lossy(path_match.as_bytes()).to_string();

                    package_map.insert(package_name, package_path);
                }
            }
        }

        Logger::log(LogLevel::Info, &format!("Found {} packages.", package_map.len()), None);

        package_map
    }

    fn get_apk_path(
        package_name: &str,
        package_paths: &HashMap<String, String>,
    ) -> Option<String> {
        package_paths.get(package_name).cloned().map(|mut path| {
            if Path::new(&path).is_dir() {
                path.push_str("/base.apk");
            }
            path
        })
    }

    fn get_icon_from_apk(apk_path: &str) -> Option<ProcessIcon> {
        let file = File::open(apk_path).ok()?;
        let mut archive = ZipArchive::new(file).ok()?;

        for icon_path in ICON_PATHS {
            if let Ok(mut icon_file) = archive.by_name(icon_path) {
                let mut icon_data = vec![];
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

    fn get_live_process_maps() -> (HashMap<u32, AndroidProcessInfo>, HashMap<u32, AndroidProcessInfo>) {
        let mut all_processes = HashMap::new();
        let mut zygote_processes = HashMap::new();

        if let Ok(process_entries) = fs::read_dir("/proc") {
            for process_entry in process_entries.flatten() {
                if let Ok(process_id_string) = process_entry.file_name().into_string() {
                    if process_id_string
                        .chars()
                        .all(|char_value| char_value.is_ascii_digit())
                    {
                        if let Ok(process_id) = process_id_string.parse::<u32>() {
                            let mut package_name = String::new();
                            let mut parent_process_id = 0;

                            let cmdline_path = format!("/proc/{}/cmdline", process_id);
                            if let Ok(cmdline_content) = fs::read_to_string(&cmdline_path) {
                                package_name = cmdline_content
                                    .split('\0')
                                    .next()
                                    .unwrap_or("")
                                    .split(':')
                                    .next()
                                    .unwrap_or("")
                                    .to_string();
                            }

                            let stat_path = format!("/proc/{}/stat", process_id);
                            if let Ok(stat_content) = fs::read_to_string(&stat_path) {
                                let stat_parts: Vec<&str> = stat_content.split_whitespace().collect();
                                if stat_parts.len() > 3 {
                                    if let Ok(parsed_parent_process_id) = stat_parts[3].parse::<u32>() {
                                        parent_process_id = parsed_parent_process_id;
                                    }
                                }
                            }

                            let process_info = AndroidProcessInfo {
                                process_id,
                                parent_process_id,
                                package_name: package_name.clone(),
                            };

                            all_processes.insert(process_id, process_info.clone());

                            if package_name == "zygote" || package_name == "zygote64" {
                                zygote_processes.insert(process_id, process_info);
                            }
                        }
                    }
                }
            }
        }

        (all_processes, zygote_processes)
    }
}

impl ProcessQueryer for AndroidProcessQuery {
    fn start_monitoring() -> Result<(), ProcessQueryError> {
        // Android process query now exposes immediate operations only.
        Ok(())
    }

    fn stop_monitoring() -> Result<(), ProcessQueryError> {
        // Android process query now exposes immediate operations only.
        Ok(())
    }

    // Android has no concept of opening a process -- do nothing, return 0 for handle.
    fn open_process(process_info: &ProcessInfo) -> Result<OpenedProcessInfo, ProcessQueryError> {
        Ok(OpenedProcessInfo {
            process_id: process_info.process_id,
            name: process_info.name.clone(),
            handle: 0,
            bitness: Bitness::Bit64,
            icon: process_info.icon.clone(),
        })
    }

    // Android has no concept of closing a process -- do nothing.
    fn close_process(_handle: u64) -> Result<(), ProcessQueryError> {
        Ok(())
    }

    fn get_processes(options: ProcessQueryOptions) -> Vec<ProcessInfo> {
        let package_paths = Self::parse_packages_xml();
        let (all_processes, zygote_processes) = Self::get_live_process_maps();
        let mut results = Vec::new();

        for android_process_info in all_processes.values() {
            let apk_path = Self::get_apk_path(&android_process_info.package_name, &package_paths);
            let is_windowed = apk_path.is_some()
                && zygote_processes.contains_key(&android_process_info.parent_process_id)
                && Self::is_user_app(android_process_info.process_id);

            let icon = if let Some(apk_path) = apk_path {
                Self::get_icon_from_apk(&apk_path)
            } else {
                None
            };

            let process_info = ProcessInfo {
                process_id: android_process_info.process_id,
                name: android_process_info.package_name.clone(),
                is_windowed,
                icon,
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
