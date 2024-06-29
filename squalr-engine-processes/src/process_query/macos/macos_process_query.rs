use sysinfo::{Pid, System};
use crate::process_query::IProcessQueryer;

pub struct MacOsProcessQuery {
    system: System,
}

impl MacOsProcessQuery {
    pub fn new() -> Self {
        MacOsProcessQuery {
            system: System::new_all(),
        }
    }
}

impl IProcessQueryer for MacOsProcessQuery {
    // Not implemented
}
