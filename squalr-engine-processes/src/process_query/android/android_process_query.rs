use crate::process_info::{Bitness, OpenedProcessInfo, ProcessIcon, ProcessInfo};
use crate::process_query::process_queryer::{ProcessQueryOptions, ProcessQueryer};
use jni::objects::JList;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::system::android_globals::AndroidGlobals;
use std::sync::{Arc, RwLock};
use sysinfo::{Pid, System};

pub struct AndroidProcessQuery {}

impl ProcessQueryer for AndroidProcessQuery {
    fn open_process(process_info: &ProcessInfo) -> Result<OpenedProcessInfo, String> {
        Ok(OpenedProcessInfo {
            pid: process_info.pid,
            name: process_info.name.clone(),
            handle: 0,
            bitness: Bitness::Bit64,
            icon: process_info.icon.clone(),
        })
    }

    fn close_process(_handle: u64) -> Result<(), String> {
        Ok(())
    }

    fn get_processes(
        options: ProcessQueryOptions,
        system: Arc<RwLock<System>>,
    ) -> Vec<ProcessInfo> {
        Logger::get_instance().log(LogLevel::Info, "Fetching processes...", None);
        let system_guard = match system.read() {
            Ok(guard) => guard,
            Err(e) => {
                Logger::get_instance().log(LogLevel::Error, &format!("Failed to acquire system read lock: {}", e), None);
                return Vec::new();
            }
        };

        let mut results = Vec::new();

        for (pid, proc_) in system_guard.processes() {
            let pid_u32 = pid.as_u32();
            let name = proc_.name().to_string_lossy().to_string();
            let is_windowed = Self::is_process_windowed(pid);
            let process_info = ProcessInfo {
                pid: pid_u32,
                name,
                is_windowed,
                icon: None,
            };
            let mut matches = true;

            if let Some(ref term) = options.search_name {
                if options.match_case {
                    matches &= process_info.name.contains(term);
                } else {
                    matches &= process_info.name.to_lowercase().contains(&term.to_lowercase());
                }
            }

            if let Some(required_pid) = options.required_pid {
                matches &= process_info.pid == required_pid.as_u32();
            }

            if options.require_windowed {
                matches &= process_info.is_windowed;
            }

            if matches {
                results.push(process_info);
            }

            if let Some(limit) = options.limit {
                if results.len() >= limit as usize {
                    break;
                }
            }
        }

        results
    }

    fn is_process_windowed(process_id: &Pid) -> bool {
        Logger::get_instance().log(LogLevel::Info, &format!("Checking pid[1]: {:?}", process_id), None);

        let mut env = match AndroidGlobals::get_instance().get_env() {
            Ok(env) => env,
            Err(e) => {
                Logger::get_instance().log(LogLevel::Error, &format!("Failed to get JNI environment: {}", e), None);
                return false;
            }
        };
        Logger::get_instance().log(LogLevel::Info, &format!("Checking pid[2]: {:?}", process_id), None);

        // Get ActivityManager class
        let activity_manager_class = match env.find_class("android/app/ActivityManager") {
            Ok(class) => class,
            Err(e) => {
                Logger::get_instance().log(LogLevel::Error, &format!("Failed to find ActivityManager class: {}", e), None);
                return false;
            }
        };
        Logger::get_instance().log(LogLevel::Info, &format!("Checking pid[3]: {:?}", process_id), None);

        // Get running app processes
        let running_apps = match env.call_static_method(activity_manager_class, "getRunningAppProcesses", "()Ljava/util/List;", &[]) {
            Ok(result) => match result.l() {
                Ok(obj) => obj,
                Err(e) => {
                    Logger::get_instance().log(
                        LogLevel::Error,
                        &format!("Failed to convert ActivityManager.getRunningAppProcesses result to JObject: {}", e),
                        None,
                    );
                    return false;
                }
            },
            Err(e) => {
                Logger::get_instance().log(LogLevel::Error, &format!("Failed to call ActivityManager.getRunningAppProcesses: {}", e), None);
                return false;
            }
        };
        Logger::get_instance().log(LogLevel::Info, &format!("Checking pid[4]: {:?}", process_id), None);

        // Convert to JList for easier iteration
        let process_list = match JList::from_env(&mut env, &running_apps) {
            Ok(list) => list,
            Err(e) => {
                Logger::get_instance().log(LogLevel::Error, &format!("Failed to create JList from running apps: {}", e), None);
                return false;
            }
        };
        Logger::get_instance().log(LogLevel::Info, &format!("Checking pid[5]: {:?}", process_id), None);

        let size = match process_list.size(&mut env) {
            Ok(s) => s,
            Err(e) => {
                Logger::get_instance().log(LogLevel::Error, &format!("Failed to get JList size: {}", e), None);
                return false;
            }
        };
        Logger::get_instance().log(LogLevel::Info, &format!("Checking pid[6]: {:?}", process_id), None);

        for i in 0..size {
            let process_info = match process_list.get(&mut env, i) {
                Ok(info) => match info {
                    Some(obj) => obj,
                    None => {
                        Logger::get_instance().log(LogLevel::Error, &format!("Process info at index {} is null", i), None);
                        continue;
                    }
                },
                Err(e) => {
                    Logger::get_instance().log(LogLevel::Error, &format!("Failed to get process info at index {}: {}", i, e), None);
                    continue;
                }
            };

            // Get the process ID
            let pid = match env.get_field(&process_info, "pid", "I") {
                Ok(field) => match field.i() {
                    Ok(val) => val as u32,
                    Err(e) => {
                        Logger::get_instance().log(LogLevel::Error, &format!("Failed to convert pid field to integer: {}", e), None);
                        continue;
                    }
                },
                Err(e) => {
                    Logger::get_instance().log(LogLevel::Error, &format!("Failed to get pid field: {}", e), None);
                    continue;
                }
            };

            // Check if this is our target process
            if pid == process_id.as_u32() {
                // Get importance level
                let importance = match env.get_field(&process_info, "importance", "I") {
                    Ok(field) => match field.i() {
                        Ok(val) => val,
                        Err(e) => {
                            Logger::get_instance().log(LogLevel::Error, &format!("Failed to convert importance field to integer: {}", e), None);
                            continue;
                        }
                    },
                    Err(e) => {
                        Logger::get_instance().log(LogLevel::Error, &format!("Failed to get importance field: {}", e), None);
                        continue;
                    }
                };

                Logger::get_instance().log(LogLevel::Error, &format!("Result: {}", importance <= 200), None);
                // Check if process is a foreground app or visible app
                // IMPORTANCE_FOREGROUND = 100
                // IMPORTANCE_VISIBLE = 200
                return importance <= 200;
            }
        }

        false
    }

    fn get_icon(_process_id: &Pid) -> Option<ProcessIcon> {
        None
    }
}
