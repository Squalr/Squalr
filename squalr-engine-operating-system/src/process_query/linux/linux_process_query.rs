use crate::process_query::process_query_error::ProcessQueryError;
use crate::process_query::process_query_options::ProcessQueryOptions;
use crate::process_query::process_queryer::ProcessQueryer;
use squalr_engine_api::structures::memory::bitness::Bitness;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::processes::process_info::ProcessInfo;
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

    fn is_process_windowed(process_id: u32) -> bool {
        let process_environ_path = format!("/proc/{process_id}/environ");
        let environment_bytes = match fs::read(process_environ_path) {
            Ok(environment_bytes) => environment_bytes,
            Err(_) => return false,
        };

        environment_bytes
            .split(|byte_value| *byte_value == 0)
            .filter_map(|entry_bytes| std::str::from_utf8(entry_bytes).ok())
            .any(|entry_string| entry_string.starts_with("DISPLAY=") || entry_string.starts_with("WAYLAND_DISPLAY="))
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

                let process_is_windowed = Self::is_process_windowed(process_id_raw);

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
