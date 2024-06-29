bitflags::bitflags! {
    /// Flags that indicate the memory protection for a region of memory.
    pub struct MemoryProtectionEnum: u32 {
        /// Writable memory.
        const WRITE = 0x1;

        /// Executable memory.
        const EXECUTE = 0x2;

        /// Memory marked as copy on write.
        const COPY_ON_WRITE = 0x4;
    }
}

bitflags::bitflags! {
    /// Flags that indicate the memory type for a region of memory.
    pub struct MemoryTypeEnum: u32 {
        /// No other flags specified.
        const NONE = 0x1;

        /// Indicates that the memory pages within the region are private (that is, not shared by other processes).
        const PRIVATE = 0x2;

        /// Indicates that the memory pages within the region are mapped into the view of an image section.
        const IMAGE = 0x4;

        /// Indicates that the memory pages within the region are mapped into the view of a section.
        const MAPPED = 0x8;
    }
}
