use crate::process_query::process_query_error::ProcessQueryError;
use crate::process_query::process_query_options::ProcessQueryOptions;
use crate::process_query::process_queryer::ProcessQueryer;
use crate::process_query::windows::windows_icon_handle::{DcHandle, IconHandle};
use squalr_engine_api::structures::memory::bitness::Bitness;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::processes::process_icon::ProcessIcon;
use squalr_engine_api::structures::processes::process_info::ProcessInfo;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use sysinfo::{Pid, ProcessesToUpdate, System};
use windows_sys::Win32::Foundation::{CloseHandle, FALSE, HANDLE, HWND, LPARAM, TRUE};
use windows_sys::Win32::Graphics::Gdi::{BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, GetDC, GetDIBits};
use windows_sys::Win32::System::ProcessStatus::K32GetModuleFileNameExW;
use windows_sys::Win32::System::Threading::{IsWow64Process, IsWow64Process2};
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows_sys::Win32::UI::Shell::ExtractIconW;
use windows_sys::Win32::UI::WindowsAndMessaging::{EnumWindows, IsWindowVisible};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetIconInfo, ICONINFO};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindowThreadProcessId, HICON};
use windows_sys::core::BOOL;

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
                if unsafe { IsWindowVisible(hwnd) } == TRUE {
                    finder.found.store(true, Ordering::SeqCst);
                    // Stop enumeration.
                    FALSE
                } else {
                    // Continue enumeration.
                    TRUE
                }
            } else {
                // Continue enumeration.
                TRUE
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

            // Convert BGR to RGB while keeping alpha.
            let width = bmi.bmiHeader.biWidth as usize;
            let height = bmi.bmiHeader.biHeight.unsigned_abs() as usize;
            let stride = ((width * 4 + 3) & !3) as usize; // stride aligned to 4 bytes

            let mut rgba = Vec::with_capacity(width * height * 4);

            // Iterate rows bottom-to-top, because windows stores these stupidly so we need to flip the image.
            for y in (0..height).rev() {
                for x in 0..width {
                    let index = y * stride + x * 4;
                    rgba.extend_from_slice(&[
                        pixels[index + 2], // B -> R
                        pixels[index + 1], // G stays G
                        pixels[index],     // R -> B
                        pixels[index + 3], // Alpha stays alpha
                    ]);
                }
            }

            Some(ProcessIcon::new(rgba, width as u32, height as u32))
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
}

impl ProcessQueryer for WindowsProcessQuery {
    fn start_monitoring() -> Result<(), ProcessQueryError> {
        // Windows process query now exposes immediate operations only.
        Ok(())
    }

    fn stop_monitoring() -> Result<(), ProcessQueryError> {
        // Windows process query now exposes immediate operations only.
        Ok(())
    }

    fn open_process(process_info: &ProcessInfo) -> Result<OpenedProcessInfo, ProcessQueryError> {
        unsafe {
            let handle: HANDLE = OpenProcess(PROCESS_ALL_ACCESS, 0, process_info.get_process_id_raw());
            if handle == std::ptr::null_mut() {
                Err(ProcessQueryError::OpenProcessFailed {
                    process_id: process_info.get_process_id_raw(),
                })
            } else {
                let opened_process_info = OpenedProcessInfo::new(
                    process_info.get_process_id_raw(),
                    process_info.get_name().to_string(),
                    handle as u64,
                    Self::get_process_bitness(&handle),
                    process_info.get_icon().clone(),
                );

                Ok(opened_process_info)
            }
        }
    }

    fn close_process(handle: u64) -> Result<(), ProcessQueryError> {
        unsafe {
            if CloseHandle(handle as HANDLE) == 0 {
                Err(ProcessQueryError::CloseProcessFailed { handle })
            } else {
                Ok(())
            }
        }
    }

    fn get_processes(process_query_options: ProcessQueryOptions) -> Vec<ProcessInfo> {
        let mut system = System::new_all();
        system.refresh_processes(ProcessesToUpdate::All, true);

        system
            .processes()
            .iter()
            .filter_map(|(process_id, process)| {
                let process_name = process.name().to_string_lossy().into_owned();

                if let Some(required_process_id) = process_query_options.required_process_id {
                    if process_id.as_u32() != required_process_id.as_u32() {
                        return None;
                    }
                }

                if let Some(ref term) = process_query_options.search_name {
                    if process_query_options.match_case {
                        if !process_name.contains(term) {
                            return None;
                        }
                    } else {
                        let process_name_lowercase = process_name.to_lowercase();
                        let term_lowercase = term.to_lowercase();
                        if !process_name_lowercase.contains(&term_lowercase) {
                            return None;
                        }
                    }
                }

                let process_is_windowed = Self::is_process_windowed(process_id);

                if process_query_options.require_windowed && !process_is_windowed {
                    return None;
                }

                let process_icon = if process_query_options.fetch_icons {
                    Self::get_icon(process_id)
                } else {
                    None
                };

                Some(ProcessInfo::new(process_id.as_u32(), process_name, process_is_windowed, process_icon))
            })
            .take(process_query_options.limit.unwrap_or(usize::MAX as u64) as usize)
            .collect()
    }
}
