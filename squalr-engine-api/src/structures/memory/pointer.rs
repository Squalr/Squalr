use crate::structures::memory::address_display::{format_absolute_address, format_module_address};
use crate::structures::memory::pointer_chain_segment::PointerChainSegment;
use crate::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Pointer {
    address: u64,
    offsets: Vec<PointerChainSegment>,
    module_name: String,
    pointer_size: PointerScanPointerSize,
}

impl Pointer {
    pub fn new(
        address: u64,
        offsets: Vec<i64>,
        module_name: String,
    ) -> Self {
        Self::new_with_size(address, offsets, module_name, PointerScanPointerSize::Pointer64)
    }

    pub fn new_with_size(
        address: u64,
        offsets: Vec<i64>,
        module_name: String,
        pointer_size: PointerScanPointerSize,
    ) -> Self {
        Self::new_with_size_and_segments(
            address,
            offsets.into_iter().map(PointerChainSegment::Offset).collect(),
            module_name,
            pointer_size,
        )
    }

    pub fn new_with_size_and_segments(
        address: u64,
        offsets: Vec<PointerChainSegment>,
        module_name: String,
        pointer_size: PointerScanPointerSize,
    ) -> Self {
        Self {
            address,
            offsets,
            module_name,
            pointer_size,
        }
    }

    pub fn get_address(&self) -> u64 {
        self.address
    }

    pub fn set_address(
        &mut self,
        address: u64,
    ) {
        self.address = address;
    }

    pub fn get_offsets(&self) -> Vec<i64> {
        self.offsets
            .iter()
            .filter_map(PointerChainSegment::as_offset)
            .collect()
    }

    pub fn get_offset_segments(&self) -> &[PointerChainSegment] {
        &self.offsets
    }

    pub fn set_offsets(
        &mut self,
        offsets: Vec<i64>,
    ) {
        self.offsets = offsets.into_iter().map(PointerChainSegment::Offset).collect();
    }

    pub fn set_offset_segments(
        &mut self,
        offsets: Vec<PointerChainSegment>,
    ) {
        self.offsets = offsets;
    }

    pub fn has_symbolic_offsets(&self) -> bool {
        self.offsets
            .iter()
            .any(|pointer_chain_segment| pointer_chain_segment.symbol_name().is_some())
    }

    pub fn get_module_name(&self) -> &str {
        &self.module_name
    }

    pub fn set_module_name(
        &mut self,
        module_name: String,
    ) {
        self.module_name = module_name;
    }

    pub fn get_pointer_size(&self) -> PointerScanPointerSize {
        self.pointer_size
    }

    pub fn get_root_display_text(&self) -> String {
        if self.module_name.is_empty() {
            format_absolute_address(self.address)
        } else {
            format_module_address(&self.module_name, self.address)
        }
    }

    pub fn set_pointer_size(
        &mut self,
        pointer_size: PointerScanPointerSize,
    ) {
        self.pointer_size = pointer_size;
    }

    pub fn resolve_final_address<ResolveModuleAddress, ReadPointerValue>(
        &self,
        mut resolve_module_address: ResolveModuleAddress,
        mut read_pointer_value: ReadPointerValue,
    ) -> Option<u64>
    where
        ResolveModuleAddress: FnMut(&str, u64) -> Option<u64>,
        ReadPointerValue: FnMut(u64, PointerScanPointerSize) -> Option<u64>,
    {
        let mut resolved_address = if self.module_name.is_empty() {
            self.address
        } else {
            resolve_module_address(&self.module_name, self.address)?
        };

        for pointer_chain_segment in &self.offsets {
            let pointer_offset = pointer_chain_segment.as_offset()?;
            let pointer_value = read_pointer_value(resolved_address, self.pointer_size)?;
            resolved_address = Self::apply_pointer_offset(pointer_value, pointer_offset)?;
        }

        Some(resolved_address)
    }

    pub fn apply_pointer_offset(
        address: u64,
        pointer_offset: i64,
    ) -> Option<u64> {
        if pointer_offset >= 0 {
            address.checked_add(pointer_offset as u64)
        } else {
            address.checked_sub(pointer_offset.unsigned_abs())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Pointer;
    use crate::structures::memory::pointer_chain_segment::PointerChainSegment;
    use crate::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;

    #[test]
    fn resolve_final_address_walks_pointer_chain_from_module_base() {
        let pointer = Pointer::new_with_size(0x10, vec![0x10, 0x20], "game.exe".to_string(), PointerScanPointerSize::Pointer64);

        let mut resolved_reads = Vec::new();
        let resolved_address = pointer.resolve_final_address(
            |module_name, offset| {
                assert_eq!(module_name, "game.exe");
                assert_eq!(offset, 0x10);
                Some(0x1010)
            },
            |address, pointer_size| {
                resolved_reads.push((address, pointer_size));

                match address {
                    0x1010 => Some(0x2000),
                    0x2010 => Some(0x3000),
                    _ => None,
                }
            },
        );

        assert_eq!(resolved_address, Some(0x3020));
        assert_eq!(
            resolved_reads,
            vec![
                (0x1010, PointerScanPointerSize::Pointer64),
                (0x2010, PointerScanPointerSize::Pointer64)
            ]
        );
    }

    #[test]
    fn resolve_final_address_supports_negative_offsets() {
        let pointer = Pointer::new_with_size(0x5000, vec![-0x20], String::new(), PointerScanPointerSize::Pointer32);

        let resolved_address = pointer.resolve_final_address(
            |_module_name, _offset| Some(0),
            |address, pointer_size| {
                assert_eq!(address, 0x5000);
                assert_eq!(pointer_size, PointerScanPointerSize::Pointer32);

                Some(0x1234)
            },
        );

        assert_eq!(resolved_address, Some(0x1214));
    }

    #[test]
    fn resolve_final_address_returns_root_address_when_offsets_are_empty() {
        let pointer = Pointer::new(0x44, Vec::new(), "engine.dll".to_string());

        let resolved_address = pointer.resolve_final_address(
            |module_name, offset| {
                assert_eq!(module_name, "engine.dll");
                assert_eq!(offset, 0x44);
                Some(0x4044)
            },
            |_address, _pointer_size| None,
        );

        assert_eq!(resolved_address, Some(0x4044));
    }

    #[test]
    fn resolve_final_address_returns_none_for_unresolved_symbolic_offsets() {
        let pointer = Pointer::new_with_size_and_segments(
            0x44,
            vec![PointerChainSegment::Symbol(String::from("health"))],
            "engine.dll".to_string(),
            PointerScanPointerSize::Pointer64,
        );

        let resolved_address = pointer.resolve_final_address(|_module_name, _offset| Some(0x4044), |_address, _pointer_size| Some(0x5000));

        assert_eq!(resolved_address, None);
    }
}
