use crate::process_query::process_query_error::ProcessQueryError;
use crate::process_query::process_query_options::ProcessQueryOptions;
use crate::process_query::process_queryer::ProcessQueryer;
use crate::process_query::windows::windows_icon_handle::{BitmapHandle, DcHandle, IconHandle, MemoryDcHandle, ProcessHandle, SelectedGdiObjectGuard};
use squalr_engine_api::structures::memory::bitness::Bitness;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::processes::process_icon::ProcessIcon;
use squalr_engine_api::structures::processes::process_info::ProcessInfo;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use sysinfo::{Pid, ProcessesToUpdate, System};
use windows_sys::Win32::Foundation::{CloseHandle, FALSE, HANDLE, HWND, LPARAM, TRUE};
use windows_sys::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS, GetDC, HGDIOBJ, SelectObject,
};
use windows_sys::Win32::System::ProcessStatus::K32GetModuleFileNameExW;
use windows_sys::Win32::System::Threading::{IsWow64Process, IsWow64Process2};
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows_sys::Win32::UI::Shell::ExtractIconW;
use windows_sys::Win32::UI::WindowsAndMessaging::{DI_NORMAL, DrawIconEx, EnumWindows, GetSystemMetrics, IsWindowVisible};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindowThreadProcessId, SM_CXICON, SM_CYICON};
use windows_sys::core::BOOL;

pub struct WindowsProcessQuery {}

impl WindowsProcessQuery {
    const DEFAULT_ICON_DIMENSION_PIXELS: i32 = 32;
    const MAX_EXECUTABLE_PATH_LENGTH: usize = 32_768;

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
            EnumWindows(Some(enum_window_callback), (&finder as *const WindowFinder) as LPARAM);
        }

        finder.found.load(Ordering::SeqCst)
    }

    fn get_icon(process_id: &Pid) -> Option<ProcessIcon> {
        unsafe {
            let process_handle = ProcessHandle::new(OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, process_id.as_u32()))?;
            let executable_path_buffer = Self::resolve_process_executable_path(process_handle.0)?;
            let icon_handle = IconHandle::new(ExtractIconW(std::ptr::null_mut(), executable_path_buffer.as_ptr(), 0))?;

            Self::render_icon_to_rgba(&icon_handle)
        }
    }

    fn resolve_process_executable_path(process_handle: HANDLE) -> Option<Vec<u16>> {
        let mut executable_path_buffer_length = 260usize;

        loop {
            let mut executable_path_buffer = vec![0u16; executable_path_buffer_length];
            let executable_path_length = unsafe {
                K32GetModuleFileNameExW(
                    process_handle,
                    std::ptr::null_mut(),
                    executable_path_buffer.as_mut_ptr(),
                    executable_path_buffer.len() as u32,
                )
            };

            if executable_path_length == 0 {
                return None;
            }

            if executable_path_length as usize >= executable_path_buffer_length.saturating_sub(1) {
                executable_path_buffer_length = executable_path_buffer_length.saturating_mul(2);
                if executable_path_buffer_length > Self::MAX_EXECUTABLE_PATH_LENGTH {
                    log::warn!("Skipping process icon because the executable path exceeded the supported path length.");
                    return None;
                }

                continue;
            }

            executable_path_buffer[executable_path_length as usize] = 0;
            executable_path_buffer.truncate(executable_path_length as usize + 1);
            return Some(executable_path_buffer);
        }
    }

    fn render_icon_to_rgba(icon_handle: &IconHandle) -> Option<ProcessIcon> {
        let (icon_width, icon_height) = Self::resolve_icon_dimensions();
        let screen_device_context = unsafe { DcHandle(GetDC(std::ptr::null_mut())) };
        if screen_device_context.0.is_null() {
            return None;
        }

        let memory_device_context = unsafe { MemoryDcHandle(CreateCompatibleDC(screen_device_context.0)) };
        if memory_device_context.0.is_null() {
            return None;
        }

        let bitmap_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: icon_width,
                biHeight: -icon_height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                ..unsafe { std::mem::zeroed() }
            },
            ..unsafe { std::mem::zeroed() }
        };
        let mut raw_bitmap_bits = std::ptr::null_mut();
        let icon_bitmap = unsafe {
            BitmapHandle(CreateDIBSection(
                screen_device_context.0,
                &bitmap_info,
                DIB_RGB_COLORS,
                &mut raw_bitmap_bits,
                std::ptr::null_mut(),
                0,
            ))
        };
        if icon_bitmap.0.is_null() || raw_bitmap_bits.is_null() {
            return None;
        }

        let previous_bitmap = unsafe { SelectObject(memory_device_context.0, icon_bitmap.0 as HGDIOBJ) };
        if previous_bitmap.is_null() {
            return None;
        }
        let _selected_bitmap_guard = SelectedGdiObjectGuard::new(memory_device_context.0, previous_bitmap);

        let pixel_count = icon_width as usize * icon_height as usize;
        unsafe {
            std::ptr::write_bytes(raw_bitmap_bits, 0, pixel_count * 4);
        }

        let draw_result = unsafe {
            DrawIconEx(
                memory_device_context.0,
                0,
                0,
                icon_handle.0,
                icon_width,
                icon_height,
                0,
                std::ptr::null_mut(),
                DI_NORMAL,
            )
        };
        if draw_result == 0 {
            return None;
        }

        let bgra_pixels = unsafe { std::slice::from_raw_parts(raw_bitmap_bits as *const u8, pixel_count * 4) };
        let mut rgba_pixels = Vec::with_capacity(pixel_count * 4);
        for bgra_pixel in bgra_pixels.chunks_exact(4) {
            rgba_pixels.extend_from_slice(&[bgra_pixel[2], bgra_pixel[1], bgra_pixel[0], bgra_pixel[3]]);
        }

        Some(ProcessIcon::new(rgba_pixels, icon_width as u32, icon_height as u32))
    }

    fn resolve_icon_dimensions() -> (i32, i32) {
        let icon_width = unsafe { GetSystemMetrics(SM_CXICON) };
        let icon_height = unsafe { GetSystemMetrics(SM_CYICON) };

        (
            if icon_width > 0 { icon_width } else { Self::DEFAULT_ICON_DIMENSION_PIXELS },
            if icon_height > 0 { icon_height } else { Self::DEFAULT_ICON_DIMENSION_PIXELS },
        )
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
                Err(ProcessQueryError::open_process_failed(
                    process_info.get_process_id_raw(),
                    "OpenProcess returned a null process handle",
                ))
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
