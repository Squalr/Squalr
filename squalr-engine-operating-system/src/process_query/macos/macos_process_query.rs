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
use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::os::raw::c_int;
use std::sync::{LazyLock, RwLock};
use sysinfo::{Pid, ProcessesToUpdate, System};

pub struct MacOsProcessQuery {}
static PROCESS_ICON_CACHE: LazyLock<RwLock<HashMap<String, Option<ProcessIcon>>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

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
    fn collect_window_owner_process_ids() -> HashSet<u32> {
        let mut window_owner_process_ids = HashSet::new();
        let window_info_array = unsafe { CGWindowListCopyWindowInfo(CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY, CG_NULL_WINDOW_ID) };

        if window_info_array.is_null() {
            return window_owner_process_ids;
        }

        let window_count = unsafe { CFArrayGetCount(window_info_array) };

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

            if parsed_number != 0 && window_owner_pid > 0 {
                window_owner_process_ids.insert(window_owner_pid as u32);
            }
        }

        unsafe { CFRelease(window_info_array as CFTypeRef) };
        window_owner_process_ids
    }

    fn matches_process_filters(
        options: &ProcessQueryOptions,
        process_name: &str,
        process_is_windowed: bool,
        process_id_raw: u32,
    ) -> bool {
        if options.require_windowed && !process_is_windowed {
            return false;
        }

        if let Some(search_term) = options.search_name.as_ref() {
            if options.match_case {
                if !process_name.contains(search_term) {
                    return false;
                }
            } else if !process_name
                .to_lowercase()
                .contains(&search_term.to_lowercase())
            {
                return false;
            }
        }

        if let Some(required_process_id) = options.required_process_id {
            if process_id_raw != required_process_id.as_u32() {
                return false;
            }
        }

        true
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
        let process_id_raw = process_id.as_u32();
        let process_icon_cache_key = format!("pid:{process_id_raw}");

        if let Ok(icon_cache) = PROCESS_ICON_CACHE.read() {
            if let Some(cached_process_icon) = icon_cache.get(&process_icon_cache_key) {
                return cached_process_icon.clone();
            }
        }

        let process_icon = Self::get_running_application_icon(process_id)
            .or_else(|| Self::get_process_executable_path(process_id).and_then(|executable_path| Self::get_icon_for_executable_path(&executable_path)));

        if let Ok(mut icon_cache) = PROCESS_ICON_CACHE.write() {
            icon_cache.insert(process_icon_cache_key, process_icon.clone());
        }

        process_icon
    }

    fn get_running_application_icon(process_id: &Pid) -> Option<ProcessIcon> {
        let process_id_value = process_id.as_u32() as i32;
        let autorelease_pool: *mut Object = unsafe { msg_send![class!(NSAutoreleasePool), new] };
        if autorelease_pool.is_null() {
            return None;
        }

        let process_icon = (|| {
            let running_application: *mut Object = unsafe {
                msg_send![
                    class!(NSRunningApplication),
                    runningApplicationWithProcessIdentifier: process_id_value
                ]
            };
            if running_application.is_null() {
                return None;
            }

            let icon_image: *mut Object = unsafe { msg_send![running_application, icon] };
            Self::decode_ns_image_to_process_icon(icon_image)
        })();

        let _: () = unsafe { msg_send![autorelease_pool, drain] };
        process_icon
    }

    fn decode_ns_image_to_process_icon(icon_image: *mut Object) -> Option<ProcessIcon> {
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
    }

    fn get_icon_for_executable_path(executable_path: &str) -> Option<ProcessIcon> {
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
            Self::decode_ns_image_to_process_icon(icon_image)
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
        let mut matched_processes = Vec::new();
        let window_owner_process_ids = Self::collect_window_owner_process_ids();
        let process_limit = options.limit.unwrap_or(u64::MAX) as usize;

        system.refresh_processes(ProcessesToUpdate::All, true);

        for (process_id, process) in system.processes() {
            if matched_processes.len() >= process_limit {
                break;
            }

            let process_id_raw = process_id.as_u32();
            let process_name = process.name().to_string_lossy().to_string();
            let process_is_windowed = window_owner_process_ids.contains(&process_id_raw);

            if !Self::matches_process_filters(&options, &process_name, process_is_windowed, process_id_raw) {
                continue;
            }

            let process_icon = if options.fetch_icons {
                Self::get_icon(process_id)
            } else {
                None
            };

            matched_processes.push(ProcessInfo::new(
                process_id_raw,
                process_name,
                process_is_windowed,
                process_icon,
            ));
        }

        matched_processes
    }
}

#[cfg(test)]
mod tests {
    use super::MacOsProcessQuery;
    use crate::process_query::process_query_options::ProcessQueryOptions;
    use sysinfo::Pid;

    fn create_options() -> ProcessQueryOptions {
        ProcessQueryOptions {
            required_process_id: None,
            search_name: None,
            require_windowed: false,
            match_case: false,
            fetch_icons: false,
            limit: None,
        }
    }

    #[test]
    fn process_filter_rejects_non_windowed_when_required() {
        let mut options = create_options();
        options.require_windowed = true;

        assert!(!MacOsProcessQuery::matches_process_filters(
            &options,
            "Calculator",
            false,
            1234,
        ));
    }

    #[test]
    fn process_filter_respects_case_insensitive_search() {
        let mut options = create_options();
        options.search_name = Some("calculator".to_string());
        options.match_case = false;

        assert!(MacOsProcessQuery::matches_process_filters(
            &options,
            "Calculator",
            true,
            1234,
        ));
    }

    #[test]
    fn process_filter_respects_required_process_id() {
        let mut options = create_options();
        options.required_process_id = Some(Pid::from_u32(44));

        assert!(!MacOsProcessQuery::matches_process_filters(
            &options,
            "Calculator",
            true,
            43,
        ));
        assert!(MacOsProcessQuery::matches_process_filters(
            &options,
            "Calculator",
            true,
            44,
        ));
    }
}
