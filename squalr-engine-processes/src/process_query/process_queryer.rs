use crate::process_info::OpenedProcessInfo;
use crate::process_info::ProcessIcon;
use crate::process_info::ProcessInfo;
use crate::process_monitor::ProcessMonitor;
use once_cell::sync::Lazy;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use sysinfo::Pid;
use sysinfo::System;

pub(crate) trait ProcessQueryer {
    fn open_process(process_info: &ProcessInfo) -> Result<OpenedProcessInfo, String>;
    fn close_process(handle: u64) -> Result<(), String>;
    fn get_processes(
        options: ProcessQueryOptions,
        system: Arc<RwLock<System>>,
    ) -> Vec<ProcessInfo>;
    fn is_process_windowed(process_id: &Pid) -> bool;
    fn get_icon(process_id: &Pid) -> Option<ProcessIcon>;
}

pub struct ProcessQueryOptions {
    pub required_pid: Option<Pid>,
    pub search_name: Option<String>,
    pub require_windowed: bool,
    pub match_case: bool,
    pub fetch_icons: bool,
    pub limit: Option<u64>,
}

#[cfg(any(target_os = "android"))]
use crate::process_query::android::android_process_query::AndroidProcessQuery as ProcessQueryImpl;

#[cfg(any(target_os = "linux"))]
use crate::process_query::linux::linux_process_query::LinuxProcessQuery as ProcessQueryImpl;

#[cfg(any(target_os = "macos"))]
use crate::process_query::macos::macos_process_query::MacOsProcessQuery as ProcessQueryImpl;

#[cfg(target_os = "windows")]
use crate::process_query::windows::windows_process_query::WindowsProcessQuery as ProcessQueryImpl;

pub struct ProcessQuery;

pub(crate) static PROCESS_MONITOR: Lazy<Mutex<ProcessMonitor>> = Lazy::new(|| Mutex::new(ProcessMonitor::new()));

impl ProcessQuery {
    pub fn start_monitoring() -> Result<(), String> {
        let mut monitor = PROCESS_MONITOR
            .lock()
            .map_err(|e| format!("Failed to acquire process monitor lock: {}", e))?;

        monitor.start_monitoring();

        Ok(())
    }

    pub fn stop_monitoring() -> Result<(), String> {
        let mut monitor = PROCESS_MONITOR
            .lock()
            .map_err(|e| format!("Failed to acquire process monitor lock: {}", e))?;

        monitor.stop_monitoring();

        Ok(())
    }

    pub fn open_process(process_info: &ProcessInfo) -> Result<OpenedProcessInfo, String> {
        ProcessQueryImpl::open_process(process_info)
    }

    pub fn close_process(handle: u64) -> Result<(), String> {
        ProcessQueryImpl::close_process(handle)
    }

    pub fn get_processes(options: ProcessQueryOptions) -> Vec<ProcessInfo> {
        if let Ok(monitor) = PROCESS_MONITOR.lock() {
            ProcessQueryImpl::get_processes(options, monitor.get_system())
        } else {
            Logger::get_instance().log(LogLevel::Error, "Error fetching processes: Failed to acquire process monitor lock.", None);
            vec![]
        }
    }
}
