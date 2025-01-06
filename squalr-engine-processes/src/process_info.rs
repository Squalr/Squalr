use sysinfo::Pid;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Bitness {
    Bit32,
    Bit64,
}

#[derive(Clone, Debug)]
pub struct ProcessIcon {
    pub bytes_rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug)]
pub struct ProcessInfo {
    pub pid: Pid,
    pub name: String,
    pub is_windowed: bool,
    pub icon: Option<ProcessIcon>,
}

#[derive(Clone, Debug)]
pub struct OpenedProcessInfo {
    pub pid: Pid,
    pub name: String,
    pub handle: u64,
    pub bitness: Bitness,
    pub icon: Option<ProcessIcon>,
}
