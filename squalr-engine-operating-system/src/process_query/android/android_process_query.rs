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

#[cfg(test)]
mod tests {
    use super::AndroidProcessQuery;
    use crate::process_query::android::android_process_info::AndroidProcessInfo;
    use std::collections::HashMap;

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

    #[test]
    fn extract_package_name_from_apk_directory_parses_package_segment() {
        let package_name = AndroidProcessQuery::extract_package_name_from_apk_directory("com.squalr.android-ABCD1234==");

        assert_eq!(package_name.as_deref(), Some("com.squalr.android"));
    }

    #[test]
    fn extract_package_name_from_apk_directory_rejects_invalid_names() {
        let package_name = AndroidProcessQuery::extract_package_name_from_apk_directory("not_a_package_name");

        assert!(package_name.is_none());
    }

    #[test]
    fn primary_package_process_requires_exact_cmdline_package_match() {
        let is_primary_package_process = AndroidProcessQuery::is_primary_package_process(Some("com.squalr.android"), Some("com.squalr.android"));

        assert!(is_primary_package_process);
    }

    #[test]
    fn primary_package_process_rejects_colon_suffixed_processes() {
        let is_primary_package_process = AndroidProcessQuery::is_primary_package_process(Some("com.squalr.android:worker"), Some("com.squalr.android"));

        assert!(!is_primary_package_process);
    }

    #[test]
    fn package_manager_line_parser_ignores_malformed_lines() {
        let package_line = "package:/data/app/~~token/com.squalr.android-ABC==/base.apk=com.squalr.android";
        let malformed_line = "package:missing_equals_separator";

        let mut package_map = std::collections::HashMap::new();
        for package_list_line in [package_line, malformed_line] {
            if let Some((package_name, apk_path)) = AndroidProcessQuery::parse_package_manager_entry(package_list_line) {
                package_map.insert(package_name, apk_path);
            }
        }

        assert_eq!(
            package_map.get("com.squalr.android"),
            Some(&"/data/app/~~token/com.squalr.android-ABC==/base.apk".to_string())
        );
        assert_eq!(package_map.len(), 1);
    }

    #[test]
    fn zygote_process_name_detection_accepts_known_variants() {
        assert!(AndroidProcessQuery::is_zygote_process_name(Some("zygote")));
        assert!(AndroidProcessQuery::is_zygote_process_name(Some("zygote64")));
        assert!(AndroidProcessQuery::is_zygote_process_name(Some("app_zygote")));
        assert!(AndroidProcessQuery::is_zygote_process_name(Some("app_zygote64")));
        assert!(AndroidProcessQuery::is_zygote_process_name(Some("webview_zygote")));
    }

    #[test]
    fn zygote_process_name_detection_accepts_path_prefixed_names() {
        assert!(AndroidProcessQuery::is_zygote_process_name(Some("/system/bin/zygote64")));
    }

    #[test]
    fn zygote_process_name_detection_rejects_non_zygote_names() {
        assert!(!AndroidProcessQuery::is_zygote_process_name(Some("system_server")));
        assert!(!AndroidProcessQuery::is_zygote_process_name(Some("com.squalr.android")));
    }

    #[test]
    fn zygote_ancestor_walk_detects_indirect_lineage() {
        let mut all_processes = HashMap::new();
        all_processes.insert(
            1,
            AndroidProcessInfo {
                process_id: 1,
                parent_process_id: 0,
                process_name: "init".to_string(),
                package_name: None,
                is_primary_package_process: false,
            },
        );
        all_processes.insert(
            100,
            AndroidProcessInfo {
                process_id: 100,
                parent_process_id: 1,
                process_name: "zygote64".to_string(),
                package_name: None,
                is_primary_package_process: false,
            },
        );
        all_processes.insert(
            200,
            AndroidProcessInfo {
                process_id: 200,
                parent_process_id: 100,
                process_name: "usap64".to_string(),
                package_name: None,
                is_primary_package_process: false,
            },
        );
        all_processes.insert(
            300,
            AndroidProcessInfo {
                process_id: 300,
                parent_process_id: 200,
                process_name: "com.squalr.android".to_string(),
                package_name: Some("com.squalr.android".to_string()),
                is_primary_package_process: true,
            },
        );

        let mut zygote_processes = HashMap::new();
        zygote_processes.insert(
            100,
            AndroidProcessInfo {
                process_id: 100,
                parent_process_id: 1,
                process_name: "zygote64".to_string(),
                package_name: None,
                is_primary_package_process: false,
            },
        );

        assert!(AndroidProcessQuery::has_zygote_ancestor(300, &all_processes, &zygote_processes));
    }

    #[test]
    fn zygote_ancestor_walk_handles_parent_cycles() {
        let mut all_processes = HashMap::new();
        all_processes.insert(
            700,
            AndroidProcessInfo {
                process_id: 700,
                parent_process_id: 800,
                process_name: "com.squalr.android".to_string(),
                package_name: Some("com.squalr.android".to_string()),
                is_primary_package_process: true,
            },
        );
        all_processes.insert(
            800,
            AndroidProcessInfo {
                process_id: 800,
                parent_process_id: 700,
                process_name: "loop-parent".to_string(),
                package_name: None,
                is_primary_package_process: false,
            },
        );

        let zygote_processes = HashMap::new();

        assert!(!AndroidProcessQuery::has_zygote_ancestor(700, &all_processes, &zygote_processes));
    }

    #[test]
    fn parse_visible_windowed_packages_from_dumpsys_output_extracts_activity_packages() {
        let dumpsys_output = r#"
            mCurrentFocus=Window{43a9d73 u0 com.google.android.youtube/com.google.android.apps.youtube.app.WatchWhileActivity}
            mFocusedApp=ActivityRecord{f5238f4 u0 com.android.vending/com.google.android.finsky.activities.MainActivity t312}
            topResumedActivity=ActivityRecord{2d99f4f u0 com.google.android.calendar/com.android.calendar.homepage.AllInOneActivity t103}
            ResumedActivity: ActivityRecord{34fda9e u0 com.google.android.apps.photos/com.google.android.apps.photos.home.HomeActivity t301}
        "#;

        let visible_windowed_packages = AndroidProcessQuery::parse_visible_windowed_packages_from_dumpsys_output(dumpsys_output);

        assert!(visible_windowed_packages.contains("com.google.android.youtube"));
        assert!(visible_windowed_packages.contains("com.android.vending"));
        assert!(visible_windowed_packages.contains("com.google.android.calendar"));
        assert!(visible_windowed_packages.contains("com.google.android.apps.photos"));
    }

    #[test]
    fn parse_visible_windowed_packages_from_dumpsys_output_ignores_lines_without_package_activity_pairs() {
        let dumpsys_output = r#"
            Window #0 Window{32f44d u0 NotificationShade}
            no package separator here
            another unrelated line
        "#;

        let visible_windowed_packages = AndroidProcessQuery::parse_visible_windowed_packages_from_dumpsys_output(dumpsys_output);

        assert!(visible_windowed_packages.is_empty());
    }
}
