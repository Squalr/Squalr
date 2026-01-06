use crate::process_query::{linux::linux_process_monitor::LinuxProcessMonitor, process_query_options::ProcessQueryOptions, process_queryer::ProcessQueryer};

use once_cell::sync::Lazy;

use squalr_engine_api::structures::memory::bitness::Bitness;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::processes::process_icon::ProcessIcon;
use squalr_engine_api::structures::processes::process_info::ProcessInfo;

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::sync::{Mutex, RwLock};

use sysinfo::Pid;

pub(crate) static PROCESS_MONITOR: Lazy<Mutex<LinuxProcessMonitor>> = Lazy::new(|| Mutex::new(LinuxProcessMonitor::new()));
static PROCESS_CACHE: Lazy<RwLock<HashMap<Pid, ProcessInfo>>> = Lazy::new(|| RwLock::new(HashMap::new()));
// static ICON_CACHE: Lazy<RwLock<HashMap<String, String>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub struct LinuxProcessQuery {}

impl LinuxProcessQuery {
    fn is_process_windowed(_process_id: &Pid) -> bool {
        // TODO: Windowing is managed by display servers like X11 or Wayland, not the kernel; and
        // there's no unified API call to determine if a process has a window or not.
        // ---
        // Naive approach:
        // Check mapped modules for X11/Wayland usage.
        true
    }

    fn get_icon(_process_id: &Pid) -> Option<ProcessIcon> {
        // TODO: Icons aren't embedded in a .rsrc section like PE executables, they're
        // conventionally installed to the filesystem, but may not be packaged at all.
        // ---
        // Naive approach:
        // Walk /usr/share/icons/hicolor/* and search for a file with a name matching
        // /proc/{pid}/comm
        //
        // By convention, icons are installed to:
        // /usr/share/icons/{theme}/{resolution}/{program_name}.{png,svg}
        //
        // hicolor is a "fallback" theme if not using a specific DE like GNOME or KDE, many
        // applications package icons here as not to assume a particular DE.
        None
    }

    fn update_cache(
        process_id: Pid,
        name: String,
        is_windowed: bool,
        icon: Option<ProcessIcon>,
    ) {
        if let Ok(mut cache) = PROCESS_CACHE.write() {
            cache.insert(process_id, ProcessInfo::new(process_id.as_u32(), name, is_windowed, icon));
        }
    }

    fn get_from_cache(process_id: &Pid) -> Option<ProcessInfo> {
        PROCESS_CACHE
            .read()
            .ok()
            .and_then(|cache| cache.get(process_id).cloned())
    }

    fn get_process_bitness(pid: u32) -> Bitness {
        let bitness = Bitness::Bit64;
        let elf_path = format!("/proc/{}/exe", pid);

        let mut elf_file = match File::open(&elf_path) {
            Err(_) => return bitness,
            Ok(f) => f,
        };

        // man 5 elf
        // ELF Class (0 = invalid, 1 = 32-bit, 2 = 64-bit)
        if let Err(_) = elf_file.seek(SeekFrom::Start(5)) {
            return bitness;
        };

        let mut buffer = [0u8; 1];

        if let Err(_) = elf_file.read(&mut buffer) {
            return bitness;
        };

        match buffer[0] {
            1 => Bitness::Bit32,
            2 => Bitness::Bit64,
            _ => bitness,
        }
    }
}

impl ProcessQueryer for LinuxProcessQuery {
    fn start_monitoring() -> Result<(), String> {
        let mut monitor = PROCESS_MONITOR
            .lock()
            .map_err(|error| format!("Failed to acquire process monitor lock: {}", error))?;

        monitor.start_monitoring();

        Ok(())
    }

    fn stop_monitoring() -> Result<(), String> {
        let mut monitor = PROCESS_MONITOR
            .lock()
            .map_err(|error| format!("Failed to acquire process monitor lock: {}", error))?;

        monitor.stop_monitoring();

        Ok(())
    }

    fn open_process(process_info: &ProcessInfo) -> Result<OpenedProcessInfo, String> {
        let pid = process_info.get_process_id_raw();

        Ok(OpenedProcessInfo::new(
            pid,
            process_info.get_name().to_string(),
            0, // No process handles in Linux, PID is sufficient
            Self::get_process_bitness(pid),
            process_info.get_icon().clone(),
        ))
    }

    fn close_process(_handle: u64) -> Result<(), String> {
        // NOP, no handles to release
        Ok(())
    }

    fn get_processes(process_query_options: ProcessQueryOptions) -> Vec<ProcessInfo> {
        let process_monitor_guard = match PROCESS_MONITOR.lock() {
            Ok(guard) => guard,
            Err(error) => {
                log::error!("Failed to acquire process monitor lock: {}", error);
                return Vec::new();
            }
        };

        let system = process_monitor_guard.get_system();
        let system_guard = match system.read() {
            Ok(guard) => guard,
            Err(error) => {
                log::error!("Failed to acquire system lock: {}", error);
                return Vec::new();
            }
        };

        let filtered_processes: Vec<ProcessInfo> = system_guard
            .processes()
            .iter()
            .filter_map(|(process_id, process)| {
                let process_info = if let Some(cached_info) = Self::get_from_cache(process_id) {
                    cached_info
                } else {
                    let new_info = ProcessInfo::new(
                        process_id.as_u32(),
                        // process.name().to_string_lossy().into_owned(),
                        format!("[{}] {}", process_id.as_u32(), process.name().to_string_lossy()),
                        Self::is_process_windowed(process_id),
                        Self::get_icon(process_id),
                    );

                    Self::update_cache(
                        *process_id,
                        new_info.get_name().to_string(),
                        new_info.get_is_windowed(),
                        new_info.get_icon().clone(),
                    );

                    new_info
                };

                let mut matches = true;

                if process_query_options.require_windowed {
                    matches &= process_info.get_is_windowed();
                }

                if let Some(ref term) = process_query_options.search_name {
                    if process_query_options.match_case {
                        matches &= process_info.get_name().contains(term);
                    } else {
                        matches &= process_info
                            .get_name()
                            .to_lowercase()
                            .contains(&term.to_lowercase());
                    }
                }

                if let Some(required_process_id) = process_query_options.required_process_id {
                    matches &= process_info.get_process_id_raw() == required_process_id.as_u32();
                }

                matches.then_some(process_info)
            })
            .take(process_query_options.limit.unwrap_or(u64::MAX) as usize)
            .collect();

        filtered_processes
    }
}
