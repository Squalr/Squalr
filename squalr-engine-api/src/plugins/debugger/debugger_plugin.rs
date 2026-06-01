use crate::plugins::{
    Plugin,
    debugger::{debugger_error::DebuggerPluginError, debugger_session::DebuggerSession},
};
use crate::structures::processes::opened_process_info::OpenedProcessInfo;

pub trait DebuggerPlugin: Plugin {
    fn can_attach(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> bool;

    fn create_session(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> Result<Box<dyn DebuggerSession>, DebuggerPluginError>;
}
