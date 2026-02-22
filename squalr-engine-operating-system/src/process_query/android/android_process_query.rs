use crate::process_query::android::android_process_info::AndroidProcessInfo;
use crate::process_query::process_query_error::ProcessQueryError;
use crate::process_query::process_query_options::ProcessQueryOptions;
use crate::process_query::process_queryer::ProcessQueryer;
use image::ImageReader;
use regex::bytes::Regex;
use squalr_engine_api::structures::memory::bitness::Bitness;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::processes::process_icon::ProcessIcon;
use squalr_engine_api::structures::processes::process_info::ProcessInfo;
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
    fn read_cmdline_process_name(process_id: u32) -> Option<String> {
        let cmdline_path = format!("/proc/{}/cmdline", process_id);
        let cmdline_bytes = fs::read(cmdline_path).ok()?;
        let cmdline_name = cmdline_bytes
            .split(|byte_value| *byte_value == 0)
            .next()
            .map(|name_bytes| String::from_utf8_lossy(name_bytes).trim().to_string())?;

        if cmdline_name.is_empty() { None } else { Some(cmdline_name) }
    }

    fn read_comm_process_name(process_id: u32) -> Option<String> {
        let comm_path = format!("/proc/{}/comm", process_id);
        let comm_name = fs::read_to_string(comm_path).ok()?.trim().to_string();

        if comm_name.is_empty() { None } else { Some(comm_name) }
    }

    fn extract_package_name(cmdline_process_name: &str) -> Option<String> {
        let package_name_candidate = cmdline_process_name
            .split(':')
            .next()
            .map(str::trim)
            .unwrap_or_default();

        if package_name_candidate.is_empty()
            || package_name_candidate.starts_with('/')
            || !package_name_candidate.contains('.')
            || !package_name_candidate
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_' || character == '.')
        {
            return None;
        }

        Some(package_name_candidate.to_string())
    }

    /// Checks if a process belongs to a user app (UID >= 10000).
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

    /// Parses `/data/system/packages.xml` (ABX Binary XML) and extracts package-to-APK path mapping.
    /// This is done with a greedy binary regex solution to avoid writing a complex binary XML parser.
    fn parse_packages_xml() -> HashMap<String, String> {
        let mut package_map = HashMap::new();
        log::info!("Scanning packages.xml.");
        let file = match File::open("/data/system/packages.xml") {
            Ok(file) => file,
            Err(error) => {
                log::error!("Error opening packages.xml: {}", error);
                return package_map;
            }
        };

        let mut buffer = vec![];
        if let Err(error) = BufReader::new(file).read_to_end(&mut buffer) {
            log::error!("Error reading packages.xml: {}", error);
            return package_map;
        }

        let regex = match Regex::new(r"/data/app(?:/[^/]+)?/(?P<package_name>[a-zA-Z0-9_.]+)-[^/]+/") {
            Ok(regex) => regex,
            Err(error) => {
                log::error!("Failed to compile regex: {}", error);
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

        log::info!("Found {} packages.", package_map.len());

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

                    if let Ok(image) = reader.decode() {
                        let rgba_image = image.to_rgba8();
                        let (width, height) = rgba_image.dimensions();
                        let bytes_rgba = rgba_image.into_raw();

                        return Some(ProcessIcon::new(bytes_rgba, width, height));
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
                            let cmdline_process_name = Self::read_cmdline_process_name(process_id);
                            let process_name = cmdline_process_name
                                .clone()
                                .or_else(|| Self::read_comm_process_name(process_id))
                                .unwrap_or_default();
                            if process_name.is_empty() {
                                continue;
                            }

                            let package_name = cmdline_process_name
                                .as_deref()
                                .and_then(Self::extract_package_name);
                            let mut parent_process_id = 0;

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
                                process_name: process_name.clone(),
                                package_name: package_name.clone(),
                            };

                            all_processes.insert(process_id, process_info.clone());

                            if process_name == "zygote" || process_name == "zygote64" {
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

    // Android has no concept of opening a process, so return a zero handle.
    fn open_process(process_info: &ProcessInfo) -> Result<OpenedProcessInfo, ProcessQueryError> {
        Ok(OpenedProcessInfo::new(
            process_info.get_process_id(),
            process_info.get_name().to_string(),
            0,
            Bitness::Bit64,
            process_info.get_icon().clone(),
        ))
    }

    // Android has no concept of closing a process.
    fn close_process(_handle: u64) -> Result<(), ProcessQueryError> {
        Ok(())
    }

    fn get_processes(options: ProcessQueryOptions) -> Vec<ProcessInfo> {
        let package_paths = Self::parse_packages_xml();
        let (all_processes, zygote_processes) = Self::get_live_process_maps();
        let mut results = Vec::new();

        for android_process_info in all_processes.values() {
            let apk_path = android_process_info
                .package_name
                .as_deref()
                .and_then(|package_name| Self::get_apk_path(package_name, &package_paths));
            let is_windowed = apk_path.is_some()
                && zygote_processes.contains_key(&android_process_info.parent_process_id)
                && Self::is_user_app(android_process_info.process_id);

            let icon = if options.fetch_icons {
                if let Some(apk_path) = apk_path.as_deref() {
                    Self::get_icon_from_apk(apk_path)
                } else {
                    None
                }
            } else {
                None
            };

            let process_info = ProcessInfo::new(android_process_info.process_id, android_process_info.process_name.clone(), is_windowed, icon);
            let mut matches = true;

            if let Some(ref term) = options.search_name {
                if options.match_case {
                    matches &= process_info.get_name().contains(term);
                } else {
                    matches &= process_info
                        .get_name()
                        .to_lowercase()
                        .contains(&term.to_lowercase());
                }
            }

            if let Some(required_process_id) = options.required_process_id {
                matches &= process_info.get_process_id() == required_process_id.as_u32();
            }

            if options.require_windowed {
                matches &= process_info.get_is_windowed();
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

#[cfg(test)]
mod tests {
    use super::AndroidProcessQuery;

    #[test]
    fn extract_package_name_uses_base_name_without_process_suffix() {
        let package_name = AndroidProcessQuery::extract_package_name("com.squalr.android:worker");

        assert_eq!(package_name.as_deref(), Some("com.squalr.android"));
    }

    #[test]
    fn extract_package_name_rejects_paths() {
        let package_name = AndroidProcessQuery::extract_package_name("/system/bin/surfaceflinger");

        assert!(package_name.is_none());
    }
}
