use crate::process_info::{Bitness, ProcessInfo};
use crate::process_query::process_queryer::ProcessQueryOptions;
use crate::process_query::process_queryer::ProcessQueryer;
use image::{DynamicImage, ImageBuffer, Rgba};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use sysinfo::{Pid, System};
use windows_sys::Win32::Foundation::{CloseHandle, BOOL, HANDLE, HWND, LPARAM};
use windows_sys::Win32::Graphics::Gdi::{GetDC, GetDIBits, ReleaseDC, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS};
use windows_sys::Win32::System::ProcessStatus::K32GetModuleFileNameExW;
use windows_sys::Win32::System::Threading::{IsWow64Process, IsWow64Process2};
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows_sys::Win32::UI::Shell::ExtractIconW;
use windows_sys::Win32::UI::WindowsAndMessaging::{DestroyIcon, EnumWindows};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetIconInfo, ICONINFO};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindowThreadProcessId, HICON};

pub struct WindowsProcessQuery {
    system: System,
}

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
        finder.found.store(true, Ordering::SeqCst);
        BOOL::from(false)
    } else {
        BOOL::from(true)
    }
}

impl WindowsProcessQuery {
    pub fn new() -> Self {
        WindowsProcessQuery { system: System::new_all() }
    }

    fn get_process_name(
        &self,
        process_id: Pid,
    ) -> Option<String> {
        self.system
            .process(process_id)
            .map(|process| process.name().to_str().unwrap_or_default().to_string())
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

impl ProcessQueryer for WindowsProcessQuery {
    fn get_processes(
        &mut self,
        options: ProcessQueryOptions,
    ) -> Vec<ProcessInfo> {
        self.system.refresh_all();
        let mut processes: Vec<ProcessInfo> = self
            .system
            .processes()
            .keys()
            .filter(|pid| {
                if let Some(name) = self.get_process_name(**pid) {
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
                    matches
                } else {
                    false
                }
            })
            .filter_map(|pid| {
                self.get_process_name(*pid).map(|name| ProcessInfo {
                    pid: *pid,
                    name,
                    handle: 0,
                    bitness: Bitness::Bit64,
                })
            })
            .collect();

        // Limit the result after filtering
        if let Some(limit) = options.limit {
            processes.truncate(limit as usize);
        }

        processes
    }

    fn get_process_name(
        &self,
        pid: Pid,
    ) -> Option<String> {
        self.system
            .process(pid)
            .map(|process| process.name().to_str().unwrap_or_default().to_string())
    }

    fn is_process_windowed(
        &self,
        process_id: &Pid,
    ) -> bool {
        let finder = WindowFinder {
            pid: process_id.as_u32(),
            found: AtomicBool::new(false),
        };

        unsafe {
            EnumWindows(Some(enum_window_callback), std::mem::transmute(&finder));
        }

        finder.found.load(Ordering::SeqCst)
    }

    fn get_icon(
        &self,
        process_id: &Pid,
    ) -> Option<DynamicImage> {
        unsafe {
            let process_handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, process_id.as_u32() as u32);

            if process_handle.is_null() {
                return None;
            }

            // Switch to wide char buffer for W functions
            let mut buffer = [0u16; 260];
            let len = K32GetModuleFileNameExW(process_handle, std::ptr::null_mut(), buffer.as_mut_ptr(), buffer.len() as u32);

            if len == 0 {
                return None;
            }

            let icon_handle = ExtractIconW(std::ptr::null_mut(), buffer.as_ptr(), 0);

            if icon_handle.is_null() {
                return None;
            }

            let mut icon_info = std::mem::zeroed::<ICONINFO>();
            if GetIconInfo(icon_handle as HICON, &mut icon_info) == 0 {
                DestroyIcon(icon_handle as HICON);
                return None;
            }

            let dc = GetDC(std::ptr::null_mut());
            let mut bmi = std::mem::zeroed::<BITMAPINFO>();
            bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;

            if GetDIBits(dc, icon_info.hbmColor, 0, 0, std::ptr::null_mut(), &mut bmi, DIB_RGB_COLORS) == 0 {
                ReleaseDC(std::ptr::null_mut(), dc);
                DestroyIcon(icon_handle as HICON);
                return None;
            }

            let size = ((bmi.bmiHeader.biWidth * 4 + 3) & !3) * bmi.bmiHeader.biHeight.abs();
            let mut pixels = vec![0u8; size as usize];

            if GetDIBits(
                dc,
                icon_info.hbmColor,
                0,
                bmi.bmiHeader.biHeight.unsigned_abs(),
                pixels.as_mut_ptr() as *mut _,
                &mut bmi,
                DIB_RGB_COLORS,
            ) == 0
            {
                ReleaseDC(std::ptr::null_mut(), dc);
                DestroyIcon(icon_handle as HICON);
                return None;
            }

            ReleaseDC(std::ptr::null_mut(), dc);
            DestroyIcon(icon_handle as HICON);

            // Convert raw pixels to DynamicImage
            let width = bmi.bmiHeader.biWidth as u32;
            let height = bmi.bmiHeader.biHeight.unsigned_abs();
            let mut img = ImageBuffer::new(width, height);

            for y in 0..height {
                for x in 0..width {
                    let i = ((y * width + x) * 4) as usize;
                    img.put_pixel(
                        x,
                        y,
                        Rgba([
                            pixels[i + 2], // BGR to RGB
                            pixels[i + 1],
                            pixels[i],
                            pixels[i + 3],
                        ]),
                    );
                }
            }

            Some(DynamicImage::ImageRgba8(img))
        }
    }

    fn open_process(
        &self,
        process_id: &Pid,
    ) -> Result<ProcessInfo, String> {
        unsafe {
            let handle: HANDLE = OpenProcess(PROCESS_ALL_ACCESS, 0, process_id.as_u32());
            if handle == std::ptr::null_mut() {
                return Err("Failed to open process".to_string());
            } else {
                let bitness = self.get_process_bitness(handle);
                let process_info = ProcessInfo {
                    pid: *process_id,
                    name: "TODO".to_string(),
                    handle: handle as u64,
                    bitness,
                };
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
