use crate::constants::WINDBG_DEBUGGER_PLUGIN_ID;
use squalr_engine_api::{plugins::debugger::DebuggerPluginError, structures::processes::opened_process_info::OpenedProcessInfo};

pub(crate) struct WindbgBackend {
    process_info: OpenedProcessInfo,
}

impl WindbgBackend {
    pub(crate) fn new(process_info: OpenedProcessInfo) -> Self {
        Self { process_info }
    }

    pub(crate) fn attach(&self) -> Result<(), DebuggerPluginError> {
        Err(DebuggerPluginError::new(
            WINDBG_DEBUGGER_PLUGIN_ID,
            format!(
                "DbgEng attach is not implemented yet for process '{}' ({})",
                self.process_info.get_name(),
                self.process_info.get_process_id()
            ),
        ))
    }

    pub(crate) fn unavailable_error(&self) -> DebuggerPluginError {
        DebuggerPluginError::new(WINDBG_DEBUGGER_PLUGIN_ID, "DbgEng debugger backend is not implemented yet.")
    }
}
