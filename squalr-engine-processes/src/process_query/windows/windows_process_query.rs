use crate::process_info::ProcessIcon;
use crate::process_info::{Bitness, OpenedProcessInfo, ProcessInfo};
use crate::process_query::process_query_options::ProcessQueryOptions;
use crate::process_query::process_queryer::ProcessQueryer;
use crate::process_query::windows::windows_icon_handle::{DcHandle, IconHandle};
use crate::process_query::windows::windows_process_monitor::WindowsProcessMonitor;
use once_cell::sync::Lazy;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Mutex, RwLock};
use sysinfo::Pid;
use windows_sys::Win32::Foundation::{BOOL, CloseHandle, HANDLE, HWND, LPARAM};
use windows_sys::Win32::Graphics::Gdi::{BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, GetDC, GetDIBits};
use windows_sys::Win32::System::ProcessStatus::K32GetModuleFileNameExW;
use windows_sys::Win32::System::Threading::{IsWow64Process, IsWow64Process2};
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows_sys::Win32::UI::Shell::ExtractIconW;
use windows_sys::Win32::UI::WindowsAndMessaging::{EnumWindows, IsWindowVisible};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetIconInfo, ICONINFO};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindowThreadProcessId, HICON};

pub(crate) static PROCESS_MONITOR: Lazy<Mutex<WindowsProcessMonitor>> = Lazy::new(|| Mutex::new(WindowsProcessMonitor::new()));
static PROCESS_CACHE: Lazy<RwLock<HashMap<Pid, ProcessInfo>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub struct WindowsProcessQuery {}

impl WindowsProcessQuery {
    fn is_process_windowed(process_id: &Pid) -> bool {
        struct WindowFinder {
            process_id: u32,
            found: AtomicBool,
        }

        unsafe extern "system" fn enum_window_callback(
            hwnd: HWND,
            lparam: LPARAM,
        ) -> BOOL {
            let finder = unsafe { &*(lparam as *mut WindowFinder) };
            let mut process_id: u32 = 0;
            unsafe { GetWindowThreadProcessId(hwnd, &mut process_id) };

            if process_id == finder.process_id {
                // Only count the window if visible.
                if unsafe { IsWindowVisible(hwnd) } == BOOL::from(true) {
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
            process_id: process_id.as_u32(),
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

    fn update_cache(
        process_id: Pid,
        name: String,
        is_windowed: bool,
        icon: Option<ProcessIcon>,
    ) {
        if let Ok(mut cache) = PROCESS_CACHE.write() {
            cache.insert(process_id, ProcessInfo {
                process_id: process_id.as_u32(),
                name,
                is_windowed,
                icon,
            });
        }
    }

    fn get_from_cache(process_id: &Pid) -> Option<ProcessInfo> {
        PROCESS_CACHE
            .read()
            .ok()
            .and_then(|cache| cache.get(process_id).cloned())
    }
}

impl ProcessQueryer for WindowsProcessQuery {
    fn start_monitoring() -> Result<(), String> {
        let mut monitor = PROCESS_MONITOR
            .lock()
            .map_err(|err| format!("Failed to acquire process monitor lock: {}", err))?;

        monitor.start_monitoring();

        Ok(())
    }

    fn stop_monitoring() -> Result<(), String> {
        let mut monitor = PROCESS_MONITOR
            .lock()
            .map_err(|err| format!("Failed to acquire process monitor lock: {}", err))?;

        monitor.stop_monitoring();

        Ok(())
    }

    fn open_process(process_info: &ProcessInfo) -> Result<OpenedProcessInfo, String> {
        unsafe {
            let handle: HANDLE = OpenProcess(PROCESS_ALL_ACCESS, 0, process_info.process_id);
            if handle == std::ptr::null_mut() {
                Err("Failed to open process".to_string())
            } else {
                let opened_process_info = OpenedProcessInfo {
                    process_id: process_info.process_id,
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

    fn get_processes(options: ProcessQueryOptions) -> Vec<ProcessInfo> {
        let process_monitor_guard = match PROCESS_MONITOR.lock() {
            Ok(guard) => guard,
            Err(e) => {
                Logger::get_instance().log(LogLevel::Error, &format!("Failed to acquire process monitor lock: {}", e), None);
                return Vec::new();
            }
        };

        let system = process_monitor_guard.get_system();
        let system_guard = match system.read() {
            Ok(guard) => guard,
            Err(e) => {
                Logger::get_instance().log(LogLevel::Error, &format!("Failed to acquire system read lock: {}", e), None);
                return Vec::new();
            }
        };

        // Process and filter in a single pass, using cache when possible
        let filtered_processes: Vec<ProcessInfo> = system_guard
            .processes()
            .iter()
            .filter_map(|(process_id, process)| {
                // Try to get from cache first
                let process_info = if let Some(cached_info) = Self::get_from_cache(process_id) {
                    // If icons are required but not in cache, update the icon
                    if options.fetch_icons && cached_info.icon.is_none() {
                        let mut updated_info = cached_info.clone();
                        updated_info.icon = Self::get_icon(process_id);
                        // Update cache with new icon
                        Self::update_cache(*process_id, updated_info.name.clone(), updated_info.is_windowed, updated_info.icon.clone());
                        updated_info
                    } else {
                        cached_info
                    }
                } else {
                    // Create new ProcessInfo and cache it
                    let new_info = ProcessInfo {
                        process_id: process_id.as_u32(),
                        name: process.name().to_string_lossy().into_owned(),
                        is_windowed: Self::is_process_windowed(process_id),
                        icon: if options.fetch_icons { Self::get_icon(process_id) } else { None },
                    };
                    Self::update_cache(*process_id, new_info.name.clone(), new_info.is_windowed, new_info.icon.clone());
                    new_info
                };

                let mut matches = true;

                // Apply filters
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

                if let Some(required_process_id) = options.required_process_id {
                    matches &= process_info.process_id == required_process_id.as_u32();
                }

                matches.then_some(process_info)
            })
            .take(options.limit.unwrap_or(usize::MAX as u64) as usize)
            .collect();

        filtered_processes
    }
}
