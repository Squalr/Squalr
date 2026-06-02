use crate::{
    constants::{WINDBG_DEBUGGER_PLUGIN_DESCRIPTION, WINDBG_DEBUGGER_PLUGIN_DISPLAY_NAME, WINDBG_DEBUGGER_PLUGIN_ID},
    session::WindbgDebuggerSession,
};
use squalr_engine_api::{
    plugins::{
        Plugin, PluginCapability, PluginMetadata, PluginPackage, PluginPermission,
        debugger::{DebuggerPlugin, DebuggerPluginError, DebuggerSession, DebuggerTraceEventSink},
    },
    structures::processes::opened_process_info::OpenedProcessInfo,
};

pub struct WindbgDebuggerPlugin {
    metadata: PluginMetadata,
}

impl WindbgDebuggerPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new_with_permissions(
                WINDBG_DEBUGGER_PLUGIN_ID,
                WINDBG_DEBUGGER_PLUGIN_DISPLAY_NAME,
                WINDBG_DEBUGGER_PLUGIN_DESCRIPTION,
                vec![PluginCapability::Debugger],
                vec![
                    PluginPermission::AttachDebugger,
                    PluginPermission::ControlDebuggerExecution,
                    PluginPermission::ManageDebuggerBreakpoints,
                    PluginPermission::ReadRegisters,
                    PluginPermission::WriteRegisters,
                ],
                true,
                true,
            ),
        }
    }
}

impl Default for WindbgDebuggerPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for WindbgDebuggerPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
}

impl PluginPackage for WindbgDebuggerPlugin {
    fn as_debugger_plugin(&self) -> Option<&dyn DebuggerPlugin> {
        Some(self)
    }
}

impl DebuggerPlugin for WindbgDebuggerPlugin {
    fn can_attach(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> bool {
        cfg!(windows) && process_info.get_handle() != 0
    }

    fn create_session(
        &self,
        process_info: &OpenedProcessInfo,
        trace_event_sink: DebuggerTraceEventSink,
    ) -> Result<Box<dyn DebuggerSession>, DebuggerPluginError> {
        Ok(Box::new(WindbgDebuggerSession::new(process_info.clone(), trace_event_sink)))
    }
}
