use sysinfo::{Pid, System};
use crate::process_query::IProcessQueryer;

pub struct UnixProcessQuery {
    system: System,
}

impl UnixProcessQuery {
    pub fn new() -> Self {
        UnixProcessQuery {
            system: System::new_all(),
        }
    }
}

impl IProcessQueryer for UnixProcessQuery {
    // Not implemented
}
