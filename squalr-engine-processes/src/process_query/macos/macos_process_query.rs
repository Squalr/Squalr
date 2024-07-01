use crate::process_query::IProcessQueryer;

use sysinfo::{Pid, System};

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
