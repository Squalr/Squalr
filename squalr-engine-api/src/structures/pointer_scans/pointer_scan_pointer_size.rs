use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::memory::bitness::Bitness;
use crate::structures::memory::endian::Endian;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum PointerScanPointerSize {
    Pointer32,
    Pointer32be,
    #[default]
    Pointer64,
    Pointer64be,
}

impl PointerScanPointerSize {
    pub fn get_size_in_bytes(&self) -> u64 {
        match self {
            Self::Pointer32 | Self::Pointer32be => 4,
            Self::Pointer64 | Self::Pointer64be => 8,
        }
    }

    pub fn get_endian(&self) -> Endian {
        match self {
            Self::Pointer32 | Self::Pointer64 => Endian::Little,
            Self::Pointer32be | Self::Pointer64be => Endian::Big,
        }
    }

    pub fn from_process_bitness(process_bitness: Bitness) -> Self {
        match process_bitness {
            Bitness::Bit32 => Self::Pointer32,
            Bitness::Bit64 => Self::Pointer64,
        }
    }

    pub fn to_data_type_ref(&self) -> DataTypeRef {
        match self {
            Self::Pointer32 => DataTypeRef::new("u32"),
            Self::Pointer32be => DataTypeRef::new("u32be"),
            Self::Pointer64 => DataTypeRef::new("u64"),
            Self::Pointer64be => DataTypeRef::new("u64be"),
        }
    }

    pub fn read_address_value(
        &self,
        data_value: &DataValue,
    ) -> Option<u64> {
        let value_bytes = data_value.get_value_bytes();

        match self {
            Self::Pointer32 => {
                let value_bytes: [u8; 4] = value_bytes.as_slice().try_into().ok()?;

                Some(u32::from_le_bytes(value_bytes) as u64)
            }
            Self::Pointer32be => {
                let value_bytes: [u8; 4] = value_bytes.as_slice().try_into().ok()?;

                Some(u32::from_be_bytes(value_bytes) as u64)
            }
            Self::Pointer64 => {
                let value_bytes: [u8; 8] = value_bytes.as_slice().try_into().ok()?;

                Some(u64::from_le_bytes(value_bytes))
            }
            Self::Pointer64be => {
                let value_bytes: [u8; 8] = value_bytes.as_slice().try_into().ok()?;

                Some(u64::from_be_bytes(value_bytes))
            }
        }
    }
}

impl Display for PointerScanPointerSize {
    fn fmt(
        &self,
        formatter: &mut Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::Pointer32 => write!(formatter, "u32"),
            Self::Pointer32be => write!(formatter, "u32be"),
            Self::Pointer64 => write!(formatter, "u64"),
            Self::Pointer64be => write!(formatter, "u64be"),
        }
    }
}

impl FromStr for PointerScanPointerSize {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string.trim().to_ascii_lowercase().as_str() {
            "4" | "32" | "u32" => Ok(Self::Pointer32),
            "u32be" | "32be" => Ok(Self::Pointer32be),
            "8" | "64" | "u64" => Ok(Self::Pointer64),
            "u64be" | "64be" => Ok(Self::Pointer64be),
            _ => Err(format!("Unsupported pointer size: {string}. Expected one of: 4, 8, u32, u32be, u64, u64be.")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PointerScanPointerSize;
    use crate::structures::data_types::data_type_ref::DataTypeRef;
    use crate::structures::data_values::data_value::DataValue;
    use std::str::FromStr;

    #[test]
    fn pointer_scan_pointer_size_parses_numeric_and_symbolic_values() {
        assert_eq!(PointerScanPointerSize::from_str("4"), Ok(PointerScanPointerSize::Pointer32));
        assert_eq!(PointerScanPointerSize::from_str("u32be"), Ok(PointerScanPointerSize::Pointer32be));
        assert_eq!(PointerScanPointerSize::from_str("u64"), Ok(PointerScanPointerSize::Pointer64));
        assert_eq!(PointerScanPointerSize::from_str("u64be"), Ok(PointerScanPointerSize::Pointer64be));
    }

    #[test]
    fn pointer_scan_pointer_size_reads_address_values() {
        let pointer32_value = DataValue::new(DataTypeRef::new("u32"), 0x1234_u32.to_le_bytes().to_vec());
        let pointer32be_value = DataValue::new(DataTypeRef::new("u32be"), 0x1234_5678_u32.to_be_bytes().to_vec());
        let pointer64_value = DataValue::new(DataTypeRef::new("u64"), 0x1234_5678_9ABC_DEF0_u64.to_le_bytes().to_vec());
        let pointer64be_value = DataValue::new(DataTypeRef::new("u64be"), 0x1234_5678_9ABC_DEF0_u64.to_be_bytes().to_vec());

        assert_eq!(PointerScanPointerSize::Pointer32.read_address_value(&pointer32_value), Some(0x1234));
        assert_eq!(PointerScanPointerSize::Pointer32be.read_address_value(&pointer32be_value), Some(0x1234_5678));
        assert_eq!(
            PointerScanPointerSize::Pointer64.read_address_value(&pointer64_value),
            Some(0x1234_5678_9ABC_DEF0)
        );
        assert_eq!(
            PointerScanPointerSize::Pointer64be.read_address_value(&pointer64be_value),
            Some(0x1234_5678_9ABC_DEF0)
        );
    }
}
