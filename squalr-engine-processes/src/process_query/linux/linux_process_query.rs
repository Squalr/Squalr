use sysinfo::{Pid, System};
use crate::process_query::IProcessQueryer;

pub struct LinuxProcessQuery {
    system: System,
}

impl LinuxProcessQuery {
    pub fn new() -> Self {
        LinuxProcessQuery {
            system: System::new_all(),
        }
    }
}

impl IProcessQueryer for LinuxProcessQuery {
    // Not implemented
}
