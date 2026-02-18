use crate::process_query::process_query_error::ProcessQueryError;
use crate::process_query::process_query_options::ProcessQueryOptions;
use crate::process_query::process_queryer::ProcessQueryer;
use squalr_engine_api::structures::memory::bitness::Bitness;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::processes::process_info::ProcessInfo;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use sysinfo::{ProcessesToUpdate, System};

pub struct LinuxProcessQuery;

impl LinuxProcessQuery {
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

                Some(ProcessInfo::new(process_id_raw, process_name, process_is_windowed, None))
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
