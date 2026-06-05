use crate::process_query::process_query_error::ProcessQueryError;
use crate::process_query::process_query_options::ProcessQueryOptions;
use crate::process_query::process_queryer::ProcessQueryer;
use libc::{PROC_PIDPATHINFO_MAXSIZE, c_void, proc_pidpath, size_t, sysctl, sysctlnametomib};
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
use squalr_engine_api::structures::processes::target_architecture::{Endianness, TargetArchitecture};
use std::collections::HashSet;
use std::ffi::CString;
use std::fs;
use std::os::raw::{c_int, c_uint};
use std::path::{Path, PathBuf};
use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};

pub struct MacOsProcessQuery {}

type CFArrayRef = *const c_void;
type CFDictionaryRef = *const c_void;
type CFNumberRef = *const c_void;
type CFStringRef = *const c_void;
type CFTypeRef = *const c_void;
type CFIndex = isize;
type CGContextRef = *mut c_void;
type CGColorSpaceRef = *mut c_void;
type CGImageRef = *const c_void;
type DispatchQueue = *const c_void;

#[repr(C)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
struct CGSize {
    width: f64,
    height: f64,
}

#[repr(C)]
struct CGRect {
    origin: CGPoint,
    size: CGSize,
}

const CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY: u32 = 1 << 0;
const CG_NULL_WINDOW_ID: u32 = 0;
const CF_NUMBER_SINT32_TYPE: i32 = 3;
const NS_UTF8_STRING_ENCODING: usize = 4;
const MAX_PROCESS_ICON_EDGE_PX: u32 = 24;
const CG_IMAGE_ALPHA_PREMULTIPLIED_LAST: u32 = 1;
const CG_BITMAP_BYTE_ORDER_32_BIG: u32 = 4 << 12;
const CG_BITMAP_INFO_RGBA8_PREMULTIPLIED_LAST: u32 = CG_IMAGE_ALPHA_PREMULTIPLIED_LAST | CG_BITMAP_BYTE_ORDER_32_BIG;
const MACH_MAGIC_32_LE: [u8; 4] = [0xCE, 0xFA, 0xED, 0xFE];
const MACH_MAGIC_64_LE: [u8; 4] = [0xCF, 0xFA, 0xED, 0xFE];
const MACH_MAGIC_32_BE: [u8; 4] = [0xFE, 0xED, 0xFA, 0xCE];
const MACH_MAGIC_64_BE: [u8; 4] = [0xFE, 0xED, 0xFA, 0xCF];
const FAT_MAGIC_BE: [u8; 4] = [0xCA, 0xFE, 0xBA, 0xBE];
const FAT_MAGIC_LE: [u8; 4] = [0xBE, 0xBA, 0xFE, 0xCA];
const MACH_CPU_ARCH_ABI64: i32 = 0x0100_0000;
const MACH_CPU_TYPE_X86: i32 = 7;
const MACH_CPU_TYPE_ARM: i32 = 12;
const MACH_CPU_TYPE_POWERPC: i32 = 18;
const MACH_CPU_TYPE_X64: i32 = MACH_CPU_ARCH_ABI64 | MACH_CPU_TYPE_X86;
const MACH_CPU_TYPE_ARM64: i32 = MACH_CPU_ARCH_ABI64 | MACH_CPU_TYPE_ARM;

#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {}

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    static kCGWindowOwnerPID: CFStringRef;

    fn CGWindowListCopyWindowInfo(
        option: u32,
        relative_to_window: u32,
    ) -> CFArrayRef;
    fn CGImageGetWidth(image: CGImageRef) -> usize;
    fn CGImageGetHeight(image: CGImageRef) -> usize;
    fn CGColorSpaceCreateDeviceRGB() -> CGColorSpaceRef;
    fn CGColorSpaceRelease(color_space: CGColorSpaceRef);
    fn CGBitmapContextCreate(
        data: *mut c_void,
        width: usize,
        height: usize,
        bits_per_component: usize,
        bytes_per_row: usize,
        color_space: CGColorSpaceRef,
        bitmap_info: u32,
    ) -> CGContextRef;
    fn CGContextDrawImage(
        context: CGContextRef,
        rect: CGRect,
        image: CGImageRef,
    );
    fn CGContextRelease(context: CGContextRef);
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

#[link(name = "System")]
unsafe extern "C" {
    static _dispatch_main_q: c_void;
    fn dispatch_sync_f(
        queue: DispatchQueue,
        context: *mut c_void,
        work: extern "C" fn(*mut c_void),
    );
}

enum MainThreadIconLookupKind {
    RunningApplication { process_id: i32 },
    FilePath { path: String },
}

struct MainThreadIconLookupRequest {
    kind: MainThreadIconLookupKind,
    result: Option<ProcessIcon>,
}

impl MacOsProcessQuery {
    fn default_target_architecture() -> TargetArchitecture {
        #[cfg(target_arch = "aarch64")]
        {
            TargetArchitecture::arm64()
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            TargetArchitecture::x64()
        }
    }

    fn get_process_target_architecture(process_id: u32) -> TargetArchitecture {
        Self::query_live_process_cpu_type(process_id)
            .map(|cpu_type| Self::target_architecture_from_mach_cpu_type(cpu_type, Endianness::Little))
            .or_else(|| {
                Self::get_process_executable_path(&Pid::from_u32(process_id))
                    .and_then(|executable_path| fs::read(executable_path).ok())
                    .and_then(|executable_bytes| Self::parse_mach_target_architecture_from_bytes(&executable_bytes))
            })
            .unwrap_or_else(Self::default_target_architecture)
    }

    fn query_live_process_cpu_type(process_id: u32) -> Option<i32> {
        let sysctl_name = CString::new("sysctl.proc_cputype").ok()?;
        let mut management_information_base = [0 as c_int; 8];
        let mut management_information_base_len = management_information_base.len() as size_t;

        if unsafe {
            sysctlnametomib(
                sysctl_name.as_ptr(),
                management_information_base.as_mut_ptr(),
                &mut management_information_base_len,
            )
        } != 0
        {
            return None;
        }

        if management_information_base_len >= management_information_base.len() as size_t {
            return None;
        }

        management_information_base[management_information_base_len as usize] = process_id as c_int;
        management_information_base_len += 1;

        let mut cpu_type: c_int = 0;
        let mut cpu_type_size = std::mem::size_of::<c_int>() as size_t;

        if unsafe {
            sysctl(
                management_information_base.as_mut_ptr(),
                management_information_base_len as c_uint,
                &mut cpu_type as *mut c_int as *mut c_void,
                &mut cpu_type_size,
                std::ptr::null_mut(),
                0,
            )
        } != 0
        {
            return None;
        }

        Some(cpu_type)
    }

    fn parse_mach_target_architecture_from_bytes(executable_bytes: &[u8]) -> Option<TargetArchitecture> {
        if executable_bytes.len() < 8 {
            return None;
        }

        let magic = executable_bytes.get(0..4)?;

        if magic == MACH_MAGIC_32_LE.as_slice() || magic == MACH_MAGIC_64_LE.as_slice() {
            let cpu_type = i32::from_le_bytes(executable_bytes.get(4..8)?.try_into().ok()?);
            return Some(Self::target_architecture_from_mach_cpu_type(cpu_type, Endianness::Little));
        }

        if magic == MACH_MAGIC_32_BE.as_slice() || magic == MACH_MAGIC_64_BE.as_slice() {
            let cpu_type = i32::from_be_bytes(executable_bytes.get(4..8)?.try_into().ok()?);
            return Some(Self::target_architecture_from_mach_cpu_type(cpu_type, Endianness::Big));
        }

        if magic == FAT_MAGIC_BE.as_slice() || magic == FAT_MAGIC_LE.as_slice() {
            let is_little_endian_fat = magic == FAT_MAGIC_LE.as_slice();
            let read_u32 = |bytes: &[u8]| -> Option<u32> {
                let fixed_bytes: [u8; 4] = bytes.try_into().ok()?;
                Some(if is_little_endian_fat {
                    u32::from_le_bytes(fixed_bytes)
                } else {
                    u32::from_be_bytes(fixed_bytes)
                })
            };
            let architecture_count = read_u32(executable_bytes.get(4..8)?)? as usize;
            let available_architecture_count = executable_bytes
                .len()
                .saturating_sub(8)
                .checked_div(20)
                .unwrap_or(0);
            let architecture_count = architecture_count.min(available_architecture_count);
            let mut fallback_target_architecture = None;

            for architecture_index in 0..architecture_count {
                let architecture_offset = 8usize.saturating_add(architecture_index.saturating_mul(20));
                let cpu_type_end_offset = architecture_offset.checked_add(4)?;
                let cpu_type = read_u32(executable_bytes.get(architecture_offset..cpu_type_end_offset)?)? as i32;
                let target_architecture = Self::target_architecture_from_mach_cpu_type(cpu_type, Endianness::Little);

                if matches!(target_architecture.get_instruction_set_id(), "arm64" | "x64") {
                    return Some(target_architecture);
                }

                fallback_target_architecture.get_or_insert(target_architecture);
            }

            return fallback_target_architecture;
        }

        None
    }

    fn target_architecture_from_mach_cpu_type(
        cpu_type: i32,
        endianness: Endianness,
    ) -> TargetArchitecture {
        match cpu_type {
            MACH_CPU_TYPE_X86 => TargetArchitecture::x86(),
            MACH_CPU_TYPE_X64 => TargetArchitecture::x64(),
            MACH_CPU_TYPE_ARM => TargetArchitecture::arm(),
            MACH_CPU_TYPE_ARM64 => TargetArchitecture::arm64(),
            MACH_CPU_TYPE_POWERPC => TargetArchitecture::power_pc32_be(),
            _ => {
                let pointer_width = if (cpu_type & MACH_CPU_ARCH_ABI64) != 0 {
                    Bitness::Bit64
                } else {
                    Bitness::Bit32
                };

                TargetArchitecture::unknown(pointer_width, endianness)
            }
        }
    }

    fn task_for_pid_failure_details(task_for_pid_status: i32) -> String {
        let status_reason = match task_for_pid_status {
            4 => "KERN_INVALID_ARGUMENT",
            5 => "KERN_FAILURE",
            8 => "KERN_PROTECTION_FAILURE",
            _ => "UNKNOWN_KERN_STATUS",
        };

        format!(
            "task_for_pid failed with status {} ({}). On macOS this generally means the target process is protected or the caller lacks debugging rights",
            task_for_pid_status, status_reason
        )
    }

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

    fn resolve_icon_lookup_paths(executable_path: &str) -> Vec<PathBuf> {
        let executable_path = Path::new(executable_path);
        let mut icon_lookup_paths = executable_path
            .ancestors()
            .filter(|ancestor_path| {
                ancestor_path
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("app"))
            })
            .map(Path::to_path_buf)
            .collect::<Vec<_>>();

        icon_lookup_paths.reverse();

        icon_lookup_paths
    }

    fn get_icon(process_id: &Pid) -> Option<ProcessIcon> {
        Self::get_running_application_icon(process_id)
            .or_else(|| Self::get_process_executable_path(process_id).and_then(|executable_path| Self::get_icon_for_executable_path(&executable_path)))
    }

    fn get_running_application_icon(process_id: &Pid) -> Option<ProcessIcon> {
        Self::run_icon_lookup_on_main_thread(MainThreadIconLookupKind::RunningApplication {
            process_id: process_id.as_u32() as i32,
        })
    }

    fn decode_ns_image_to_process_icon(icon_image: *mut Object) -> Option<ProcessIcon> {
        if icon_image.is_null() {
            return None;
        }

        let cg_image: CGImageRef = unsafe {
            msg_send![
                icon_image,
                CGImageForProposedRect: std::ptr::null_mut::<CGRect>()
                context: std::ptr::null_mut::<Object>()
                hints: std::ptr::null_mut::<Object>()
            ]
        };
        if cg_image.is_null() {
            return None;
        }

        Self::render_cg_image_to_process_icon(cg_image)
    }

    fn render_cg_image_to_process_icon(cg_image: CGImageRef) -> Option<ProcessIcon> {
        if cg_image.is_null() {
            return None;
        }

        let source_width = unsafe { CGImageGetWidth(cg_image) } as u32;
        let source_height = unsafe { CGImageGetHeight(cg_image) } as u32;
        if source_width == 0 || source_height == 0 {
            return None;
        }

        let scale_numerator = MAX_PROCESS_ICON_EDGE_PX as f64;
        let scale_denominator = (source_width.max(source_height)) as f64;
        let scale_factor = (scale_numerator / scale_denominator).min(1.0);
        let icon_width = ((source_width as f64) * scale_factor).round().max(1.0) as usize;
        let icon_height = ((source_height as f64) * scale_factor).round().max(1.0) as usize;
        let bytes_per_row = icon_width * 4;
        let mut icon_rgba_bytes = vec![0u8; bytes_per_row * icon_height];
        let color_space = unsafe { CGColorSpaceCreateDeviceRGB() };
        if color_space.is_null() {
            return None;
        }

        let bitmap_context = unsafe {
            CGBitmapContextCreate(
                icon_rgba_bytes.as_mut_ptr() as *mut c_void,
                icon_width,
                icon_height,
                8,
                bytes_per_row,
                color_space,
                CG_BITMAP_INFO_RGBA8_PREMULTIPLIED_LAST,
            )
        };
        unsafe { CGColorSpaceRelease(color_space) };

        if bitmap_context.is_null() {
            return None;
        }

        unsafe {
            CGContextDrawImage(
                bitmap_context,
                CGRect {
                    origin: CGPoint { x: 0.0, y: 0.0 },
                    size: CGSize {
                        width: icon_width as f64,
                        height: icon_height as f64,
                    },
                },
                cg_image,
            );
            CGContextRelease(bitmap_context);
        }

        Some(ProcessIcon::new(icon_rgba_bytes, icon_width as u32, icon_height as u32))
    }

    fn get_icon_for_executable_path(executable_path: &str) -> Option<ProcessIcon> {
        Self::resolve_icon_lookup_paths(executable_path)
            .into_iter()
            .find_map(|icon_lookup_path| {
                Self::run_icon_lookup_on_main_thread(MainThreadIconLookupKind::FilePath {
                    path: icon_lookup_path.to_string_lossy().to_string(),
                })
            })
    }

    fn run_icon_lookup_on_main_thread(icon_lookup_kind: MainThreadIconLookupKind) -> Option<ProcessIcon> {
        let mut icon_lookup_request = MainThreadIconLookupRequest {
            kind: icon_lookup_kind,
            result: None,
        };
        let is_main_thread: bool = unsafe { msg_send![class!(NSThread), isMainThread] };

        if is_main_thread {
            Self::execute_main_thread_icon_lookup(&mut icon_lookup_request);
        } else {
            unsafe {
                dispatch_sync_f(
                    &_dispatch_main_q,
                    &mut icon_lookup_request as *mut MainThreadIconLookupRequest as *mut c_void,
                    Self::dispatch_main_thread_icon_lookup,
                );
            }
        }

        icon_lookup_request.result
    }

    extern "C" fn dispatch_main_thread_icon_lookup(context: *mut c_void) {
        if context.is_null() {
            return;
        }

        let icon_lookup_request = unsafe { &mut *(context as *mut MainThreadIconLookupRequest) };
        Self::execute_main_thread_icon_lookup(icon_lookup_request);
    }

    fn execute_main_thread_icon_lookup(icon_lookup_request: &mut MainThreadIconLookupRequest) {
        icon_lookup_request.result = match &icon_lookup_request.kind {
            MainThreadIconLookupKind::RunningApplication { process_id } => Self::get_running_application_icon_on_main_thread(*process_id),
            MainThreadIconLookupKind::FilePath { path } => Self::get_icon_for_lookup_path_on_main_thread(path),
        };
    }

    fn get_running_application_icon_on_main_thread(process_id: i32) -> Option<ProcessIcon> {
        let autorelease_pool: *mut Object = unsafe { msg_send![class!(NSAutoreleasePool), new] };
        if autorelease_pool.is_null() {
            return None;
        }

        let process_icon = (|| {
            let running_application: *mut Object = unsafe {
                msg_send![
                    class!(NSRunningApplication),
                    runningApplicationWithProcessIdentifier: process_id
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

    fn get_icon_for_lookup_path_on_main_thread(icon_lookup_path: &str) -> Option<ProcessIcon> {
        let icon_lookup_path_bytes = icon_lookup_path.as_bytes();
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
                    stringWithBytes: icon_lookup_path_bytes.as_ptr()
                    length: icon_lookup_path_bytes.len()
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
            return Err(ProcessQueryError::open_process_failed(
                process_id,
                Self::task_for_pid_failure_details(task_for_pid_status),
            ));
        }

        let target_architecture = Self::get_process_target_architecture(process_id);

        Ok(OpenedProcessInfo::new(
            process_id,
            process_info.get_name().to_string(),
            task_port as u64,
            target_architecture.get_pointer_width(),
            process_info.get_icon().clone(),
        )
        .with_target_architecture(target_architecture))
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
        let system = System::new_with_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing().without_tasks()));
        let mut matched_processes = Vec::new();
        let window_owner_process_ids = Self::collect_window_owner_process_ids();
        let process_limit = options.limit.unwrap_or(u64::MAX) as usize;

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

            let process_icon = if options.fetch_icons { Self::get_icon(process_id) } else { None };

            matched_processes.push(ProcessInfo::new(process_id_raw, process_name, process_is_windowed, process_icon));
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

        assert!(!MacOsProcessQuery::matches_process_filters(&options, "Calculator", false, 1234,));
    }

    #[test]
    fn process_filter_respects_case_insensitive_search() {
        let mut options = create_options();
        options.search_name = Some("calculator".to_string());
        options.match_case = false;

        assert!(MacOsProcessQuery::matches_process_filters(&options, "Calculator", true, 1234,));
    }

    #[test]
    fn process_filter_respects_required_process_id() {
        let mut options = create_options();
        options.required_process_id = Some(Pid::from_u32(44));

        assert!(!MacOsProcessQuery::matches_process_filters(&options, "Calculator", true, 43,));
        assert!(MacOsProcessQuery::matches_process_filters(&options, "Calculator", true, 44,));
    }
}
