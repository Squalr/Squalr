#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryBasicInformation64 {
    pub BaseAddress: u64,
    pub AllocationBase: u64,
    pub AllocationProtect: u32,
    pub RegionSize: u64,
    pub State: u32,
    pub Protect: u32,
    pub Type: u32,
}
