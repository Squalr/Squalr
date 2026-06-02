use crate::plugins::{
    Plugin,
    debugger::{debugger_error::DebuggerPluginError, debugger_session::DebuggerSession},
};
use crate::structures::debugger::DebuggerTraceEvent;
use crate::structures::processes::opened_process_info::OpenedProcessInfo;
use std::sync::Arc;

pub type DebuggerTraceEventSink = Arc<dyn Fn(DebuggerTraceEvent) + Send + Sync>;

pub trait DebuggerPlugin: Plugin {
    fn can_attach(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> bool;

    fn create_session(
        &self,
        process_info: &OpenedProcessInfo,
        trace_event_sink: DebuggerTraceEventSink,
    ) -> Result<Box<dyn DebuggerSession>, DebuggerPluginError>;
}
