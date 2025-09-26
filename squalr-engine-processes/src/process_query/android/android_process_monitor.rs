use crate::process_query::android::android_process_info::AndroidProcessInfo;
use std::collections::HashMap;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct AndroidProcessMonitor {
    stop_signal: Arc<AtomicBool>,
    monitor_thread: Option<JoinHandle<()>>,
    all_processes: Arc<RwLock<HashMap<u32, AndroidProcessInfo>>>,
    zygote_processes: Arc<RwLock<HashMap<u32, AndroidProcessInfo>>>,
}

impl AndroidProcessMonitor {
    pub fn new() -> Self {
        Self {
            stop_signal: Arc::new(AtomicBool::new(false)),
            monitor_thread: None,
            all_processes: Arc::new(RwLock::new(HashMap::new())),
            zygote_processes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get_all_processes(&self) -> Arc<RwLock<HashMap<u32, AndroidProcessInfo>>> {
        self.all_processes.clone()
    }

    pub fn get_zygote_processes(&self) -> Arc<RwLock<HashMap<u32, AndroidProcessInfo>>> {
        self.zygote_processes.clone()
    }

    pub fn start_monitoring(&mut self) {
        self.stop_signal.store(false, Ordering::SeqCst);

        let stop_signal = self.stop_signal.clone();
        let all_processes = self.all_processes.clone();
        let zygote_processes = self.zygote_processes.clone();

        let handle = thread::spawn(move || {
            Self::monitor_loop(all_processes, zygote_processes, stop_signal);
        });

        self.monitor_thread = Some(handle);
    }

    pub fn stop_monitoring(&mut self) {
        self.stop_signal.store(true, Ordering::SeqCst);

        if let Some(handle) = self.monitor_thread.take() {
            let _ = handle.join();
        }
    }

    fn monitor_loop(
        all_processes: Arc<RwLock<HashMap<u32, AndroidProcessInfo>>>,
        zygote_processes: Arc<RwLock<HashMap<u32, AndroidProcessInfo>>>,
        stop_signal: Arc<AtomicBool>,
    ) {
        while !stop_signal.load(Ordering::SeqCst) {
            let mut all_pids = HashMap::new();
            let mut zygote_pids = HashMap::new();

            if let Ok(entries) = fs::read_dir("/proc") {
                for entry in entries.flatten() {
                    if let Ok(process_id_str) = entry.file_name().into_string() {
                        if process_id_str.chars().all(|c| c.is_digit(10)) {
                            if let Ok(pid) = process_id_str.parse::<u32>() {
                                let mut package_name = String::new();
                                let mut parent_pid = 0;

                                // Read parent PID and process name from /proc/<pid>/cmdline
                                let cmdline_path = format!("/proc/{}/cmdline", pid);
                                if let Ok(cmdline_content) = fs::read_to_string(&cmdline_path) {
                                    // Parse out the package name, excluding any potential trailing info behind ':'.
                                    package_name = cmdline_content
                                        .split('\0')
                                        .next()
                                        .unwrap_or("")
                                        .split(':')
                                        .next()
                                        .unwrap_or("")
                                        .to_string();
                                }

                                // Read parent PID and process name from /proc/<pid>/stat
                                let stat_path = format!("/proc/{}/stat", pid);
                                if let Ok(stat_content) = fs::read_to_string(&stat_path) {
                                    let parts: Vec<&str> = stat_content.split_whitespace().collect();
                                    if parts.len() > 3 {
                                        if let Ok(ppid) = parts[3].parse::<u32>() {
                                            parent_pid = ppid;
                                        }
                                    }
                                }

                                let process_info = AndroidProcessInfo {
                                    process_id: pid,
                                    parent_process_id: parent_pid,
                                    package_name: package_name.clone(),
                                };

                                all_pids.insert(pid, process_info.clone());

                                // Identify zygote processes
                                if package_name == "zygote" || package_name == "zygote64" {
                                    zygote_pids.insert(pid, process_info);
                                }
                            }
                        }
                    }
                }
            }

            if let Ok(mut all_processes_guard) = all_processes.write() {
                *all_processes_guard = all_pids;
            }
            if let Ok(mut zygote_processes_guard) = zygote_processes.write() {
                *zygote_processes_guard = zygote_pids;
            }

            thread::sleep(Duration::from_millis(250));
        }
    }
}

impl Drop for AndroidProcessMonitor {
    fn drop(&mut self) {
        self.stop_monitoring();
    }
}
