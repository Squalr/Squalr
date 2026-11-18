use std::{
    sync::mpsc::{self, RecvTimeoutError, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

/// Owns a periodic session job. Dropping the owner interrupts its sleep and waits for active work.
pub struct PeriodicWorker {
    stop: Option<Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl PeriodicWorker {
    pub fn start(mut tick: impl FnMut() -> Duration + Send + 'static) -> Self {
        let (stop, stopped) = mpsc::channel();
        let thread = thread::spawn(move || while let Err(RecvTimeoutError::Timeout) = stopped.recv_timeout(tick()) {});
        Self {
            stop: Some(stop),
            thread: Some(thread),
        }
    }
}

impl Drop for PeriodicWorker {
    fn drop(&mut self) {
        drop(self.stop.take());
        if let Some(worker) = self.thread.take()
            && worker.thread().id() != thread::current().id()
            && worker.join().is_err()
        {
            log::error!("Periodic session worker terminated unexpectedly.");
        }
    }
}
