use crate::{
    constants::{DBGENG_DEBUGGER_PLUGIN_DESCRIPTION, DBGENG_DEBUGGER_PLUGIN_DISPLAY_NAME, DBGENG_DEBUGGER_PLUGIN_ID},
    session::DbgEngDebuggerSession,
};
use squalr_engine_api::{
    plugins::{
        Plugin, PluginCapability, PluginMetadata, PluginPackage, PluginPermission,
        debugger::{DebuggerPlugin, DebuggerPluginError, DebuggerSession, DebuggerTraceEventSink},
    },
    structures::processes::opened_process_info::OpenedProcessInfo,
};

pub struct DbgEngDebuggerPlugin {
    metadata: PluginMetadata,
}

impl DbgEngDebuggerPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new_with_permissions(
                DBGENG_DEBUGGER_PLUGIN_ID,
                DBGENG_DEBUGGER_PLUGIN_DISPLAY_NAME,
                DBGENG_DEBUGGER_PLUGIN_DESCRIPTION,
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

impl Default for DbgEngDebuggerPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for DbgEngDebuggerPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
}

impl PluginPackage for DbgEngDebuggerPlugin {
    fn as_debugger_plugin(&self) -> Option<&dyn DebuggerPlugin> {
        Some(self)
    }
}

impl DebuggerPlugin for DbgEngDebuggerPlugin {
    fn can_attach(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> bool {
        let instruction_set_id = process_info.get_target_architecture().get_instruction_set_id();

        cfg!(windows) && process_info.get_handle() != 0 && matches!(instruction_set_id, "x86" | "x64")
    }

    fn create_session(
        &self,
        process_info: &OpenedProcessInfo,
        trace_event_sink: DebuggerTraceEventSink,
    ) -> Result<Box<dyn DebuggerSession>, DebuggerPluginError> {
        Ok(Box::new(DbgEngDebuggerSession::new(process_info.clone(), trace_event_sink)))
    }
}
