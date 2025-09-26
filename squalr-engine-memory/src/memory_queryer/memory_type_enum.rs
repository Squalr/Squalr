bitflags::bitflags! {
    #[derive(PartialEq, Eq)]
    pub struct MemoryTypeEnum: u32 {
        const NONE = 0x1;
        const PRIVATE = 0x2;
        const IMAGE = 0x4;
        const MAPPED = 0x8;
    }
}
