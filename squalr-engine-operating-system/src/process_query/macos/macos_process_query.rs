use crate::process_query::process_query_error::ProcessQueryError;
use crate::process_query::process_query_options::ProcessQueryOptions;
use crate::process_query::process_queryer::ProcessQueryer;
use image::ImageReader;
use libc::{PROC_PIDPATHINFO_MAXSIZE, c_void, proc_pidpath};
use mach2::kern_return::KERN_SUCCESS;
use mach2::mach_port::mach_port_deallocate;
use mach2::port::{MACH_PORT_NULL, mach_port_name_t, mach_port_t};
use mach2::traps::{mach_task_self, task_for_pid};
use objc::runtime::Object;
use objc::{class, msg_send, sel, sel_impl};
use squalr_engine_api::structures::memory::bitness::Bitness;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::processes::process_icon::ProcessIcon;
use squalr_engine_api::structures::processes::process_info::ProcessInfo;
use std::io::Cursor;
use std::os::raw::c_int;
use sysinfo::{Pid, ProcessesToUpdate, System};

pub struct MacOsProcessQuery {}

type CFArrayRef = *const c_void;
type CFDictionaryRef = *const c_void;
type CFNumberRef = *const c_void;
type CFStringRef = *const c_void;
type CFTypeRef = *const c_void;
type CFIndex = isize;

const CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY: u32 = 1 << 0;
const CG_NULL_WINDOW_ID: u32 = 0;
const CF_NUMBER_SINT32_TYPE: i32 = 3;
const NS_UTF8_STRING_ENCODING: usize = 4;

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    static kCGWindowOwnerPID: CFStringRef;

    fn CGWindowListCopyWindowInfo(
        option: u32,
        relative_to_window: u32,
    ) -> CFArrayRef;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFArrayGetCount(array: CFArrayRef) -> CFIndex;
    fn CFArrayGetValueAtIndex(
        array: CFArrayRef,
        index: CFIndex,
    ) -> *const c_void;
    fn CFDictionaryGetValue(
        dictionary: CFDictionaryRef,
        key: *const c_void,
    ) -> *const c_void;
    fn CFNumberGetValue(
        number: CFNumberRef,
        number_type: i32,
        value_ptr: *mut c_void,
    ) -> u8;
    fn CFRelease(cf: CFTypeRef);
}

impl MacOsProcessQuery {
    fn is_process_windowed(process_id: &Pid) -> bool {
        let owner_pid = process_id.as_u32() as i32;
        let window_info_array = unsafe { CGWindowListCopyWindowInfo(CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY, CG_NULL_WINDOW_ID) };

        if window_info_array.is_null() {
            return false;
        }

        let window_count = unsafe { CFArrayGetCount(window_info_array) };
        let mut has_window = false;

        for window_index in 0..window_count {
            let window_dictionary = unsafe { CFArrayGetValueAtIndex(window_info_array, window_index) as CFDictionaryRef };
            if window_dictionary.is_null() {
                continue;
            }

            let owner_pid_number = unsafe { CFDictionaryGetValue(window_dictionary, kCGWindowOwnerPID as *const c_void) as CFNumberRef };
            if owner_pid_number.is_null() {
                continue;
            }

            let mut window_owner_pid: i32 = 0;
            let parsed_number = unsafe { CFNumberGetValue(owner_pid_number, CF_NUMBER_SINT32_TYPE, &mut window_owner_pid as *mut i32 as *mut c_void) };

            if parsed_number != 0 && window_owner_pid == owner_pid {
                has_window = true;
                break;
            }
        }

        unsafe { CFRelease(window_info_array as CFTypeRef) };
        has_window
    }

    fn get_process_executable_path(process_id: &Pid) -> Option<String> {
        let mut path_buffer = vec![0u8; PROC_PIDPATHINFO_MAXSIZE as usize];
        let path_length = unsafe { proc_pidpath(process_id.as_u32() as c_int, path_buffer.as_mut_ptr() as *mut c_void, path_buffer.len() as u32) };

        if path_length <= 0 {
            return None;
        }

        let path_string = String::from_utf8_lossy(&path_buffer[..path_length as usize])
            .trim_end_matches('\0')
            .to_string();

        if path_string.is_empty() { None } else { Some(path_string) }
    }

    fn get_icon(process_id: &Pid) -> Option<ProcessIcon> {
        let executable_path = Self::get_process_executable_path(process_id)?;
        let executable_path_bytes = executable_path.as_bytes();

        let autorelease_pool: *mut Object = unsafe { msg_send![class!(NSAutoreleasePool), new] };
        if autorelease_pool.is_null() {
            return None;
        }

        let process_icon = (|| {
            let workspace: *mut Object = unsafe { msg_send![class!(NSWorkspace), sharedWorkspace] };
            if workspace.is_null() {
                return None;
            }

            let ns_executable_path: *mut Object = unsafe {
                msg_send![
                    class!(NSString),
                    stringWithBytes: executable_path_bytes.as_ptr()
                    length: executable_path_bytes.len()
                    encoding: NS_UTF8_STRING_ENCODING
                ]
            };

            if ns_executable_path.is_null() {
                return None;
            }

            let icon_image: *mut Object = unsafe { msg_send![workspace, iconForFile: ns_executable_path] };
            if icon_image.is_null() {
                return None;
            }

            let tiff_data: *mut Object = unsafe { msg_send![icon_image, TIFFRepresentation] };
            if tiff_data.is_null() {
                return None;
            }

            let image_bytes_ptr: *const u8 = unsafe { msg_send![tiff_data, bytes] };
            let image_bytes_len: usize = unsafe { msg_send![tiff_data, length] };
            if image_bytes_ptr.is_null() || image_bytes_len == 0 {
                return None;
            }

            let image_bytes = unsafe { std::slice::from_raw_parts(image_bytes_ptr, image_bytes_len) };
            let image_reader = ImageReader::new(Cursor::new(image_bytes))
                .with_guessed_format()
                .ok()?;
            let decoded_image = image_reader.decode().ok()?;
            let rgba_image = decoded_image.to_rgba8();
            let (icon_width, icon_height) = rgba_image.dimensions();
            let icon_rgba_bytes = rgba_image.into_raw();

            Some(ProcessIcon::new(icon_rgba_bytes, icon_width, icon_height))
        })();

        let _: () = unsafe { msg_send![autorelease_pool, drain] };
        process_icon
    }
}

impl ProcessQueryer for MacOsProcessQuery {
    fn start_monitoring() -> Result<(), ProcessQueryError> {
        Ok(())
    }

    fn stop_monitoring() -> Result<(), ProcessQueryError> {
        Ok(())
    }

    fn open_process(process_info: &ProcessInfo) -> Result<OpenedProcessInfo, ProcessQueryError> {
        let process_id = process_info.get_process_id_raw();
        let mut task_port: mach_port_t = MACH_PORT_NULL;
        let task_for_pid_status = unsafe { task_for_pid(mach_task_self(), process_id as c_int, &mut task_port as *mut mach_port_t) };

        if task_for_pid_status != KERN_SUCCESS || task_port == MACH_PORT_NULL {
            return Err(ProcessQueryError::OpenProcessFailed { process_id });
        }

        Ok(OpenedProcessInfo::new(
            process_id,
            process_info.get_name().to_string(),
            task_port as u64,
            Bitness::Bit64,
            process_info.get_icon().clone(),
        ))
    }

    fn close_process(handle: u64) -> Result<(), ProcessQueryError> {
        if handle == 0 {
            return Ok(());
        }

        let deallocate_status = unsafe { mach_port_deallocate(mach_task_self(), handle as mach_port_name_t) };

        if deallocate_status == KERN_SUCCESS {
            Ok(())
        } else {
            Err(ProcessQueryError::CloseProcessFailed { handle })
        }
    }

    fn get_processes(options: ProcessQueryOptions) -> Vec<ProcessInfo> {
        let mut system = System::new_all();

        system.refresh_processes(ProcessesToUpdate::All, true);

        system
            .processes()
            .iter()
            .filter_map(|(process_id, process)| {
                let process_name = process.name().to_string_lossy().to_string();
                let process_is_windowed = Self::is_process_windowed(process_id);
                let process_icon = if options.fetch_icons { Self::get_icon(process_id) } else { None };

                let process_info = ProcessInfo::new(process_id.as_u32(), process_name, process_is_windowed, process_icon);

                let mut matches = true;

                if options.require_windowed {
                    matches &= process_info.get_is_windowed();
                }

                if let Some(ref term) = options.search_name {
                    if options.match_case {
                        matches &= process_info.get_name().contains(term);
                    } else {
                        matches &= process_info
                            .get_name()
                            .to_lowercase()
                            .contains(&term.to_lowercase());
                    }
                }

                if let Some(required_pid) = options.required_process_id {
                    matches &= process_info.get_process_id_raw() == required_pid.as_u32();
                }

                matches.then_some(process_info)
            })
            .take(options.limit.unwrap_or(u64::MAX) as usize)
            .collect()
    }
}
