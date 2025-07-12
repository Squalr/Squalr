use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use sysinfo::{ProcessRefreshKind, RefreshKind, System};

pub struct WindowsProcessMonitor {
    stop_signal: Arc<AtomicBool>,
    monitor_thread: Option<JoinHandle<()>>,
    system: Arc<RwLock<System>>,
}

impl WindowsProcessMonitor {
    pub fn new() -> Self {
        Self {
            stop_signal: Arc::new(AtomicBool::new(false)),
            monitor_thread: None,
            system: Arc::new(RwLock::new(System::new_with_specifics(
                RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
            ))),
        }
    }

    pub fn get_system(&self) -> Arc<RwLock<System>> {
        return self.system.clone();
    }

    pub fn start_monitoring(&mut self) {
        // Reset the stop signal in case this is a restart.
        self.stop_signal.store(false, Ordering::SeqCst);

        let stop_signal = self.stop_signal.clone();
        let system = self.system.clone();
        let handle = thread::spawn(move || {
            Self::monitor_loop(system, stop_signal);
        });

        self.monitor_thread = Some(handle);
    }

    pub fn stop_monitoring(&mut self) {
        self.stop_signal.store(true, Ordering::SeqCst);

        // Wait for the monitoring thread to finish.
        if let Some(handle) = self.monitor_thread.take() {
            // Ignore the result since we don't care about propagating thread panic information in this case.
            let _ = handle.join();
        }
    }

    fn monitor_loop(
        system: Arc<RwLock<System>>,
        stop_signal: Arc<AtomicBool>,
    ) {
        while !stop_signal.load(Ordering::SeqCst) {
            if let Ok(mut sys) = system.write() {
                sys.refresh_all();
            }

            thread::sleep(Duration::from_millis(250));
        }
    }
}

impl Drop for WindowsProcessMonitor {
    fn drop(&mut self) {
        self.stop_monitoring();
    }
}
