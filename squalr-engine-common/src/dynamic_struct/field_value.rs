use crate::values::data_type::DataType;
use crate::values::data_value::DataValue;
use crate::values::endian::Endian;
use std::cmp::Ordering;
use std::str::FromStr;

pub type FieldMemoryLoadFunc = unsafe fn(&mut FieldValue, *const u8);

/// TODO: This belongs in projects. This does not need to be known nor exist at a common level.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct FieldValue {
    pub data_type: DataType,
    pub data_value: DataValue,
}

impl Eq for FieldValue {}

impl Ord for FieldValue {
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        self.data_value.cmp(&other.data_value)
    }
}

impl Default for FieldValue {
    fn default() -> Self {
        FieldValue {
            data_type: DataType::default(),
            data_value: DataValue::default(),
        }
    }
}

impl FieldValue {
    pub fn new(
        data_type: DataType,
        data_value: DataValue,
    ) -> Self {
        FieldValue { data_type, data_value }
    }

    pub fn get_size_in_bytes(&self) -> u64 {
        self.data_value.get_size_in_bytes()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match (&self.data_type, &self.data_value) {
            (DataType::U8(), DataValue::U8(value)) => value.to_le_bytes().to_vec(),
            (DataType::U16(endian), DataValue::U16(value)) => match endian {
                Endian::Little => value.to_le_bytes().to_vec(),
                Endian::Big => value.to_be_bytes().to_vec(),
            },
            (DataType::U32(endian), DataValue::U32(value)) => match endian {
                Endian::Little => value.to_le_bytes().to_vec(),
                Endian::Big => value.to_be_bytes().to_vec(),
            },
            (DataType::U64(endian), DataValue::U64(value)) => match endian {
                Endian::Little => value.to_le_bytes().to_vec(),
                Endian::Big => value.to_be_bytes().to_vec(),
            },
            (DataType::I8(), DataValue::I8(value)) => value.to_le_bytes().to_vec(),
            (DataType::I16(endian), DataValue::I16(value)) => match endian {
                Endian::Little => value.to_le_bytes().to_vec(),
                Endian::Big => value.to_be_bytes().to_vec(),
            },
            (DataType::I32(endian), DataValue::I32(value)) => match endian {
                Endian::Little => value.to_le_bytes().to_vec(),
                Endian::Big => value.to_be_bytes().to_vec(),
            },
            (DataType::I64(endian), DataValue::I64(value)) => match endian {
                Endian::Little => value.to_le_bytes().to_vec(),
                Endian::Big => value.to_be_bytes().to_vec(),
            },
            (DataType::F32(endian), DataValue::F32(value)) => match endian {
                Endian::Little => value.to_le_bytes().to_vec(),
                Endian::Big => value.to_be_bytes().to_vec(),
            },
            (DataType::F64(endian), DataValue::F64(value)) => match endian {
                Endian::Little => value.to_le_bytes().to_vec(),
                Endian::Big => value.to_be_bytes().to_vec(),
            },
            (DataType::Bytes(_), DataValue::Bytes(bytes)) => bytes.clone(),
            (DataType::BitField(bits), DataValue::BitField { value, .. }) => {
                let total_bytes = ((*bits + 7) / 8) as usize;
                value.iter().take(total_bytes).copied().collect()
            }
            _ => panic!("Mismatched data type and value"),
        }
    }

    pub fn copy_from_bytes(
        &mut self,
        bytes: &[u8],
    ) {
        self.data_value = match self.data_type {
            DataType::U8() => DataValue::U8(bytes[0]),
            DataType::U16(ref endian) => {
                let value = match endian {
                    Endian::Little => u16::from_le_bytes(bytes[0..2].try_into().unwrap()),
                    Endian::Big => u16::from_be_bytes(bytes[0..2].try_into().unwrap()),
                };
                DataValue::U16(value)
            }
            DataType::U32(ref endian) => {
                let value = match endian {
                    Endian::Little => u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
                    Endian::Big => u32::from_be_bytes(bytes[0..4].try_into().unwrap()),
                };
                DataValue::U32(value)
            }
            DataType::U64(ref endian) => {
                let value = match endian {
                    Endian::Little => u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
                    Endian::Big => u64::from_be_bytes(bytes[0..8].try_into().unwrap()),
                };
                DataValue::U64(value)
            }
            DataType::I8() => DataValue::I8(bytes[0] as i8),
            DataType::I16(ref endian) => {
                let value = match endian {
                    Endian::Little => i16::from_le_bytes(bytes[0..2].try_into().unwrap()),
                    Endian::Big => i16::from_be_bytes(bytes[0..2].try_into().unwrap()),
                };
                DataValue::I16(value)
            }
            DataType::I32(ref endian) => {
                let value = match endian {
                    Endian::Little => i32::from_le_bytes(bytes[0..4].try_into().unwrap()),
                    Endian::Big => i32::from_be_bytes(bytes[0..4].try_into().unwrap()),
                };
                DataValue::I32(value)
            }
            DataType::I64(ref endian) => {
                let value = match endian {
                    Endian::Little => i64::from_le_bytes(bytes[0..8].try_into().unwrap()),
                    Endian::Big => i64::from_be_bytes(bytes[0..8].try_into().unwrap()),
                };
                DataValue::I64(value)
            }
            DataType::F32(ref endian) => {
                let value = match endian {
                    Endian::Little => f32::from_le_bytes(bytes[0..4].try_into().unwrap()),
                    Endian::Big => f32::from_be_bytes(bytes[0..4].try_into().unwrap()),
                };
                DataValue::F32(value)
            }
            DataType::F64(ref endian) => {
                let value = match endian {
                    Endian::Little => f64::from_le_bytes(bytes[0..8].try_into().unwrap()),
                    Endian::Big => f64::from_be_bytes(bytes[0..8].try_into().unwrap()),
                };
                DataValue::F64(value)
            }
            DataType::Bytes(_) => DataValue::Bytes(bytes.to_vec()),
            DataType::BitField(bits) => {
                let total_bytes = ((bits + 7) / 8) as usize;
                DataValue::BitField {
                    value: bytes[..total_bytes].to_vec(),
                    bits,
                }
            }
        };
    }
}

impl FromStr for FieldValue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
            return Ok(FieldValue::new(data_type.clone(), data_type.to_default_value()));
        }

        let value_str = value_and_type[0];

        let value = match data_type {
            DataType::U8() => value_str
                .parse::<u8>()
                .map(DataValue::U8)
                .map_err(|e| e.to_string()),
            DataType::U16(_) => value_str
                .parse::<u16>()
                .map(DataValue::U16)
                .map_err(|e| e.to_string()),
            DataType::U32(_) => value_str
                .parse::<u32>()
                .map(DataValue::U32)
                .map_err(|e| e.to_string()),
            DataType::U64(_) => value_str
                .parse::<u64>()
                .map(DataValue::U64)
                .map_err(|e| e.to_string()),
            DataType::I8() => value_str
                .parse::<i8>()
                .map(DataValue::I8)
                .map_err(|e| e.to_string()),
            DataType::I16(_) => value_str
                .parse::<i16>()
                .map(DataValue::I16)
                .map_err(|e| e.to_string()),
            DataType::I32(_) => value_str
                .parse::<i32>()
                .map(DataValue::I32)
                .map_err(|e| e.to_string()),
            DataType::I64(_) => value_str
                .parse::<i64>()
                .map(DataValue::I64)
                .map_err(|e| e.to_string()),
            DataType::F32(_) => value_str
                .parse::<f32>()
                .map(DataValue::F32)
                .map_err(|e| e.to_string()),
            DataType::F64(_) => value_str
                .parse::<f64>()
                .map(DataValue::F64)
                .map_err(|e| e.to_string()),
            DataType::Bytes(_) => Ok(DataValue::Bytes(value_str.as_bytes().to_vec())),
            DataType::BitField(bits) => {
                let bytes = hex::decode(value_str).map_err(|e| e.to_string())?;

                if bytes.len() * 8 < bits as usize {
                    return Err("Not enough bits in bitfield".to_string());
                }

                Ok(DataValue::BitField { value: bytes, bits })
            }
        }?;

        return Ok(FieldValue::new(data_type, value));
    }
}
