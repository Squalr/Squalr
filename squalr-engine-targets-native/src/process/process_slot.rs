use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_targets::ProcessQueryError;

pub(super) struct ProcessSlot {
    pub current: Option<OpenedProcessInfo>,
    release: Box<dyn Fn(u64) -> Result<(), ProcessQueryError> + Send + Sync>,
}

impl ProcessSlot {
    pub fn new(release: impl Fn(u64) -> Result<(), ProcessQueryError> + Send + Sync + 'static) -> Self {
        Self {
            current: None,
            release: Box::new(release),
        }
    }

    /// A failed close leaves ownership with this slot so the caller can retry.
    pub fn close(&mut self) -> Result<Option<OpenedProcessInfo>, ProcessQueryError> {
        if let Some(target) = &self.current {
            (self.release)(target.get_handle())?;
        }
        Ok(self.current.take())
    }
}

impl Drop for ProcessSlot {
    fn drop(&mut self) {
        if let Err(error) = self.close() {
            log::warn!("Failed to release the last process attachment: {error}");
        }
    }
}
