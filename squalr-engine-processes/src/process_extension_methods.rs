use crate::process_query::IProcessQueryer;
use sysinfo::Pid;

pub trait ProcessExtensionMethods {
    fn is_system_process(&self, query: &dyn IProcessQueryer) -> bool;
    fn has_window(&self, query: &dyn IProcessQueryer) -> bool;
    fn is_32bit(&self) -> bool;
    fn is_64bit(&self) -> bool;
    fn get_icon(&self, query: &dyn IProcessQueryer) -> Option<Vec<u8>>;
}

impl ProcessExtensionMethods for Pid {
    fn is_system_process(&self, query: &dyn IProcessQueryer) -> bool {
        query.is_process_system_process(self)
    }

    fn has_window(&self, query: &dyn IProcessQueryer) -> bool {
        query.is_process_windowed(self)
    }

    fn is_32bit(&self) -> bool {
        !cfg!(target_pointer_width = "64")
    }

    fn is_64bit(&self) -> bool {
        cfg!(target_pointer_width = "64")
    }

    fn get_icon(&self, query: &dyn IProcessQueryer) -> Option<Vec<u8>> {
        query.get_icon(self)
    }
}
