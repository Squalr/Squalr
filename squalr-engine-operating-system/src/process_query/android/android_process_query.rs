use crate::process_query::android::android_process_info::AndroidProcessInfo;
use crate::process_query::process_query_error::ProcessQueryError;
use crate::process_query::process_query_options::ProcessQueryOptions;
use crate::process_query::process_queryer::ProcessQueryer;
use image::ImageReader;
use regex::Regex;
use regex::bytes::Regex as BytesRegex;
use squalr_engine_api::structures::memory::bitness::Bitness;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::processes::process_icon::ProcessIcon;
use squalr_engine_api::structures::processes::process_info::ProcessInfo;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::Cursor;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use zip::ZipArchive;

/// Minimum UID for user-installed apps.
const MIN_USER_UID: u32 = 10000;
const PACKAGE_ACTIVITY_PATTERN: &str = r"(?P<package_name>[A-Za-z0-9_]+(?:\.[A-Za-z0-9_]+)+)/";
const ICON_PATHS: [&str; 5] = [
    "res/mipmap-mdpi-v4/ic_launcher.png",
    "res/mipmap-hdpi-v4/ic_launcher.png",
    "res/mipmap-xhdpi-v4/ic_launcher.png",
    "res/mipmap-mdpi/ic_launcher.png",
    "res/mipmap-hdpi/ic_launcher.png",
];
const SUPPORTED_ICON_EXTENSIONS: [&str; 4] = ["png", "webp", "jpg", "jpeg"];

pub struct AndroidProcessQuery {}

impl AndroidProcessQuery {
    fn is_zygote_process_name(process_name: Option<&str>) -> bool {
        let normalized_process_name = process_name
            .unwrap_or_default()
            .trim()
            .rsplit('/')
            .next()
            .unwrap_or_default();

        matches!(
            normalized_process_name,
            "zygote" | "zygote64" | "app_zygote" | "app_zygote64" | "webview_zygote"
        )
    }

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

    fn extract_package_name_from_apk_directory(apk_directory_name: &str) -> Option<String> {
        let package_name_candidate = apk_directory_name.split('-').next().unwrap_or_default().trim();

        if package_name_candidate.is_empty()
            || !package_name_candidate.contains('.')
            || !package_name_candidate
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_' || character == '.')
        {
            return None;
        }

        Some(package_name_candidate.to_string())
    }

    fn is_primary_package_process(
        cmdline_process_name: Option<&str>,
        package_name: Option<&str>,
    ) -> bool {
        match (cmdline_process_name, package_name) {
            (Some(cmdline_process_name), Some(package_name)) => cmdline_process_name == package_name,
            _ => false,
        }
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

    /// Parses `/data/app` package directories and extracts package-to-APK path mapping.
    fn parse_data_app_directories() -> HashMap<String, String> {
        let mut package_map = HashMap::new();
        let data_app_root = Path::new("/data/app");

        let root_entries = match fs::read_dir(data_app_root) {
            Ok(entries) => entries,
            Err(error) => {
                log::error!("Error reading {}: {}", data_app_root.display(), error);
                return package_map;
            }
        };

        for root_entry in root_entries.flatten() {
            let root_path = root_entry.path();
            let root_name = root_entry.file_name().to_string_lossy().to_string();
            if !root_path.is_dir() {
                continue;
            }

            let mut candidate_directories = Vec::new();
            if root_name.starts_with("~~") {
                if let Ok(child_entries) = fs::read_dir(&root_path) {
                    for child_entry in child_entries.flatten() {
                        if child_entry.path().is_dir() {
                            candidate_directories.push(child_entry.path());
                        }
                    }
                }
            } else {
                candidate_directories.push(root_path);
            }

            for apk_directory_path in candidate_directories {
                let apk_directory_name = match apk_directory_path
                    .file_name()
                    .and_then(|file_name| file_name.to_str())
                {
                    Some(file_name) => file_name,
                    None => continue,
                };

                let package_name = match Self::extract_package_name_from_apk_directory(apk_directory_name) {
                    Some(package_name) => package_name,
                    None => continue,
                };

                let base_apk_path = apk_directory_path.join("base.apk");
                if base_apk_path.is_file() {
                    package_map.insert(package_name, base_apk_path.to_string_lossy().to_string());
                }
            }
        }

        package_map
    }

    /// Parses `/data/system/packages.xml` (ABX Binary XML) and extracts package-to-APK path mapping.
    /// This is done with a greedy binary regex solution to avoid writing a complex binary XML parser.
    fn parse_packages_xml() -> HashMap<String, String> {
        let mut package_map = HashMap::new();
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

        let regex = match BytesRegex::new(r"/data/app(?:/[^/]+)?/(?P<package_name>[a-zA-Z0-9_.]+)-[^/]+/") {
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

        package_map
    }

    fn parse_visible_windowed_packages_from_dumpsys_output(dumpsys_output: &str) -> HashSet<String> {
        let package_name_regex = match Regex::new(PACKAGE_ACTIVITY_PATTERN) {
            Ok(package_name_regex) => package_name_regex,
            Err(error) => {
                log::error!("Failed to compile visible package regex: {}", error);
                return HashSet::new();
            }
        };
        let mut visible_windowed_packages = HashSet::new();

        for regex_capture in package_name_regex.captures_iter(dumpsys_output) {
            let package_name = regex_capture
                .name("package_name")
                .map(|package_name_match| package_name_match.as_str().trim().to_string())
                .unwrap_or_default();
            if package_name.is_empty() {
                continue;
            }

            visible_windowed_packages.insert(package_name);
        }

        visible_windowed_packages
    }

    fn query_visible_windowed_packages() -> HashSet<String> {
        let dumpsys_argument_sets = [
            vec!["window", "visible-apps"],
            vec!["window", "windows"],
            vec!["activity", "activities"],
        ];

        for dumpsys_arguments in dumpsys_argument_sets {
            let dumpsys_output = match Command::new("dumpsys").args(&dumpsys_arguments).output() {
                Ok(command_output) => command_output,
                Err(error) => {
                    log::warn!("Failed running dumpsys {}: {}", dumpsys_arguments.join(" "), error);
                    continue;
                }
            };

            if !dumpsys_output.status.success() {
                let stderr_output = String::from_utf8_lossy(&dumpsys_output.stderr);
                log::warn!("dumpsys {} returned non-zero status: {}", dumpsys_arguments.join(" "), stderr_output.trim());
                continue;
            }

            let stdout_output = String::from_utf8_lossy(&dumpsys_output.stdout);
            let visible_windowed_packages = Self::parse_visible_windowed_packages_from_dumpsys_output(&stdout_output);
            if !visible_windowed_packages.is_empty() {
                return visible_windowed_packages;
            }
        }

        HashSet::new()
    }

    fn parse_installed_package_paths() -> HashMap<String, String> {
        let package_map_from_data_app = Self::parse_data_app_directories();
        if !package_map_from_data_app.is_empty() {
            return package_map_from_data_app;
        }

        log::warn!("No package paths found in /data/app. Falling back to packages.xml parsing.");
        let package_map_from_packages_xml = Self::parse_packages_xml();
        if !package_map_from_packages_xml.is_empty() {
            return package_map_from_packages_xml;
        }

        log::warn!("No package paths found in packages.xml. Falling back to package manager parsing.");
        Self::parse_package_manager_packages()
    }

    fn parse_package_manager_packages() -> HashMap<String, String> {
        let mut package_map = HashMap::new();
        let package_manager_output = match Command::new("pm")
            .arg("list")
            .arg("packages")
            .arg("-f")
            .output()
        {
            Ok(output) => output,
            Err(error) => {
                log::error!("Failed to execute package manager query: {}", error);
                return package_map;
            }
        };

        if !package_manager_output.status.success() {
            let error_output = String::from_utf8_lossy(&package_manager_output.stderr);
            log::error!("Package manager query failed: {}", error_output.trim());
            return package_map;
        }

        let package_list_output = String::from_utf8_lossy(&package_manager_output.stdout);
        for package_list_line in package_list_output.lines() {
            if let Some((package_name, apk_path)) = Self::parse_package_manager_entry(package_list_line) {
                package_map.insert(package_name, apk_path);
            }
        }

        package_map
    }

    fn parse_package_manager_entry(package_list_line: &str) -> Option<(String, String)> {
        let package_line = package_list_line.trim();
        if !package_line.starts_with("package:") {
            return None;
        }

        let entry_without_prefix = package_line.trim_start_matches("package:");
        let (apk_path, package_name) = entry_without_prefix.rsplit_once('=')?;
        let parsed_apk_path = apk_path.trim();
        let parsed_package_name = package_name.trim();

        if parsed_apk_path.is_empty() || parsed_package_name.is_empty() {
            return None;
        }

        Some((parsed_package_name.to_string(), parsed_apk_path.to_string()))
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

        // Fast path for older launcher resource layouts.
        for icon_path in ICON_PATHS {
            if let Ok(mut icon_file) = archive.by_name(icon_path) {
                let mut icon_data = vec![];
                if icon_file.read_to_end(&mut icon_data).is_ok() {
                    if let Some(process_icon) = Self::decode_process_icon(icon_data) {
                        return Some(process_icon);
                    }
                }
            }
        }

        // Modern APKs often use non-ic_launcher names and WEBP density variants.
        let mut scored_icon_paths = Self::collect_scored_icon_paths(&mut archive);
        scored_icon_paths.sort_by(|left_entry, right_entry| {
            right_entry
                .0
                .cmp(&left_entry.0)
                .then_with(|| left_entry.1.cmp(&right_entry.1))
        });

        for (_, icon_path) in scored_icon_paths {
            if let Ok(mut icon_file) = archive.by_name(&icon_path) {
                let mut icon_data = vec![];
                if icon_file.read_to_end(&mut icon_data).is_ok() {
                    if let Some(process_icon) = Self::decode_process_icon(icon_data) {
                        return Some(process_icon);
                    }
                }
            }
        }

        None
    }

    fn decode_process_icon(icon_data: Vec<u8>) -> Option<ProcessIcon> {
        let image_reader = ImageReader::new(Cursor::new(icon_data))
            .with_guessed_format()
            .ok()?;
        let decoded_image = image_reader.decode().ok()?;
        let rgba_image = decoded_image.to_rgba8();
        let (icon_width, icon_height) = rgba_image.dimensions();
        let icon_bytes_rgba = rgba_image.into_raw();

        Some(ProcessIcon::new(icon_bytes_rgba, icon_width, icon_height))
    }

    fn collect_scored_icon_paths(archive: &mut ZipArchive<File>) -> Vec<(i32, String)> {
        let mut scored_icon_paths = Vec::new();

        for archive_entry_index in 0..archive.len() {
            let archive_entry_name = {
                let archive_entry = match archive.by_index(archive_entry_index) {
                    Ok(archive_entry) => archive_entry,
                    Err(_) => continue,
                };
                archive_entry.name().to_string()
            };

            if let Some(icon_score) = Self::score_icon_archive_entry(&archive_entry_name) {
                scored_icon_paths.push((icon_score, archive_entry_name));
            }
        }

        scored_icon_paths
    }

    fn score_icon_archive_entry(archive_entry_name: &str) -> Option<i32> {
        let normalized_archive_entry_name = archive_entry_name.to_ascii_lowercase();

        if !normalized_archive_entry_name.starts_with("res/") {
            return None;
        }

        if !(normalized_archive_entry_name.contains("/mipmap") || normalized_archive_entry_name.contains("/drawable")) {
            return None;
        }

        let icon_extension = Path::new(&normalized_archive_entry_name)
            .extension()
            .and_then(|icon_extension| icon_extension.to_str())?;
        if !Self::is_supported_icon_extension(icon_extension) {
            return None;
        }

        let mut icon_score = 0;

        if normalized_archive_entry_name.contains("/mipmap") {
            icon_score += 150;
        }

        if normalized_archive_entry_name.contains("ic_launcher") {
            icon_score += 250;
        }
        if normalized_archive_entry_name.contains("launcher") {
            icon_score += 150;
        }
        if normalized_archive_entry_name.contains("app_icon") {
            icon_score += 125;
        }
        if normalized_archive_entry_name.contains("icon") {
            icon_score += 50;
        }
        if normalized_archive_entry_name.contains("foreground") || normalized_archive_entry_name.contains("background") {
            icon_score -= 80;
        }

        if normalized_archive_entry_name.contains("xxxhdpi") {
            icon_score += 90;
        } else if normalized_archive_entry_name.contains("xxhdpi") {
            icon_score += 80;
        } else if normalized_archive_entry_name.contains("xhdpi") {
            icon_score += 70;
        } else if normalized_archive_entry_name.contains("hdpi") {
            icon_score += 60;
        } else if normalized_archive_entry_name.contains("mdpi") {
            icon_score += 50;
        } else if normalized_archive_entry_name.contains("ldpi") {
            icon_score += 40;
        } else if normalized_archive_entry_name.contains("anydpi") {
            icon_score += 25;
        }

        if icon_extension == "png" {
            icon_score += 15;
        } else if icon_extension == "webp" {
            icon_score += 10;
        }

        Some(icon_score)
    }

    fn is_supported_icon_extension(icon_extension: &str) -> bool {
        SUPPORTED_ICON_EXTENSIONS
            .iter()
            .any(|supported_icon_extension| supported_icon_extension.eq_ignore_ascii_case(icon_extension))
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
                            let comm_process_name = Self::read_comm_process_name(process_id);
                            let process_name = cmdline_process_name
                                .clone()
                                .or_else(|| comm_process_name.clone())
                                .unwrap_or_default();
                            if process_name.is_empty() {
                                continue;
                            }

                            let package_name = cmdline_process_name
                                .as_deref()
                                .and_then(Self::extract_package_name);
                            let is_primary_package_process = Self::is_primary_package_process(cmdline_process_name.as_deref(), package_name.as_deref());
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
                                is_primary_package_process,
                            };
                            let is_zygote_process = Self::is_zygote_process_name(cmdline_process_name.as_deref())
                                || Self::is_zygote_process_name(comm_process_name.as_deref())
                                || Self::is_zygote_process_name(Some(process_name.as_str()));

                            all_processes.insert(process_id, process_info.clone());

                            if is_zygote_process {
                                zygote_processes.insert(process_id, process_info);
                            }
                        }
                    }
                }
            }
        }

        (all_processes, zygote_processes)
    }

    /// Determines whether a process has a zygote ancestor in its process tree.
    fn has_zygote_ancestor(
        process_id: u32,
        all_processes: &HashMap<u32, AndroidProcessInfo>,
        zygote_processes: &HashMap<u32, AndroidProcessInfo>,
    ) -> bool {
        let process_info = match all_processes.get(&process_id) {
            Some(process_info) => process_info,
            None => return false,
        };
        let mut current_parent_process_id = process_info.parent_process_id;
        let mut visited_processes = HashSet::new();

        while current_parent_process_id > 0 {
            if zygote_processes.contains_key(&current_parent_process_id) {
                return true;
            }

            if !visited_processes.insert(current_parent_process_id) {
                return false;
            }

            let parent_process_info = match all_processes.get(&current_parent_process_id) {
                Some(parent_process_info) => parent_process_info,
                None => return false,
            };
            current_parent_process_id = parent_process_info.parent_process_id;
        }

        false
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
        let package_paths = Self::parse_installed_package_paths();
        let (all_processes, zygote_processes) = Self::get_live_process_maps();
        let visible_windowed_packages = Self::query_visible_windowed_packages();
        let mut results = Vec::new();

        for android_process_info in all_processes.values() {
            let apk_path = android_process_info
                .package_name
                .as_deref()
                .and_then(|package_name| Self::get_apk_path(package_name, &package_paths));
            let is_windowed = apk_path.is_some()
                && Self::has_zygote_ancestor(android_process_info.process_id, &all_processes, &zygote_processes)
                && Self::is_user_app(android_process_info.process_id)
                && android_process_info.is_primary_package_process
                && (visible_windowed_packages.is_empty()
                    || android_process_info
                        .package_name
                        .as_ref()
                        .map(|package_name| visible_windowed_packages.contains(package_name))
                        .unwrap_or(false));

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
