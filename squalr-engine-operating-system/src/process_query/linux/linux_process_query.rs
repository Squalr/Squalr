use crate::process_query::process_query_error::ProcessQueryError;
use crate::process_query::process_query_options::ProcessQueryOptions;
use crate::process_query::process_queryer::ProcessQueryer;
use image::ImageReader;
use squalr_engine_api::structures::memory::bitness::Bitness;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::processes::process_icon::ProcessIcon;
use squalr_engine_api::structures::processes::process_info::ProcessInfo;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use sysinfo::{ProcessesToUpdate, System};

pub struct LinuxProcessQuery;

#[derive(Clone)]
struct LinuxDesktopEntry {
    executable_name: String,
    icon_name: String,
}

impl LinuxProcessQuery {
    const DESKTOP_ENTRY_DIRECTORY_SUFFIX: &'static str = ".local/share/applications";
    const ICON_DIRECTORY_SUFFIX: &'static str = ".local/share/icons";
    const USER_ICON_FALLBACK_DIRECTORY_SUFFIX: &'static str = ".icons";
    const SUPPORTED_ICON_EXTENSIONS: [&'static str; 6] = ["png", "xpm", "ico", "jpg", "jpeg", "bmp"];
    const SHARED_DESKTOP_ENTRY_DIRECTORIES: [&'static str; 3] = [
        "/usr/share/applications",
        "/usr/local/share/applications",
        "/var/lib/flatpak/exports/share/applications",
    ];
    const SHARED_ICON_DIRECTORIES: [&'static str; 2] = ["/usr/share/icons", "/usr/share/pixmaps"];

    fn build_process_executable_path(process_id: u32) -> PathBuf {
        PathBuf::from(format!("/proc/{process_id}/exe"))
    }

    fn get_process_bitness(process_id: u32) -> Bitness {
        let process_executable_path = Self::build_process_executable_path(process_id);
        let executable_bytes = match fs::read(process_executable_path) {
            Ok(executable_bytes) => executable_bytes,
            Err(_) => return Bitness::Bit64,
        };

        Self::parse_elf_bitness_from_bytes(&executable_bytes).unwrap_or(Bitness::Bit64)
    }

    fn build_process_fd_directory_path(process_id: u32) -> PathBuf {
        PathBuf::from(format!("/proc/{process_id}/fd"))
    }

    fn get_home_directory() -> Option<PathBuf> {
        std::env::var_os("HOME").map(PathBuf::from)
    }

    fn get_desktop_entry_directories() -> Vec<PathBuf> {
        let mut desktop_entry_directories = Vec::new();

        if let Some(home_directory_path) = Self::get_home_directory() {
            desktop_entry_directories.push(home_directory_path.join(Self::DESKTOP_ENTRY_DIRECTORY_SUFFIX));
        }

        for shared_directory_path in Self::SHARED_DESKTOP_ENTRY_DIRECTORIES {
            desktop_entry_directories.push(PathBuf::from(shared_directory_path));
        }

        desktop_entry_directories
    }

    fn get_icon_search_directories() -> Vec<PathBuf> {
        let mut icon_search_directories = Vec::new();

        if let Some(home_directory_path) = Self::get_home_directory() {
            icon_search_directories.push(home_directory_path.join(Self::ICON_DIRECTORY_SUFFIX));
            icon_search_directories.push(home_directory_path.join(Self::USER_ICON_FALLBACK_DIRECTORY_SUFFIX));
        }

        for shared_directory_path in Self::SHARED_ICON_DIRECTORIES {
            icon_search_directories.push(PathBuf::from(shared_directory_path));
        }

        icon_search_directories
    }

    fn parse_elf_bitness_from_bytes(executable_bytes: &[u8]) -> Option<Bitness> {
        const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
        const ELF_CLASS_OFFSET: usize = 4;
        const ELF_CLASS_32_BIT: u8 = 1;
        const ELF_CLASS_64_BIT: u8 = 2;

        if executable_bytes.len() <= ELF_CLASS_OFFSET {
            return None;
        }

        if executable_bytes.get(0..4)? != ELF_MAGIC {
            return None;
        }

        match executable_bytes[ELF_CLASS_OFFSET] {
            ELF_CLASS_32_BIT => Some(Bitness::Bit32),
            ELF_CLASS_64_BIT => Some(Bitness::Bit64),
            _ => None,
        }
    }

    fn parse_socket_inode_from_fd_target(fd_target: &str) -> Option<u64> {
        let socket_prefix = "socket:[";
        let socket_suffix = "]";

        if !(fd_target.starts_with(socket_prefix) && fd_target.ends_with(socket_suffix)) {
            return None;
        }

        let inode_start_index = socket_prefix.len();
        let inode_end_index = fd_target.len() - socket_suffix.len();
        let inode_value_string = &fd_target[inode_start_index..inode_end_index];

        inode_value_string.parse::<u64>().ok()
    }

    fn is_display_server_socket_path(socket_path: &str) -> bool {
        socket_path.contains("/wayland-") || socket_path.contains("/tmp/.X11-unix/")
    }

    fn parse_display_server_socket_inodes(proc_net_unix_contents: &str) -> HashSet<u64> {
        let mut display_server_socket_inodes = HashSet::new();

        for proc_net_unix_line in proc_net_unix_contents.lines() {
            let column_values: Vec<&str> = proc_net_unix_line.split_whitespace().collect();

            if column_values.len() < 8 {
                continue;
            }

            let socket_path = column_values[7];

            if !Self::is_display_server_socket_path(socket_path) {
                continue;
            }

            if let Ok(socket_inode) = column_values[6].parse::<u64>() {
                display_server_socket_inodes.insert(socket_inode);
            }
        }

        display_server_socket_inodes
    }

    fn collect_display_server_socket_inodes() -> HashSet<u64> {
        let proc_net_unix_contents = match fs::read_to_string("/proc/net/unix") {
            Ok(proc_net_unix_contents) => proc_net_unix_contents,
            Err(_) => return HashSet::new(),
        };

        Self::parse_display_server_socket_inodes(&proc_net_unix_contents)
    }

    fn is_process_windowed(
        process_id: u32,
        display_server_socket_inodes: &HashSet<u64>,
    ) -> bool {
        if display_server_socket_inodes.is_empty() {
            return false;
        }

        let process_fd_directory_path = Self::build_process_fd_directory_path(process_id);
        let file_descriptor_entries = match fs::read_dir(process_fd_directory_path) {
            Ok(file_descriptor_entries) => file_descriptor_entries,
            Err(_) => return false,
        };

        for file_descriptor_entry_result in file_descriptor_entries {
            let file_descriptor_entry = match file_descriptor_entry_result {
                Ok(file_descriptor_entry) => file_descriptor_entry,
                Err(_) => continue,
            };

            let file_descriptor_target_path = match fs::read_link(file_descriptor_entry.path()) {
                Ok(file_descriptor_target_path) => file_descriptor_target_path,
                Err(_) => continue,
            };

            let file_descriptor_target_string = file_descriptor_target_path.to_string_lossy();

            if let Some(socket_inode) = Self::parse_socket_inode_from_fd_target(&file_descriptor_target_string) {
                if display_server_socket_inodes.contains(&socket_inode) {
                    return true;
                }
            }
        }

        false
    }

    fn process_name_matches_filter(
        process_name: &str,
        process_query_options: &ProcessQueryOptions,
    ) -> bool {
        if let Some(ref search_term) = process_query_options.search_name {
            if process_query_options.match_case {
                process_name.contains(search_term)
            } else {
                process_name
                    .to_ascii_lowercase()
                    .contains(&search_term.to_ascii_lowercase())
            }
        } else {
            true
        }
    }

    fn split_shell_like_arguments(command_line: &str) -> Vec<String> {
        let mut argument_tokens = Vec::new();
        let mut current_argument = String::new();
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut is_escaping_character = false;

        for command_character in command_line.chars() {
            if is_escaping_character {
                current_argument.push(command_character);
                is_escaping_character = false;
                continue;
            }

            if command_character == '\\' {
                is_escaping_character = true;
                continue;
            }

            if command_character == '\'' && !in_double_quote {
                in_single_quote = !in_single_quote;
                continue;
            }

            if command_character == '"' && !in_single_quote {
                in_double_quote = !in_double_quote;
                continue;
            }

            if command_character.is_whitespace() && !in_single_quote && !in_double_quote {
                if !current_argument.is_empty() {
                    argument_tokens.push(current_argument.clone());
                    current_argument.clear();
                }
                continue;
            }

            current_argument.push(command_character);
        }

        if !current_argument.is_empty() {
            argument_tokens.push(current_argument);
        }

        argument_tokens
    }

    fn parse_executable_name_from_exec_value(exec_value: &str) -> Option<String> {
        let argument_tokens = Self::split_shell_like_arguments(exec_value.trim());
        if argument_tokens.is_empty() {
            return None;
        }

        let mut command_token_index = 0;

        if argument_tokens
            .first()
            .map(|command_token| command_token == "env")
            .unwrap_or(false)
        {
            command_token_index += 1;

            while command_token_index < argument_tokens.len()
                && argument_tokens[command_token_index].contains('=')
                && !argument_tokens[command_token_index].starts_with('/')
            {
                command_token_index += 1;
            }
        }

        let command_token = argument_tokens.get(command_token_index)?;
        let command_path = Path::new(command_token);
        let executable_file_name = command_path.file_name()?.to_string_lossy().to_ascii_lowercase();

        Some(executable_file_name)
    }

    fn parse_desktop_entry(desktop_entry_contents: &str) -> Option<LinuxDesktopEntry> {
        let mut found_desktop_entry_section = false;
        let mut executable_name: Option<String> = None;
        let mut icon_name: Option<String> = None;
        let mut is_hidden_from_ui = false;

        for desktop_entry_line in desktop_entry_contents.lines() {
            let trimmed_line = desktop_entry_line.trim();

            if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
                continue;
            }

            if trimmed_line.starts_with('[') && trimmed_line.ends_with(']') {
                found_desktop_entry_section = trimmed_line == "[Desktop Entry]";
                continue;
            }

            if !found_desktop_entry_section {
                continue;
            }

            let Some((desktop_key, desktop_value)) = trimmed_line.split_once('=') else {
                continue;
            };

            let desktop_value = desktop_value.trim();

            match desktop_key.trim() {
                "Exec" => executable_name = Self::parse_executable_name_from_exec_value(desktop_value),
                "Icon" => {
                    if !desktop_value.is_empty() {
                        icon_name = Some(desktop_value.to_string());
                    }
                }
                "NoDisplay" => {
                    is_hidden_from_ui = desktop_value.eq_ignore_ascii_case("true");
                }
                _ => {}
            }
        }

        if is_hidden_from_ui {
            return None;
        }

        Some(LinuxDesktopEntry {
            executable_name: executable_name?,
            icon_name: icon_name?,
        })
    }

    fn collect_desktop_entries() -> Vec<LinuxDesktopEntry> {
        let mut desktop_entries = Vec::new();

        for desktop_entry_directory_path in Self::get_desktop_entry_directories() {
            let directory_entries = match fs::read_dir(desktop_entry_directory_path) {
                Ok(directory_entries) => directory_entries,
                Err(_) => continue,
            };

            for directory_entry_result in directory_entries {
                let directory_entry = match directory_entry_result {
                    Ok(directory_entry) => directory_entry,
                    Err(_) => continue,
                };

                let desktop_entry_path = directory_entry.path();
                let has_desktop_extension = desktop_entry_path
                    .extension()
                    .and_then(|extension_value| extension_value.to_str())
                    .map(|extension_value| extension_value.eq_ignore_ascii_case("desktop"))
                    .unwrap_or(false);

                if !has_desktop_extension {
                    continue;
                }

                let desktop_entry_contents = match fs::read_to_string(&desktop_entry_path) {
                    Ok(desktop_entry_contents) => desktop_entry_contents,
                    Err(_) => continue,
                };

                if let Some(parsed_desktop_entry) = Self::parse_desktop_entry(&desktop_entry_contents) {
                    desktop_entries.push(parsed_desktop_entry);
                }
            }
        }

        desktop_entries
    }

    fn build_desktop_entry_icon_lookup() -> HashMap<String, String> {
        let mut desktop_entry_icon_lookup = HashMap::new();

        for desktop_entry in Self::collect_desktop_entries() {
            desktop_entry_icon_lookup
                .entry(desktop_entry.executable_name)
                .or_insert(desktop_entry.icon_name);
        }

        desktop_entry_icon_lookup
    }

    fn is_supported_icon_extension(icon_extension: &str) -> bool {
        Self::SUPPORTED_ICON_EXTENSIONS
            .iter()
            .any(|supported_extension| supported_extension.eq_ignore_ascii_case(icon_extension))
    }

    fn find_icon_file_in_directory(
        icon_directory_path: &Path,
        icon_file_stem: &str,
    ) -> Option<PathBuf> {
        let mut pending_directory_paths = vec![icon_directory_path.to_path_buf()];
        let icon_file_stem = icon_file_stem.to_ascii_lowercase();

        while let Some(next_directory_path) = pending_directory_paths.pop() {
            let directory_entries = match fs::read_dir(&next_directory_path) {
                Ok(directory_entries) => directory_entries,
                Err(_) => continue,
            };

            for directory_entry_result in directory_entries {
                let directory_entry = match directory_entry_result {
                    Ok(directory_entry) => directory_entry,
                    Err(_) => continue,
                };

                let entry_path = directory_entry.path();
                if entry_path.is_dir() {
                    pending_directory_paths.push(entry_path);
                    continue;
                }

                let Some(file_stem) = entry_path.file_stem().and_then(|file_stem| file_stem.to_str()) else {
                    continue;
                };

                let Some(icon_extension) = entry_path
                    .extension()
                    .and_then(|icon_extension| icon_extension.to_str())
                else {
                    continue;
                };

                if file_stem.eq_ignore_ascii_case(&icon_file_stem) && Self::is_supported_icon_extension(icon_extension) {
                    return Some(entry_path);
                }
            }
        }

        None
    }

    fn try_resolve_icon_path_with_supported_extensions(icon_path_without_extension: &Path) -> Option<PathBuf> {
        for supported_extension in Self::SUPPORTED_ICON_EXTENSIONS {
            let candidate_icon_path = icon_path_without_extension.with_extension(supported_extension);

            if candidate_icon_path.exists() {
                return Some(candidate_icon_path);
            }
        }

        None
    }

    fn resolve_icon_file_path(
        icon_name: &str,
        icon_search_directories: &[PathBuf],
    ) -> Option<PathBuf> {
        let icon_path = Path::new(icon_name);
        let icon_has_extension = icon_path.extension().is_some();
        let icon_name_for_recursive_search = if icon_has_extension {
            icon_path
                .file_stem()
                .and_then(|file_stem| file_stem.to_str())
                .unwrap_or(icon_name)
        } else {
            icon_name
        };

        if icon_path.is_absolute() {
            if icon_path.exists() {
                return Some(icon_path.to_path_buf());
            }

            if !icon_has_extension {
                return Self::try_resolve_icon_path_with_supported_extensions(icon_path);
            }
        }

        for icon_search_directory_path in icon_search_directories {
            let direct_icon_path = icon_search_directory_path.join(icon_name);

            if direct_icon_path.exists() {
                return Some(direct_icon_path);
            }

            if !icon_has_extension {
                if let Some(icon_with_extension_path) = Self::try_resolve_icon_path_with_supported_extensions(&direct_icon_path) {
                    return Some(icon_with_extension_path);
                }
            }
        }

        for icon_search_directory_path in icon_search_directories {
            if let Some(icon_path_match) = Self::find_icon_file_in_directory(icon_search_directory_path, icon_name_for_recursive_search) {
                return Some(icon_path_match);
            }
        }

        None
    }

    fn load_icon_from_path(icon_path: &Path) -> Option<ProcessIcon> {
        let icon_bytes = fs::read(icon_path).ok()?;
        let icon_reader = ImageReader::new(Cursor::new(icon_bytes))
            .with_guessed_format()
            .ok()?;
        let decoded_image = icon_reader.decode().ok()?;
        let rgba_image = decoded_image.to_rgba8();
        let (icon_width, icon_height) = rgba_image.dimensions();
        let icon_rgba_bytes = rgba_image.into_raw();

        Some(ProcessIcon::new(icon_rgba_bytes, icon_width, icon_height))
    }

    fn get_process_executable_name(process_id: u32) -> Option<String> {
        let process_executable_symlink_target = fs::read_link(Self::build_process_executable_path(process_id)).ok()?;
        let executable_file_name = process_executable_symlink_target
            .file_name()?
            .to_string_lossy()
            .to_string();
        let executable_file_name = executable_file_name
            .split(" (deleted)")
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();

        if executable_file_name.is_empty() {
            return None;
        }

        Some(executable_file_name)
    }

    fn get_icon_for_process(
        process_id: u32,
        desktop_entry_icon_lookup: &HashMap<String, String>,
        icon_search_directories: &[PathBuf],
        icon_cache: &mut HashMap<String, Option<ProcessIcon>>,
    ) -> Option<ProcessIcon> {
        let executable_file_name = Self::get_process_executable_name(process_id)?;
        let icon_name = desktop_entry_icon_lookup.get(&executable_file_name)?;

        if let Some(cached_icon) = icon_cache.get(icon_name) {
            return cached_icon.clone();
        }

        let icon_path = match Self::resolve_icon_file_path(icon_name, icon_search_directories) {
            Some(icon_path) => icon_path,
            None => {
                icon_cache.insert(icon_name.clone(), None);
                return None;
            }
        };

        let loaded_icon = Self::load_icon_from_path(&icon_path);
        icon_cache.insert(icon_name.clone(), loaded_icon.clone());

        loaded_icon
    }
}

impl ProcessQueryer for LinuxProcessQuery {
    fn start_monitoring() -> Result<(), ProcessQueryError> {
        // Linux process query currently exposes immediate operations only.
        Ok(())
    }

    fn stop_monitoring() -> Result<(), ProcessQueryError> {
        // Linux process query currently exposes immediate operations only.
        Ok(())
    }

    fn open_process(process_info: &ProcessInfo) -> Result<OpenedProcessInfo, ProcessQueryError> {
        let process_id = process_info.get_process_id_raw();

        Ok(OpenedProcessInfo::new(
            process_id,
            process_info.get_name().to_string(),
            process_id as u64,
            Self::get_process_bitness(process_id),
            process_info.get_icon().clone(),
        ))
    }

    fn close_process(_handle: u64) -> Result<(), ProcessQueryError> {
        Ok(())
    }

    fn get_processes(process_query_options: ProcessQueryOptions) -> Vec<ProcessInfo> {
        let mut system = System::new_all();
        system.refresh_processes(ProcessesToUpdate::All, true);
        let display_server_socket_inodes = Self::collect_display_server_socket_inodes();
        let desktop_entry_icon_lookup = if process_query_options.fetch_icons {
            Self::build_desktop_entry_icon_lookup()
        } else {
            HashMap::new()
        };
        let icon_search_directories = if process_query_options.fetch_icons {
            Self::get_icon_search_directories()
        } else {
            Vec::new()
        };
        let mut icon_cache: HashMap<String, Option<ProcessIcon>> = HashMap::new();

        system
            .processes()
            .iter()
            .filter_map(|(process_id, process)| {
                let process_id_raw = process_id.as_u32();

                if let Some(required_process_id) = process_query_options.required_process_id {
                    if process_id_raw != required_process_id.as_u32() {
                        return None;
                    }
                }

                let process_name = process.name().to_string_lossy().into_owned();

                if !Self::process_name_matches_filter(&process_name, &process_query_options) {
                    return None;
                }

                let process_is_windowed = Self::is_process_windowed(process_id_raw, &display_server_socket_inodes);

                if process_query_options.require_windowed && !process_is_windowed {
                    return None;
                }

                let process_icon = if process_query_options.fetch_icons {
                    Self::get_icon_for_process(process_id_raw, &desktop_entry_icon_lookup, &icon_search_directories, &mut icon_cache)
                } else {
                    None
                };

                Some(ProcessInfo::new(process_id_raw, process_name, process_is_windowed, process_icon))
            })
            .take(process_query_options.limit.unwrap_or(u64::MAX) as usize)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::LinuxProcessQuery;
    use crate::process_query::process_queryer::ProcessQueryer;
    use squalr_engine_api::structures::memory::bitness::Bitness;
    use squalr_engine_api::structures::processes::process_info::ProcessInfo;
    use std::collections::HashSet;

    #[test]
    fn parse_elf_bitness_reads_32_bit_headers() {
        let mut elf_header = vec![0x7F, b'E', b'L', b'F', 1];
        elf_header.extend_from_slice(&[0; 32]);

        assert_eq!(LinuxProcessQuery::parse_elf_bitness_from_bytes(&elf_header), Some(Bitness::Bit32));
    }

    #[test]
    fn parse_elf_bitness_reads_64_bit_headers() {
        let mut elf_header = vec![0x7F, b'E', b'L', b'F', 2];
        elf_header.extend_from_slice(&[0; 32]);

        assert_eq!(LinuxProcessQuery::parse_elf_bitness_from_bytes(&elf_header), Some(Bitness::Bit64));
    }

    #[test]
    fn parse_elf_bitness_rejects_invalid_headers() {
        assert_eq!(LinuxProcessQuery::parse_elf_bitness_from_bytes(&[0x7F, b'N', b'O', b'T', 2]), None);
        assert_eq!(LinuxProcessQuery::parse_elf_bitness_from_bytes(&[0x7F, b'E', b'L', b'F']), None);
    }

    #[test]
    fn parse_socket_inode_from_fd_target_extracts_inode() {
        assert_eq!(LinuxProcessQuery::parse_socket_inode_from_fd_target("socket:[123456]"), Some(123456));
        assert_eq!(LinuxProcessQuery::parse_socket_inode_from_fd_target("pipe:[123456]"), None);
        assert_eq!(LinuxProcessQuery::parse_socket_inode_from_fd_target("socket:[]"), None);
    }

    #[test]
    fn parse_display_server_socket_inodes_filters_wayland_and_x11_sockets() {
        let proc_net_unix_contents = "\
Num RefCount Protocol Flags Type St Inode Path
0000000000000000: 00000002 00000000 00010000 0001 01 11111 /run/user/1000/wayland-0
0000000000000000: 00000002 00000000 00010000 0001 01 22222 /tmp/.X11-unix/X0
0000000000000000: 00000002 00000000 00010000 0001 01 33333 /tmp/non-display-socket
";

        let display_server_socket_inodes = LinuxProcessQuery::parse_display_server_socket_inodes(proc_net_unix_contents);

        assert_eq!(display_server_socket_inodes, HashSet::from([11111_u64, 22222_u64]));
    }

    #[test]
    fn split_shell_like_arguments_handles_quoted_segments() {
        let argument_tokens = LinuxProcessQuery::split_shell_like_arguments("env GTK_THEME=dark \"/usr/bin/fire fox\" --new-window");

        assert_eq!(
            argument_tokens,
            vec![
                "env".to_string(),
                "GTK_THEME=dark".to_string(),
                "/usr/bin/fire fox".to_string(),
                "--new-window".to_string()
            ]
        );
    }

    #[test]
    fn parse_executable_name_from_exec_value_supports_env_prefix() {
        let executable_name =
            LinuxProcessQuery::parse_executable_name_from_exec_value("env BAMF_DESKTOP_FILE_HINT=/usr/share/applications/firefox.desktop /usr/bin/firefox %u");

        assert_eq!(executable_name, Some("firefox".to_string()));
    }

    #[test]
    fn parse_desktop_entry_extracts_exec_and_icon() {
        let desktop_entry_contents = "\
[Desktop Entry]
Type=Application
Name=Firefox
Exec=/usr/bin/firefox %u
Icon=firefox
";

        let parsed_desktop_entry = LinuxProcessQuery::parse_desktop_entry(desktop_entry_contents);

        assert!(parsed_desktop_entry.is_some());
        if let Some(parsed_desktop_entry) = parsed_desktop_entry {
            assert_eq!(parsed_desktop_entry.executable_name, "firefox");
            assert_eq!(parsed_desktop_entry.icon_name, "firefox");
        }
    }

    #[test]
    fn parse_desktop_entry_ignores_hidden_entries() {
        let desktop_entry_contents = "\
[Desktop Entry]
Type=Application
Exec=/usr/bin/hidden-app
Icon=hidden-app
NoDisplay=true
";

        let parsed_desktop_entry = LinuxProcessQuery::parse_desktop_entry(desktop_entry_contents);

        assert!(parsed_desktop_entry.is_none());
    }

    #[test]
    fn open_process_returns_opened_process_info_with_expected_fields() {
        let process_id = 1337;
        let process_name = "linux-target".to_string();
        let process_info = ProcessInfo::new(process_id, process_name.clone(), false, None);

        let opened_process_info = LinuxProcessQuery::open_process(&process_info).expect("linux open_process should return an opened process info object.");

        assert_eq!(opened_process_info.get_process_id_raw(), process_id);
        assert_eq!(opened_process_info.get_name(), process_name);
        assert_eq!(opened_process_info.get_handle(), process_id as u64);
        assert!(matches!(opened_process_info.get_bitness(), Bitness::Bit32 | Bitness::Bit64));
        assert!(opened_process_info.get_icon().is_none());
    }

    #[test]
    fn close_process_returns_ok_for_any_handle() {
        let close_result = LinuxProcessQuery::close_process(42);

        assert!(close_result.is_ok());
    }

    #[test]
    fn get_processes_with_required_process_id_returns_target_process() {
        let process_id = std::process::id();

        let processes = LinuxProcessQuery::get_processes(crate::process_query::process_query_options::ProcessQueryOptions {
            required_process_id: Some(sysinfo::Pid::from_u32(process_id)),
            search_name: None,
            require_windowed: false,
            match_case: false,
            fetch_icons: false,
            limit: Some(1),
        });

        assert_eq!(processes.len(), 1);
        assert_eq!(processes[0].get_process_id_raw(), process_id);
    }
}
