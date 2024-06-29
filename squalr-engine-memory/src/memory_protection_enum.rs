bitflags::bitflags! {
    pub struct MemoryProtectionEnum: u32 {
        const WRITE = 0x1;
        const EXECUTE = 0x2;
        const COPY_ON_WRITE = 0x4;
    }
}
