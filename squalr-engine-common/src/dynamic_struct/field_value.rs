use std::str::FromStr;

#[derive(Debug)]
pub enum Endian {
    Little,
    Big,
}

#[derive(Debug)]
pub enum FieldValue {
    U8(u8),
    U16(u16, Endian),
    U32(u32, Endian),
    U64(u64, Endian),
    I8(i8),
    I16(i16, Endian),
    I32(i32, Endian),
    I64(i64, Endian),
    F32(f32, Endian),
    F64(f64, Endian),
    Bytes(Vec<u8>),
    BitField { value: Vec<u8>, bits: u16 },
}

impl FieldValue {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            FieldValue::U8(value) => value.to_le_bytes().to_vec(),
            FieldValue::U16(value, endian) => match endian {
                Endian::Little => value.to_le_bytes().to_vec(),
                Endian::Big => value.to_be_bytes().to_vec(),
            },
            FieldValue::U32(value, endian) => match endian {
                Endian::Little => value.to_le_bytes().to_vec(),
                Endian::Big => value.to_be_bytes().to_vec(),
            },
            FieldValue::U64(value, endian) => match endian {
                Endian::Little => value.to_le_bytes().to_vec(),
                Endian::Big => value.to_be_bytes().to_vec(),
            },
            FieldValue::I8(value) => value.to_le_bytes().to_vec(),
            FieldValue::I16(value, endian) => match endian {
                Endian::Little => value.to_le_bytes().to_vec(),
                Endian::Big => value.to_be_bytes().to_vec(),
            },
            FieldValue::I32(value, endian) => match endian {
                Endian::Little => value.to_le_bytes().to_vec(),
                Endian::Big => value.to_be_bytes().to_vec(),
            },
            FieldValue::I64(value, endian) => match endian {
                Endian::Little => value.to_le_bytes().to_vec(),
                Endian::Big => value.to_be_bytes().to_vec(),
            },
            FieldValue::F32(value, endian) => match endian {
                Endian::Little => value.to_le_bytes().to_vec(),
                Endian::Big => value.to_be_bytes().to_vec(),
            },
            FieldValue::F64(value, endian) => match endian {
                Endian::Little => value.to_le_bytes().to_vec(),
                Endian::Big => value.to_be_bytes().to_vec(),
            },
            FieldValue::Bytes(bytes) => bytes.clone(),
            FieldValue::BitField { value, bits } => {
                let mut result = Vec::new();
                let total_bytes = ((*bits + 7) / 8) as usize;
                for i in 0..total_bytes {
                    result.push(value[i]);
                }
                result
            }
        }
    }

    pub fn size_in_bytes(&self) -> usize {
        match self {
            FieldValue::U8(_) => std::mem::size_of::<u8>(),
            FieldValue::U16(_, _) => std::mem::size_of::<u16>(),
            FieldValue::U32(_, _) => std::mem::size_of::<u32>(),
            FieldValue::U64(_, _) => std::mem::size_of::<u64>(),
            FieldValue::I8(_) => std::mem::size_of::<i8>(),
            FieldValue::I16(_, _) => std::mem::size_of::<i16>(),
            FieldValue::I32(_, _) => std::mem::size_of::<i32>(),
            FieldValue::I64(_, _) => std::mem::size_of::<i64>(),
            FieldValue::F32(_, _) => std::mem::size_of::<f32>(),
            FieldValue::F64(_, _) => std::mem::size_of::<f64>(),
            FieldValue::Bytes(ref bytes) => bytes.len(),
            FieldValue::BitField { value, bits } => ((*bits + 7) / 8) as usize,
        }
    }

    pub fn copy_from_bytes(&mut self, bytes: &[u8]) {
        match self {
            FieldValue::U8(ref mut value) => *value = bytes[0],
            FieldValue::U16(ref mut value, endian) => {
                *value = match endian {
                    Endian::Little => u16::from_le_bytes([bytes[0], bytes[1]]),
                    Endian::Big => u16::from_be_bytes([bytes[0], bytes[1]]),
                };
            }
            FieldValue::U32(ref mut value, endian) => {
                *value = match endian {
                    Endian::Little => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                    Endian::Big => u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                };
            }
            FieldValue::U64(ref mut value, endian) => {
                *value = match endian {
                    Endian::Little => u64::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]
                    ]),
                    Endian::Big => u64::from_be_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]
                    ]),
                };
            }
            FieldValue::I8(ref mut value) => *value = bytes[0] as i8,
            FieldValue::I16(ref mut value, endian) => {
                *value = match endian {
                    Endian::Little => i16::from_le_bytes([bytes[0], bytes[1]]),
                    Endian::Big => i16::from_be_bytes([bytes[0], bytes[1]]),
                };
            }
            FieldValue::I32(ref mut value, endian) => {
                *value = match endian {
                    Endian::Little => i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                    Endian::Big => i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                };
            }
            FieldValue::I64(ref mut value, endian) => {
                *value = match endian {
                    Endian::Little => i64::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]
                    ]),
                    Endian::Big => i64::from_be_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]
                    ]),
                };
            }
            FieldValue::F32(ref mut value, endian) => {
                let bits = match endian {
                    Endian::Little => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                    Endian::Big => u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                };
                *value = f32::from_bits(bits);
            }
            FieldValue::F64(ref mut value, endian) => {
                let bits = match endian {
                    Endian::Little => u64::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]
                    ]),
                    Endian::Big => u64::from_be_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]
                    ]),
                };
                *value = f64::from_bits(bits);
            }
            FieldValue::Bytes(ref mut value) => value.copy_from_slice(bytes),
            FieldValue::BitField { ref mut value, bits } => {
                let total_bytes = ((*bits + 7) / 8) as usize;
                value[..total_bytes].copy_from_slice(&bytes[..total_bytes]);
            }
        }
    }
}

impl FromStr for FieldValue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let type_and_value: Vec<&str> = s.split('=').collect();
        if type_and_value.len() != 2 {
            return Err("Invalid field type and value format".to_string());
        }

        let field_type = type_and_value[0];
        let value_str = type_and_value[1];

        match field_type {
            "u8" => value_str.parse::<u8>().map(FieldValue::U8).map_err(|e| e.to_string()),
            "u16" => value_str.parse::<u16>().map(|v| FieldValue::U16(v, Endian::Little)).map_err(|e| e.to_string()),
            "u32" => value_str.parse::<u32>().map(|v| FieldValue::U32(v, Endian::Little)).map_err(|e| e.to_string()),
            "u64" => value_str.parse::<u64>().map(|v| FieldValue::U64(v, Endian::Little)).map_err(|e| e.to_string()),
            "i8" => value_str.parse::<i8>().map(FieldValue::I8).map_err(|e| e.to_string()),
            "i16" => value_str.parse::<i16>().map(|v| FieldValue::I16(v, Endian::Little)).map_err(|e| e.to_string()),
            "i32" => value_str.parse::<i32>().map(|v| FieldValue::I32(v, Endian::Little)).map_err(|e| e.to_string()),
            "i64" => value_str.parse::<i64>().map(|v| FieldValue::I64(v, Endian::Little)).map_err(|e| e.to_string()),
            "f32" => value_str.parse::<f32>().map(|v| FieldValue::F32(v, Endian::Little)).map_err(|e| e.to_string()),
            "f64" => value_str.parse::<f64>().map(|v| FieldValue::F64(v, Endian::Little)).map_err(|e| e.to_string()),
            "bytes" => Ok(FieldValue::Bytes(value_str.as_bytes().to_vec())),
            _ => Err("Unknown field type".to_string()),
        }
    }
}
