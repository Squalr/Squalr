use crate::values::data_value::DataValue;
use crate::values::endian::Endian;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DataType {
    U8(),
    U16(Endian),
    U32(Endian),
    U64(Endian),
    I8(),
    I16(Endian),
    I32(Endian),
    I64(Endian),
    F32(Endian),
    F64(Endian),
    Bytes(u64),
    BitField(u16),
}

impl Default for DataType {
    fn default() -> Self {
        DataType::U8()
    }
}

impl DataType {
    pub fn get_size_in_bytes(&self) -> u64 {
        match self {
            DataType::U8() => std::mem::size_of::<u8>() as u64,
            DataType::U16(_) => std::mem::size_of::<u16>() as u64,
            DataType::U32(_) => std::mem::size_of::<u32>() as u64,
            DataType::U64(_) => std::mem::size_of::<u64>() as u64,
            DataType::I8() => std::mem::size_of::<i8>() as u64,
            DataType::I16(_) => std::mem::size_of::<i16>() as u64,
            DataType::I32(_) => std::mem::size_of::<i32>() as u64,
            DataType::I64(_) => std::mem::size_of::<i64>() as u64,
            DataType::F32(_) => std::mem::size_of::<f32>() as u64,
            DataType::F64(_) => std::mem::size_of::<f64>() as u64,
            DataType::Bytes(size) => *size,
            DataType::BitField(bits) => ((*bits + 7) / 8) as u64,
        }
    }

    pub fn get_endian(&self) -> Endian {
        match self {
            DataType::U8() => Endian::default(),
            DataType::U16(endian) => endian.clone(),
            DataType::U32(endian) => endian.clone(),
            DataType::U64(endian) => endian.clone(),
            DataType::I8() => Endian::default(),
            DataType::I16(endian) => endian.clone(),
            DataType::I32(endian) => endian.clone(),
            DataType::I64(endian) => endian.clone(),
            DataType::F32(endian) => endian.clone(),
            DataType::F64(endian) => endian.clone(),
            DataType::Bytes(_) => Endian::default(),
            DataType::BitField(_) => Endian::default(),
        }
    }

    pub fn to_default_value(&self) -> DataValue {
        match self {
            DataType::U8() => DataValue::U8(0),
            DataType::U16(_) => DataValue::U16(0),
            DataType::U32(_) => DataValue::U32(0),
            DataType::U64(_) => DataValue::U64(0),
            DataType::I8() => DataValue::I8(0),
            DataType::I16(_) => DataValue::I16(0),
            DataType::I32(_) => DataValue::I32(0),
            DataType::I64(_) => DataValue::I64(0),
            DataType::F32(_) => DataValue::F32(0.0),
            DataType::F64(_) => DataValue::F64(0.0),
            DataType::Bytes(size) => DataValue::Bytes(vec![0; *size as usize]),
            DataType::BitField(bits) => DataValue::BitField {
                value: vec![0; ((*bits + 7) / 8) as usize],
                bits: *bits,
            },
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            DataType::U8() => write!(f, "u8"),
            DataType::U16(endian) => write!(f, "u16:{}", endian),
            DataType::U32(endian) => write!(f, "u32:{}", endian),
            DataType::U64(endian) => write!(f, "u64:{}", endian),
            DataType::I8() => write!(f, "i8"),
            DataType::I16(endian) => write!(f, "i16:{}", endian),
            DataType::I32(endian) => write!(f, "i32:{}", endian),
            DataType::I64(endian) => write!(f, "i64:{}", endian),
            DataType::F32(endian) => write!(f, "f32:{}", endian),
            DataType::F64(endian) => write!(f, "f64:{}", endian),
            DataType::Bytes(size) => write!(f, "bytes:{}", size),
            DataType::BitField(bits) => write!(f, "bitfield{}", bits),
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

        match type_str {
            "bytes" => {
                if parts.len() == 2 {
                    let num_bytes: u64 = parts[1]
                        .parse()
                        .map_err(|_| "Invalid byte length format".to_string())?;
                    Ok(DataType::Bytes(num_bytes))
                } else {
                    Err("Bytes type requires a byte length".to_string())
                }
            }
            _ => {
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
                    "u8" => Ok(DataType::U8()),
                    "u16" => Ok(DataType::U16(endian)),
                    "u32" => Ok(DataType::U32(endian)),
                    "u64" => Ok(DataType::U64(endian)),
                    "i8" => Ok(DataType::I8()),
                    "i16" => Ok(DataType::I16(endian)),
                    "i32" => Ok(DataType::I32(endian)),
                    "i64" => Ok(DataType::I64(endian)),
                    "f32" => Ok(DataType::F32(endian)),
                    "f64" => Ok(DataType::F64(endian)),
                    other if other.starts_with("bitfield") => {
                        let bits_str = other.trim_start_matches("bitfield");
                        let bits: u16 = bits_str
                            .parse()
                            .map_err(|_| "Invalid bitfield format".to_string())?;
                        Ok(DataType::BitField(bits))
                    }
                    _ => Err("Unsupported type.".to_string()),
                }
            }
        }
    }
}
