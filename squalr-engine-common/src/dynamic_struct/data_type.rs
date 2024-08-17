use crate::dynamic_struct::endian::Endian;
use crate::dynamic_struct::field_value::FieldValue;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataType {
    U8,
    U16(Endian),
    U32(Endian),
    U64(Endian),
    I8,
    I16(Endian),
    I32(Endian),
    I64(Endian),
    F32(Endian),
    F64(Endian),
    Bytes,
    BitField(u16),
}

impl Default for DataType {
    fn default() -> Self {
        DataType::U8
    }
}

impl DataType {
    pub fn size_in_bytes(&self) -> u64 {
        match self {
            DataType::U8 => std::mem::size_of::<u8>() as u64,
            DataType::U16(_) => std::mem::size_of::<u16>() as u64,
            DataType::U32(_) => std::mem::size_of::<u32>() as u64,
            DataType::U64(_) => std::mem::size_of::<u64>() as u64,
            DataType::I8 => std::mem::size_of::<i8>() as u64,
            DataType::I16(_) => std::mem::size_of::<i16>() as u64,
            DataType::I32(_) => std::mem::size_of::<i32>() as u64,
            DataType::I64(_) => std::mem::size_of::<i64>() as u64,
            DataType::F32(_) => std::mem::size_of::<f32>() as u64,
            DataType::F64(_) => std::mem::size_of::<f64>() as u64,
            DataType::Bytes => 0, // Size can vary
            DataType::BitField(bits) => ((*bits + 7) / 8) as u64,
        }
    }

    pub fn to_default_value(&self) -> FieldValue {
        match self {
            DataType::U8 => FieldValue::U8(0),
            DataType::U16(_) => FieldValue::U16(0, Endian::Little),
            DataType::U32(_) => FieldValue::U32(0, Endian::Little),
            DataType::U64(_) => FieldValue::U64(0, Endian::Little),
            DataType::I8 => FieldValue::I8(0),
            DataType::I16(_) => FieldValue::I16(0, Endian::Little),
            DataType::I32(_) => FieldValue::I32(0, Endian::Little),
            DataType::I64(_) => FieldValue::I64(0, Endian::Little),
            DataType::F32(_) => FieldValue::F32(0.0, Endian::Little),
            DataType::F64(_) => FieldValue::F64(0.0, Endian::Little),
            DataType::Bytes => FieldValue::Bytes(vec![]),
            DataType::BitField(bits) => FieldValue::BitField {
                value: vec![0; ((*bits + 7) / 8) as usize],
                bits: *bits,
            },
        }
    }
}

impl FromStr for DataType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 1 && parts.len() != 2 {
            return Err("Invalid format".to_string());
        }

        let type_str = parts[0];
        let endian = if parts.len() == 2 {
            match parts[1] {
                "le" => Endian::Little,
                "be" => Endian::Big,
                _ => return Err("Invalid endian format".to_string()),
            }
        } else {
            Endian::Little
        };

        match type_str {
            "u8" => Ok(DataType::U8),
            "u16" => Ok(DataType::U16(endian)),
            "u32" => Ok(DataType::U32(endian)),
            "u64" => Ok(DataType::U64(endian)),
            "i8" => Ok(DataType::I8),
            "i16" => Ok(DataType::I16(endian)),
            "i32" => Ok(DataType::I32(endian)),
            "i64" => Ok(DataType::I64(endian)),
            "f32" => Ok(DataType::F32(endian)),
            "f64" => Ok(DataType::F64(endian)),
            "bytes" => Ok(DataType::Bytes),
            other if other.starts_with("bitfield") => {
                let bits_str = other.trim_start_matches("bitfield");
                let bits: u16 = bits_str
                    .parse()
                    .map_err(|_| "Invalid bitfield format".to_string())?;
                Ok(DataType::BitField(bits))
            }
            _ => Err("Unknown data type".to_string()),
        }
    }
}