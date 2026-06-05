use crate::structures::memory::bitness::Bitness;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Endianness {
    Little,
    Big,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TargetArchitecture {
    instruction_set_id: String,
    instruction_data_type_id: String,
    pointer_width: Bitness,
    endianness: Endianness,
}

impl TargetArchitecture {
    pub fn new(
        instruction_set_id: impl Into<String>,
        instruction_data_type_id: impl Into<String>,
        pointer_width: Bitness,
        endianness: Endianness,
    ) -> Self {
        Self {
            instruction_set_id: instruction_set_id.into(),
            instruction_data_type_id: instruction_data_type_id.into(),
            pointer_width,
            endianness,
        }
    }

    pub fn default_for_bitness(pointer_width: Bitness) -> Self {
        match pointer_width {
            Bitness::Bit32 => Self::x86(),
            Bitness::Bit64 => Self::x64(),
        }
    }

    pub fn unknown(
        pointer_width: Bitness,
        endianness: Endianness,
    ) -> Self {
        Self::new("unknown", "i_unknown", pointer_width, endianness)
    }

    pub fn x86() -> Self {
        Self::new("x86", "i_x86", Bitness::Bit32, Endianness::Little)
    }

    pub fn x64() -> Self {
        Self::new("x64", "i_x64", Bitness::Bit64, Endianness::Little)
    }

    pub fn arm() -> Self {
        Self::new("arm", "i_arm", Bitness::Bit32, Endianness::Little)
    }

    pub fn thumb() -> Self {
        Self::new("thumb", "i_thumb", Bitness::Bit32, Endianness::Little)
    }

    pub fn arm32_from_interworking_address(address: u64) -> (Self, u64) {
        if address & 1 == 0 {
            (Self::arm(), address)
        } else {
            (Self::thumb(), address & !1)
        }
    }

    pub fn arm64() -> Self {
        Self::new("arm64", "i_arm64", Bitness::Bit64, Endianness::Little)
    }

    pub fn power_pc32_be() -> Self {
        Self::new("ppc32be", "i_ppc32be", Bitness::Bit32, Endianness::Big)
    }

    pub fn get_instruction_set_id(&self) -> &str {
        &self.instruction_set_id
    }

    pub fn get_instruction_data_type_id(&self) -> &str {
        &self.instruction_data_type_id
    }

    pub fn get_pointer_width(&self) -> Bitness {
        self.pointer_width
    }

    pub fn get_endianness(&self) -> Endianness {
        self.endianness
    }
}

impl Default for TargetArchitecture {
    fn default() -> Self {
        Self::x64()
    }
}

#[cfg(test)]
mod tests {
    use super::TargetArchitecture;

    #[test]
    fn arm32_interworking_address_selects_thumb_and_clears_state_bit() {
        let (target_architecture, normalized_address) = TargetArchitecture::arm32_from_interworking_address(0x1001);

        assert_eq!(target_architecture.get_instruction_set_id(), "thumb");
        assert_eq!(target_architecture.get_instruction_data_type_id(), "i_thumb");
        assert_eq!(normalized_address, 0x1000);
    }

    #[test]
    fn arm32_interworking_address_selects_arm_for_aligned_address() {
        let (target_architecture, normalized_address) = TargetArchitecture::arm32_from_interworking_address(0x1000);

        assert_eq!(target_architecture.get_instruction_set_id(), "arm");
        assert_eq!(normalized_address, 0x1000);
    }
}
