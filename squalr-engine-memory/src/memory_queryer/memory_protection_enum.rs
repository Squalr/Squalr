bitflags::bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct MemoryProtectionEnum: u32 {
        const NONE = 0x0;
        const READ = 0x1;
        const WRITE = 0x2;
        const EXECUTE = 0x4;
        const SHARED = 0x8;
        const COPY_ON_WRITE = 0x10;
    }
}
