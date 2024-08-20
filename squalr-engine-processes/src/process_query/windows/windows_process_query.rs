use crate::process_info::{Bitness, ProcessInfo};
use crate::process_query::{IProcessQueryer, ProcessQueryOptions};
use std::ffi::CString;
use sysinfo::{Pid, System};
use windows_sys::Win32::UI::WindowsAndMessaging::{HICON, FindWindowA, GetWindowThreadProcessId};
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};
use windows_sys::Win32::Foundation::{HANDLE, CloseHandle, BOOL};
use windows_sys::Win32::UI::Shell::ExtractIconA;
use windows_sys::Win32::UI::WindowsAndMessaging::{GetIconInfo, ICONINFO};
use windows_sys::Win32::System::Threading::{IsWow64Process, IsWow64Process2};

pub struct WindowsProcessQuery {
    system: System,
}

impl WindowsProcessQuery {
    pub fn new(
    ) -> Self {
        WindowsProcessQuery {
            system: System::new_all(),
        }
    }

    fn get_process_name(
        &self,
        process_id: Pid,
    ) -> Option<String> {
        self.system.process(process_id).map(|process| process.name().to_str().unwrap_or_default().to_string())
    }

    fn get_process_bitness(
        &self,
        handle: HANDLE,
    ) -> Bitness {
        unsafe {
            let mut is_wow64: BOOL = 0;
            if IsWow64Process(handle, &mut is_wow64) != 0 {
                if is_wow64 != 0 {
                    return Bitness::Bit32;
                } else {
                    // Use IsWow64Process2 if available (Windows 10 and above)
                    let mut process_machine: u16 = 0;
                    let mut native_machine: u16 = 0;
                    if IsWow64Process2(handle, &mut process_machine, &mut native_machine) != 0 {
                        if process_machine == 0 {
                            return Bitness::Bit64;
                        } else {
                            return Bitness::Bit32;
                        }
                    }
                    return Bitness::Bit64;
                }
            }

            // Default to 64-bit if check fails
            return Bitness::Bit64;
        }
    }
}

impl IProcessQueryer for WindowsProcessQuery {
    fn get_processes(
        &mut self,
        options: ProcessQueryOptions,
    ) -> Vec<Pid> {
        self.system.refresh_all();
        let mut processes: Vec<Pid> = self.system.processes().keys().cloned().collect();

        if let Some(limit) = options.limit {
            processes.truncate(limit as usize);
        }

        let processes: Vec<Pid> = self.system.processes()
            .keys()
            .cloned()
            .filter(|pid| {
                if let Some(name) = self.get_process_name(*pid) {
                    let mut matches = true;
                    if options.require_windowed {
                        matches &= self.is_process_windowed(pid);
                    }
                    if let Some(ref term) = options.search_name {
                        if options.match_case {
                            matches &= name.contains(term);
                        } else {
                            matches &= name.to_lowercase().contains(&term.to_lowercase());
                        }
                    }
                    return matches;
                } else {
                    return false;
                }
            })
            .collect();

        // Limit the result after filtering
        if let Some(limit) = options.limit {
            return processes.into_iter().take(limit as usize).collect();
        } else {
            return processes;
        }
    }

    fn get_process_name(
        &self,
        pid: Pid,
    ) -> Option<String> {
        self.system.process(pid).map(|process| process.name().to_str().unwrap_or_default().to_string())
    }

    fn is_process_windowed(
        &self,
        process_id: &Pid,
    ) -> bool {
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

    fn get_icon(
        &self,
        process_id: &Pid,
    ) -> Option<Vec<u8>> {
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

    fn open_process(
        &self,
        process_id: &Pid,
    ) -> Result<ProcessInfo, String> {
        unsafe {
            let handle: HANDLE = OpenProcess(PROCESS_ALL_ACCESS, 0, process_id.as_u32());
            if handle == 0 {
                return Err("Failed to open process".to_string())
            } else {
                let bitness = self.get_process_bitness(handle);
                let process_info = ProcessInfo { pid: *process_id, handle: handle as u64, bitness };
                return Ok(process_info);
            }
        }
    }
    
    fn close_process(
        &self,
        handle: u64,
    ) -> Result<(), String> {
        unsafe {
            if CloseHandle(handle as HANDLE) == 0 {
                Err("Failed to close process handle".to_string())
            } else {
                Ok(())
            }
        }
    }
}
