use sysinfo::Pid;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Bitness {
    Bit32,
    Bit64,
}

#[derive(Copy, Clone, Debug)]
pub struct ProcessInfo {
    pub pid: Pid,
    pub handle: u64,
    pub bitness: Bitness,
}
