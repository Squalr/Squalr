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

    pub fn x86() -> Self {
        Self::new("x86", "i_x86", Bitness::Bit32, Endianness::Little)
    }

    pub fn x64() -> Self {
        Self::new("x64", "i_x64", Bitness::Bit64, Endianness::Little)
    }

    pub fn arm() -> Self {
        Self::new("arm", "i_arm", Bitness::Bit32, Endianness::Little)
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
