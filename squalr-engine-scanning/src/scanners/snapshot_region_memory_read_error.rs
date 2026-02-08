use thiserror::Error;

#[derive(Debug, Error)]
pub enum SnapshotRegionMemoryReadError {
    #[error("Snapshot region at base address 0x{base_address:016X} has zero size.")]
    ZeroSizedRegion { base_address: u64 },
    #[error("Failed to read snapshot region at base address 0x{base_address:016X} while {context}.")]
    ReadFailed { base_address: u64, context: &'static str },
    #[error(
        "Failed to read one or more snapshot chunks for base address 0x{base_address:016X} while {context}; first failed address: 0x{first_failed_address:016X}."
    )]
    ChunkReadFailed {
        base_address: u64,
        context: &'static str,
        first_failed_address: u64,
    },
}

impl SnapshotRegionMemoryReadError {
    pub fn zero_sized_region(base_address: u64) -> Self {
        Self::ZeroSizedRegion { base_address }
    }

    pub fn read_failed(
        base_address: u64,
        context: &'static str,
    ) -> Self {
        Self::ReadFailed { base_address, context }
    }

    pub fn chunk_read_failed(
        base_address: u64,
        context: &'static str,
        first_failed_address: u64,
    ) -> Self {
        Self::ChunkReadFailed {
            base_address,
            context,
            first_failed_address,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SnapshotRegionMemoryReadError;

    #[test]
    fn read_failed_error_includes_context() {
        let error = SnapshotRegionMemoryReadError::read_failed(0x1234, "reading standalone region");
        let rendered_error = error.to_string();

        assert!(rendered_error.contains("0x0000000000001234"));
        assert!(rendered_error.contains("reading standalone region"));
    }

    #[test]
    fn chunk_read_failed_error_includes_first_failed_address() {
        let error = SnapshotRegionMemoryReadError::chunk_read_failed(0x1000, "reading merged chunk", 0x1400);
        let rendered_error = error.to_string();

        assert!(rendered_error.contains("0x0000000000001000"));
        assert!(rendered_error.contains("0x0000000000001400"));
    }
}
