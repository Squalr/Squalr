#[cfg(target_family = "unix")]
use libc::{ESRCH, kill};
use squalr_engine_api::{
    events::{
        engine_event::{EngineEvent, EngineEventRequest},
        process::changed::process_changed_event::ProcessChangedEvent,
    },
    structures::processes::opened_process_info::OpenedProcessInfo,
};
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};
#[cfg(not(target_family = "unix"))]
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate};

use super::process_slot::ProcessSlot;

use squalr_engine_api::utils::periodic_worker::PeriodicWorker;
use squalr_engine_targets::ProcessQueryError;
use std::sync::{
    RwLockReadGuard,
    atomic::{AtomicU64, Ordering},
};

const OPEN_PROCESS_DEATH_POLL_INTERVAL: Duration = Duration::from_millis(250);

pub struct ProcessManager {
    opened_process: Arc<RwLock<ProcessSlot>>,
    attachment_gate: Arc<RwLock<()>>,
    generation: Arc<AtomicU64>,
    invalidate: Arc<dyn Fn() + Send + Sync>,
    event_emitter: Arc<dyn Fn(EngineEvent) + Send + Sync>,
    _monitor: PeriodicWorker,
}

impl ProcessManager {
    pub fn new(event_emitter: Arc<dyn Fn(EngineEvent) + Send + Sync>) -> Self {
        Self::with_liveness(
            event_emitter,
            |process| native_process_is_alive(process.get_process_id()),
            crate::process_query::process_queryer::ProcessQuery::close_process,
            || {},
        )
    }

    pub fn with_liveness(
        event_emitter: Arc<dyn Fn(EngineEvent) + Send + Sync>,
        is_alive: impl Fn(&OpenedProcessInfo) -> bool + Send + Sync + 'static,
        release: impl Fn(u64) -> Result<(), ProcessQueryError> + Send + Sync + 'static,
        invalidate: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        let opened_process = Arc::new(RwLock::new(ProcessSlot::new(release)));
        let attachment_gate = Arc::new(RwLock::new(()));
        let generation = Arc::new(AtomicU64::new(0));
        let invalidate = Arc::new(invalidate);
        let monitor = {
            let process = opened_process.clone();
            let generation = generation.clone();
            let gate = attachment_gate.clone();
            let invalidate = invalidate.clone();
            let emit = event_emitter.clone();
            PeriodicWorker::start(move || {
                let is_dead = |target: &OpenedProcessInfo| !is_alive(target);
                let detected_death = process
                    .read()
                    .is_ok_and(|current| current.current.as_ref().is_some_and(is_dead));
                if detected_death
                    && let Ok(_exclusive) = gate.write()
                    && let Ok(mut current) = process.write()
                    && current.current.as_ref().is_some_and(is_dead)
                {
                    invalidate();
                    if let Err(error) = current.close() {
                        log::warn!("Failed to release exited target: {error}");
                        current.current = None;
                    }
                    generation.fetch_add(1, Ordering::Release);
                    emit(ProcessChangedEvent { process_info: None }.to_engine_event());
                }
                OPEN_PROCESS_DEATH_POLL_INTERVAL
            })
        };

        Self {
            opened_process,
            generation,
            attachment_gate,
            invalidate,
            event_emitter,
            _monitor: monitor,
        }
    }

    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    /// Keeps the attachment stable throughout one command.
    pub fn attachment_guard(&self) -> Result<RwLockReadGuard<'_, ()>, ProcessQueryError> {
        self.attachment_gate
            .read()
            .map_err(|error| ProcessQueryError::internal("read attachment", error.to_string()))
    }

    /// Retires the previous attachment and its dependent state before publishing the replacement.
    pub fn replace_process(
        &self,
        replacement: Option<OpenedProcessInfo>,
        expected_generation: Option<u64>,
    ) -> Result<Option<OpenedProcessInfo>, ProcessQueryError> {
        let _exclusive = self
            .attachment_gate
            .write()
            .map_err(|error| ProcessQueryError::internal("change attachment", error.to_string()))?;
        if expected_generation.is_some_and(|generation| generation != self.generation()) {
            return Err(ProcessQueryError::internal("change attachment", "Target changed before the command started."));
        }
        let mut current = self
            .opened_process
            .write()
            .map_err(|error| ProcessQueryError::internal("change attachment", error.to_string()))?;
        if current.current.is_none() && replacement.is_none() {
            return Ok(None);
        }
        let previous = current.close()?;
        (self.invalidate)();
        current.current = replacement.clone();
        self.generation.fetch_add(1, Ordering::Release);
        (self.event_emitter)(ProcessChangedEvent { process_info: replacement }.to_engine_event());
        Ok(previous)
    }

    pub fn set_opened_process(
        &self,
        process: OpenedProcessInfo,
    ) {
        if let Err(error) = self.replace_process(Some(process), None) {
            log::error!("Failed to set opened process: {error}");
        }
    }

    pub fn clear_opened_process(&self) {
        if let Err(error) = self.replace_process(None, None) {
            log::error!("Failed to clear opened process: {error}");
        }
    }

    pub fn get_opened_process(&self) -> Option<OpenedProcessInfo> {
        self.opened_process
            .read()
            .ok()
            .and_then(|current| current.current.clone())
    }

    /// Borrows the current target for an operation that must finish before its handle is released.
    pub fn with_opened_process(
        &self,
        operation: impl FnOnce(&OpenedProcessInfo),
    ) {
        match self.opened_process.read() {
            Ok(current) => {
                if let Some(target) = current.current.as_ref() {
                    operation(target);
                }
            }
            Err(error) => log::error!("Failed to borrow opened process: {error}"),
        }
    }
}

pub fn native_process_is_alive(process_id: u32) -> bool {
    #[cfg(target_family = "unix")]
    {
        let kill_result = unsafe { kill(process_id as i32, 0) };

        if kill_result == 0 {
            return true;
        }

        let error_kind = std::io::Error::last_os_error().raw_os_error();

        return error_kind != Some(ESRCH);
    }

    #[cfg(not(target_family = "unix"))]
    {
        let pid = Pid::from_u32(process_id);
        let monitored_processes = [pid];
        let refresh_kind = ProcessRefreshKind::nothing().without_tasks();

        let mut system = sysinfo::System::new();
        system.refresh_processes_specifics(ProcessesToUpdate::Some(&monitored_processes), true, refresh_kind);

        system.process(pid).is_some()
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn process_liveness_check_finds_current_process() {
        assert!(super::native_process_is_alive(std::process::id()));
    }
}
