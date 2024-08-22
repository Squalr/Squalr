use crate::values::data_type::DataType;
use std::cmp::Ordering;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum DataValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Bytes(Vec<u8>),
    BitField { value: Vec<u8>, bits: u16 },
}

impl Ord for DataValue {
    fn cmp(
        &self,
        other: &Self
    ) -> Ordering {
        match (self, other) {
            (DataValue::U8(a), DataValue::U8(b)) => a.cmp(b),
            (DataValue::U16(a), DataValue::U16(b)) => a.cmp(b),
            (DataValue::U32(a), DataValue::U32(b)) => a.cmp(b),
            (DataValue::U64(a), DataValue::U64(b)) => a.cmp(b),
            (DataValue::I8(a), DataValue::I8(b)) => a.cmp(b),
            (DataValue::I16(a), DataValue::I16(b)) => a.cmp(b),
            (DataValue::I32(a), DataValue::I32(b)) => a.cmp(b),
            (DataValue::I64(a), DataValue::I64(b)) => a.cmp(b),
            (DataValue::F32(a), DataValue::F32(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            (DataValue::F64(a), DataValue::F64(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            (DataValue::Bytes(a), DataValue::Bytes(b)) => a.cmp(b),
            (DataValue::BitField { value: a, bits: bits_a }, DataValue::BitField { value: b, bits: bits_b }) => {
                a.cmp(b).then_with(|| bits_a.cmp(bits_b))
            }
            _ => panic!("Comparing unsupported types."),
        }
    }
}

impl Eq for DataValue {}

impl Default for DataValue {
    fn default() -> Self {
        DataValue::U8(0)
    }
}

impl DataValue {
    pub fn size_in_bytes(
        &self
    ) -> u64 {
        match self {
            DataValue::U8(_) => std::mem::size_of::<u8>() as u64,
            DataValue::U16(_) => std::mem::size_of::<u16>() as u64,
            DataValue::U32(_) => std::mem::size_of::<u32>() as u64,
            DataValue::U64(_) => std::mem::size_of::<u64>() as u64,
            DataValue::I8(_) => std::mem::size_of::<i8>() as u64,
            DataValue::I16(_) => std::mem::size_of::<i16>() as u64,
            DataValue::I32(_) => std::mem::size_of::<i32>() as u64,
            DataValue::I64(_) => std::mem::size_of::<i64>() as u64,
            DataValue::F32(_) => std::mem::size_of::<f32>() as u64,
            DataValue::F64(_) => std::mem::size_of::<f64>() as u64,
            DataValue::Bytes(ref bytes) => bytes.len() as u64,
            DataValue::BitField { bits, .. } => ((*bits + 7) / 8) as u64,
        }
    }

    pub fn as_u8(
        &self
    ) -> Option<u8> {
        return match self {
            DataValue::U8(v) => Some(*v),
            _ => None,
        };
    }

    pub fn as_i8(
        &self
    ) -> Option<i8> {
        return match self {
            DataValue::I8(v) => Some(*v),
            _ => None,
        };
    }

    pub fn as_u16(
        &self
    ) -> Option<u16> {
        return match self {
            DataValue::U16(v) => Some(*v),
            _ => None,
        };
    }

    pub fn as_i16(
        &self
    ) -> Option<i16> {
        return match self {
            DataValue::I16(v) => Some(*v),
            _ => None,
        };
    }

    pub fn as_u32(
        &self
    ) -> Option<u32> {
        return match self {
            DataValue::U32(v) => Some(*v),
            _ => None,
        };
    }

    pub fn as_i32(
        &self
    ) -> Option<i32> {
        return match self {
            DataValue::I32(v) => Some(*v),
            _ => None,
        };
    }

    pub fn as_u64(
        &self
    ) -> Option<u64> {
        return match self {
            DataValue::U64(v) => Some(*v),
            _ => None,
        };
    }

    pub fn as_i64(
        &self
    ) -> Option<i64> {
        return match self {
            DataValue::I64(v) => Some(*v),
            _ => None,
        };
    }

    pub fn as_f32(
        &self
    ) -> Option<f32> {
        return match self {
            DataValue::F32(v) => Some(*v),
            _ => None,
        };
    }

    pub fn as_f64(
        &self
    ) -> Option<f64> {
        return match self {
            DataValue::F64(v) => Some(*v),
            _ => None,
        };
    }

    pub fn as_bytes(
        &self
    ) -> Option<&[u8]> {
        return match self {
            DataValue::Bytes(ref v) => Some(v.as_slice()),
            _ => None,
        };
    }

    pub fn as_bitfield(
        &self
    ) -> Option<(&[u8], u16)> {
        return match self {
            DataValue::BitField { ref value, bits } => Some((value.as_slice(), *bits)),
            _ => None,
        };
    }

    pub fn as_ptr(&self) -> *const u8 {
        match self {
            DataValue::U8(v) => v as *const u8,
            DataValue::U16(v) => v as *const u16 as *const u8,
            DataValue::U32(v) => v as *const u32 as *const u8,
            DataValue::U64(v) => v as *const u64 as *const u8,
            DataValue::I8(v) => v as *const i8 as *const u8,
            DataValue::I16(v) => v as *const i16 as *const u8,
            DataValue::I32(v) => v as *const i32 as *const u8,
            DataValue::I64(v) => v as *const i64 as *const u8,
            DataValue::F32(v) => v as *const f32 as *const u8,
            DataValue::F64(v) => v as *const f64 as *const u8,
            DataValue::Bytes(v) => v.as_ptr(),
            DataValue::BitField { value, .. } => value.as_ptr(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            DataValue::U8(v) => v.to_le_bytes().to_vec(),
            DataValue::U16(v) => v.to_le_bytes().to_vec(),
            DataValue::U32(v) => v.to_le_bytes().to_vec(),
            DataValue::U64(v) => v.to_le_bytes().to_vec(),
            DataValue::I8(v) => v.to_le_bytes().to_vec(),
            DataValue::I16(v) => v.to_le_bytes().to_vec(),
            DataValue::I32(v) => v.to_le_bytes().to_vec(),
            DataValue::I64(v) => v.to_le_bytes().to_vec(),
            DataValue::F32(v) => v.to_le_bytes().to_vec(),
            DataValue::F64(v) => v.to_le_bytes().to_vec(),
            DataValue::Bytes(v) => v.clone(),
            DataValue::BitField { value, .. } => value.clone(),
        }
    }
}

impl FromStr for DataValue {
    type Err = String;

    fn from_str(
        s: &str
    ) -> Result<Self, Self::Err> {
        let value_and_type: Vec<&str> = s.split('=').collect();
        let has_value = value_and_type.len() == 2;

        if value_and_type.len() != 1 && value_and_type.len() != 2 {
            return Err("Invalid field type and value format".to_string());
        }

        let data_type = if has_value {
            DataType::from_str(value_and_type[1])?
        } else {
            DataType::from_str(value_and_type[0])?
        };

        if !has_value {
            return Ok(data_type.to_default_value());
        }

        let value_str = value_and_type[0];

        let value = match data_type {
            DataType::U8() => value_str.parse::<u8>().map(DataValue::U8).map_err(|e| e.to_string()),
            DataType::U16(_) => value_str.parse::<u16>().map(DataValue::U16).map_err(|e| e.to_string()),
            DataType::U32(_) => value_str.parse::<u32>().map(DataValue::U32).map_err(|e| e.to_string()),
            DataType::U64(_) => value_str.parse::<u64>().map(DataValue::U64).map_err(|e| e.to_string()),
            DataType::I8() => value_str.parse::<i8>().map(DataValue::I8).map_err(|e| e.to_string()),
            DataType::I16(_) => value_str.parse::<i16>().map(DataValue::I16).map_err(|e| e.to_string()),
            DataType::I32(_) => value_str.parse::<i32>().map(DataValue::I32).map_err(|e| e.to_string()),
            DataType::I64(_) => value_str.parse::<i64>().map(DataValue::I64).map_err(|e| e.to_string()),
            DataType::F32(_) => value_str.parse::<f32>().map(DataValue::F32).map_err(|e| e.to_string()),
            DataType::F64(_) => value_str.parse::<f64>().map(DataValue::F64).map_err(|e| e.to_string()),
            DataType::Bytes(_) => Ok(DataValue::Bytes(value_str.as_bytes().to_vec())),
            DataType::BitField(bits) => {
                let bytes = hex::decode(value_str).map_err(|e| e.to_string())?;
                if bytes.len() * 8 < bits as usize {
                    return Err("Not enough bits in bitfield".to_string());
                }
                Ok(DataValue::BitField { value: bytes, bits })
            }
        }?;

        return Ok(value);
    }
}
