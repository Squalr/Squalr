use sysinfo::Pid;

#[derive(Copy, Clone)]
pub struct ProcessInfo {
    pub pid: Pid,
    pub handle: u64,
}
