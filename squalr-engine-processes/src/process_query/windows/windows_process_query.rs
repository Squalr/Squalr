use crate::process_query::{IProcessQueryer, ProcessQueryOptions};

use std::ffi::CString;
use sysinfo::{Pid, System};
use windows_sys::Win32::UI::WindowsAndMessaging::{HICON, FindWindowA, GetWindowThreadProcessId};
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};
use windows_sys::Win32::Foundation::{HANDLE, CloseHandle};
use windows_sys::Win32::UI::Shell::ExtractIconA;
use windows_sys::Win32::UI::WindowsAndMessaging::{GetIconInfo, ICONINFO};

pub struct WindowsProcessQuery {
    system: System,
}

impl WindowsProcessQuery {
    pub fn new() -> Self {
        WindowsProcessQuery {
            system: System::new_all(),
        }
    }

    fn get_process_name(&self, process_id: Pid) -> Option<String> {
        self.system.process(process_id).map(|process| process.name().to_string())
    }
}

impl IProcessQueryer for WindowsProcessQuery {
    fn get_processes(&mut self, options: ProcessQueryOptions) -> Vec<Pid> {
        self.system.refresh_all();
        let mut processes: Vec<Pid> = self.system.processes().keys().cloned().collect();

        if let Some(limit) = options.limit {
            processes.truncate(limit);
        }

        processes.into_iter().filter(|pid| {
            let is_system = self.is_process_system_process(pid);
            if options.system_processes || !is_system {
                if let Some(name) = self.get_process_name(*pid) {
                    let mut matches = true;
                    if options.windowed {
                        matches &= self.is_process_windowed(pid);
                    }
                    if let Some(ref term) = options.search_term {
                        if options.match_case {
                            matches &= name.contains(term);
                        } else {
                            matches &= name.to_lowercase().contains(&term.to_lowercase());
                        }
                    }
                    matches
                } else {
                    false
                }
            } else {
                false
            }
        }).collect()
    }

    fn is_process_system_process(&self, process_id: &Pid) -> bool {
        process_id.as_u32() < 1000
    }

    fn get_process_name(&self, pid: Pid) -> Option<String> {
        self.system.process(pid).map(|process| process.name().to_string())
    }

    fn is_process_windowed(&self, process_id: &Pid) -> bool {
        if let Some(process_name) = self.get_process_name(*process_id) {
            unsafe {
                let process_name = CString::new(process_name).expect("CString::new failed");
                let handle = FindWindowA(std::ptr::null(), process_name.as_ptr() as *const u8);
                if handle == 0 {
                    return false;
                }
                let mut pid = 0;
                GetWindowThreadProcessId(handle, &mut pid);
                return pid as u32 == process_id.as_u32();
            }
        } else {
            return false;
        }
    }

    fn get_icon(&self, process_id: &Pid) -> Option<Vec<u8>> {
        if let Some(process_name) = self.get_process_name(*process_id) {
            unsafe {
                let process_name = CString::new(process_name).expect("CString::new failed");
                let hicon: HICON = ExtractIconA(0, process_name.as_ptr() as *const u8, 0);
                if hicon == 0 {
                    return None;
                }
                let mut icon_info: ICONINFO = std::mem::zeroed();
                if GetIconInfo(hicon, &mut icon_info) == 0 {
                    return None;
                }
                return Some(vec![]); // Placeholder
            }
        } else {
            return None;
        }
    }

    fn open_process(&self, process_id: &Pid) -> Result<u64, String> {
        unsafe {
            let handle: HANDLE = OpenProcess(PROCESS_ALL_ACCESS, 0, process_id.as_u32());
            if handle == 0 {
                return Err("Failed to open process".to_string())
            } else {
                return Ok(handle as u64);
            }
        }
    }
    
    fn close_process(&self, handle: u64) -> Result<(), String> {
        unsafe {
            if CloseHandle(handle as HANDLE) == 0 {
                Err("Failed to close process handle".to_string())
            } else {
                Ok(())
            }
        }
    }
}
