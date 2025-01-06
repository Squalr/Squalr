use crate::process_info::ProcessIcon;
use crate::process_info::{Bitness, OpenedProcessInfo, ProcessInfo};
use crate::process_query::process_queryer::ProcessQueryOptions;
use crate::process_query::process_queryer::ProcessQueryer;
use crate::process_query::windows::windows_icon_handle::{DcHandle, IconHandle};
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};
use sysinfo::{Pid, System};
use windows_sys::Win32::Foundation::{CloseHandle, BOOL, HANDLE, HWND, LPARAM};
use windows_sys::Win32::Graphics::Gdi::{GetDC, GetDIBits, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS};
use windows_sys::Win32::System::ProcessStatus::K32GetModuleFileNameExW;
use windows_sys::Win32::System::Threading::{IsWow64Process, IsWow64Process2};
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows_sys::Win32::UI::Shell::ExtractIconW;
use windows_sys::Win32::UI::WindowsAndMessaging::{EnumWindows, IsWindowVisible};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetIconInfo, ICONINFO};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindowThreadProcessId, HICON};

pub struct WindowsProcessQuery {}

impl WindowsProcessQuery {
    fn get_process_bitness(handle: &HANDLE) -> Bitness {
        // Default to returning 64 bit.
        let result = Bitness::Bit64;

        unsafe {
            let mut is_wow64: BOOL = 0;
            let handle = handle.clone();

            if IsWow64Process(handle, &mut is_wow64) != 0 {
                if is_wow64 != 0 {
                    return Bitness::Bit32;
                } else {
                    // Use IsWow64Process2 if available (Windows 10 and above).
                    let mut process_machine: u16 = 0;
                    let mut native_machine: u16 = 0;

                    if IsWow64Process2(handle, &mut process_machine, &mut native_machine) != 0 {
                        if process_machine != 0 {
                            return Bitness::Bit32;
                        }
                    }
                }
            }
        }

        result
    }

    fn is_process_windowed(process_id: &Pid) -> bool {
        struct WindowFinder {
            pid: u32,
            found: AtomicBool,
        }

        unsafe extern "system" fn enum_window_callback(
            hwnd: HWND,
            lparam: LPARAM,
        ) -> BOOL {
            let finder = &*(lparam as *mut WindowFinder);
            let mut process_id: u32 = 0;
            GetWindowThreadProcessId(hwnd, &mut process_id);

            if process_id == finder.pid {
                // Only count the window if visible.
                if IsWindowVisible(hwnd) == BOOL::from(true) {
                    finder.found.store(true, Ordering::SeqCst);
                    // Stop enumeration.
                    BOOL::from(false)
                } else {
                    // Continue enumeration.
                    BOOL::from(true)
                }
            } else {
                // Continue enumeration.
                BOOL::from(true)
            }
        }

        let finder = WindowFinder {
            pid: process_id.as_u32(),
            found: AtomicBool::new(false),
        };

        unsafe {
            EnumWindows(Some(enum_window_callback), std::mem::transmute(&finder));
        }

        finder.found.load(Ordering::SeqCst)
    }

    fn get_icon(process_id: &Pid) -> Option<ProcessIcon> {
        unsafe {
            let process_handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, process_id.as_u32() as u32);

            if process_handle.is_null() {
                return None;
            }

            let mut buffer = [0u16; 260];
            let len = K32GetModuleFileNameExW(process_handle, std::ptr::null_mut(), buffer.as_mut_ptr(), buffer.len() as u32);

            if len == 0 {
                return None;
            }

            let icon_handle = IconHandle(ExtractIconW(std::ptr::null_mut(), buffer.as_ptr(), 0) as HICON);

            if (icon_handle.0).is_null() {
                return None;
            }

            let mut icon_info = std::mem::zeroed::<ICONINFO>();
            if GetIconInfo(icon_handle.0, &mut icon_info) == 0 {
                return None;
            }

            let dc = DcHandle(GetDC(std::ptr::null_mut()));
            let mut bmi = std::mem::zeroed::<BITMAPINFO>();
            bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;

            if GetDIBits(dc.0, icon_info.hbmColor, 0, 0, std::ptr::null_mut(), &mut bmi, DIB_RGB_COLORS) == 0 {
                return None;
            }

            let size = ((bmi.bmiHeader.biWidth * 4 + 3) & !3) * bmi.bmiHeader.biHeight.abs();
            let mut pixels = vec![0u8; size as usize];

            if GetDIBits(
                dc.0,
                icon_info.hbmColor,
                0,
                bmi.bmiHeader.biHeight.unsigned_abs(),
                pixels.as_mut_ptr() as *mut _,
                &mut bmi,
                DIB_RGB_COLORS,
            ) == 0
            {
                return None;
            }

            // Convert BGR to RGB while keeping alpha
            let width = bmi.bmiHeader.biWidth as u32;
            let height = bmi.bmiHeader.biHeight.unsigned_abs();
            let mut rgba = Vec::with_capacity((width * height * 4) as usize);

            for y in 0..height {
                for x in 0..width {
                    let i = ((y * width + x) * 4) as usize;
                    rgba.extend_from_slice(&[
                        pixels[i + 2], // B to R
                        pixels[i + 1], // G stays G
                        pixels[i],     // R to B
                        pixels[i + 3], // Alpha stays
                    ]);
                }
            }

            Some(ProcessIcon {
                bytes_rgba: rgba,
                width: width,
                height: height,
            })
        }
    }
}

impl ProcessQueryer for WindowsProcessQuery {
    fn open_process(process_info: &ProcessInfo) -> Result<OpenedProcessInfo, String> {
        unsafe {
            let handle: HANDLE = OpenProcess(PROCESS_ALL_ACCESS, 0, process_info.pid.as_u32());
            if handle == std::ptr::null_mut() {
                Err("Failed to open process".to_string())
            } else {
                let opened_process_info = OpenedProcessInfo {
                    pid: process_info.pid,
                    name: process_info.name.clone(),
                    bitness: Self::get_process_bitness(&handle),
                    handle: handle as u64,
                    icon: process_info.icon.clone(),
                };

                Ok(opened_process_info)
            }
        }
    }

    fn close_process(handle: u64) -> Result<(), String> {
        unsafe {
            if CloseHandle(handle as HANDLE) == 0 {
                Err("Failed to close process handle".to_string())
            } else {
                Ok(())
            }
        }
    }

    fn get_processes(
        options: ProcessQueryOptions,
        system: Arc<RwLock<System>>,
    ) -> Vec<ProcessInfo> {
        let system_guard = match system.read() {
            Ok(guard) => guard,
            Err(e) => {
                Logger::get_instance().log(LogLevel::Error, &format!("Failed to acquire system read lock: {}", e), None);
                return Vec::new();
            }
        };

        // Process and filter in a single pass.
        let filtered_processes: Vec<ProcessInfo> = system_guard
            .processes()
            .iter()
            .filter_map(|(pid, process)| {
                let mut matches = true;
                let process_info = ProcessInfo {
                    pid: *pid,
                    name: process.name().to_string_lossy().into_owned(),
                    is_windowed: Self::is_process_windowed(pid),
                    icon: if options.fetch_icons { Self::get_icon(pid) } else { None },
                };

                // Apply filters.
                if options.require_windowed {
                    matches &= process_info.is_windowed;
                }

                if let Some(ref term) = options.search_name {
                    if options.match_case {
                        matches &= process_info.name.contains(term);
                    } else {
                        matches &= process_info.name.to_lowercase().contains(&term.to_lowercase());
                    }
                }

                if let Some(required_pid) = options.required_pid {
                    matches &= process_info.pid == required_pid;
                }

                matches.then_some(process_info)
            })
            .take(options.limit.unwrap_or(usize::MAX as u64) as usize)
            .collect();

        filtered_processes
    }
}
