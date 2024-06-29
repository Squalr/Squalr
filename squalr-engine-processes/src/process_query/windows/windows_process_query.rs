use sysinfo::{Pid, System};
use crate::process_query::IProcessQueryer;
use winapi::um::winuser::{GetWindowThreadProcessId, FindWindowA};
use winapi::um::shellapi::ExtractIconA;
use winapi::um::winuser::GetIconInfo;
use winapi::shared::windef::HICON;
use std::ptr::null_mut;
use std::ffi::CString;

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
    fn get_processes(&mut self) -> Vec<Pid> {
        self.system.refresh_all();
        self.system.processes().keys().cloned().collect()
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
                let handle = FindWindowA(null_mut(), process_name.as_ptr());
                if handle.is_null() {
                    return false;
                }
                let mut pid = 0;
                GetWindowThreadProcessId(handle, &mut pid);
                pid as u32 == process_id.as_u32()
            }
        } else {
            false
        }
    }

    fn get_icon(&self, process_id: &Pid) -> Option<Vec<u8>> {
        if let Some(process_name) = self.get_process_name(*process_id) {
            unsafe {
                let process_name = CString::new(process_name).expect("CString::new failed");
                let hicon: HICON = ExtractIconA(null_mut(), process_name.as_ptr(), 0);
                if hicon.is_null() {
                    return None;
                }
                let mut icon_info = std::mem::zeroed();
                if GetIconInfo(hicon, &mut icon_info) == 0 {
                    return None;
                }
                Some(vec![]) // Placeholder
            }
        } else {
            None
        }
    }
}
