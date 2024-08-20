use crate::process_query::IProcessQueryer;

use sysinfo::{Pid, System};

pub struct LinuxProcessQuery {
    system: System,
}

impl LinuxProcessQuery {
    pub fn new(
    ) -> Self {
        LinuxProcessQuery {
            system: System::new_all(),
        }
    }
}

impl IProcessQueryer for LinuxProcessQuery {
    // Not implemented
}
