use sysinfo::Pid;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Bitness {
    Bit32,
    Bit64,
}

#[derive(Clone, Debug)]
pub struct ProcessInfo {
    pub pid: Pid,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct OpenedProcessInfo {
    pub pid: Pid,
    pub name: String,
    pub handle: u64,
    pub bitness: Bitness,
}
