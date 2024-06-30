use std::collections::HashMap;
use std::str::FromStr;
use crate::dynamic_struct::to_bytes::ToBytes;

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
    fn to_bytes(&self) -> Vec<u8> {
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

#[derive(Debug)]
pub struct DynamicStruct {
    fields: HashMap<String, FieldValue>,
}

impl DynamicStruct {
    pub fn new() -> Self {
        DynamicStruct {
            fields: HashMap::new(),
        }
    }

    pub fn add_field(&mut self, name: &str, value: FieldValue) {
        self.fields.insert(name.to_string(), value);
    }
}

impl ToBytes for DynamicStruct {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut bit_offset = 0;
        let current_byte = 0u8;

        for field in self.fields.values() {
            let field_bytes = field.to_bytes();
            match field {
                FieldValue::BitField { value: _, bits } => {
                    for bit in 0..*bits {
                        let byte_index = (bit_offset + bit) / 8;
                        let bit_index = (bit_offset + bit) % 8;
                        if byte_index >= bytes.len() as u16 {
                            bytes.push(0);
                        }
                        let mask = 1 << bit_index;
                        bytes[byte_index as usize] |= field_bytes[bit_index as usize] & mask;
                    }
                    bit_offset += *bits;
                }
                _ => {
                    if bit_offset > 0 {
                        // Align to next byte
                        bit_offset = 0;
                    }
                    bytes.extend(field_bytes);
                }
            }
        }

        if bit_offset > 0 {
            bytes.push(current_byte);
        }

        bytes
    }
}

impl FromStr for DynamicStruct {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut dynamic_struct = DynamicStruct::new();
        let fields: Vec<&str> = s.split(';').filter(|&f| !f.is_empty()).collect();

        for field in fields {
            let parts: Vec<&str> = field.split('=').collect();
            if parts.len() != 2 {
                return Err("Invalid field format".to_string());
            }

            let name_and_type: Vec<&str> = parts[0].split(':').collect();
            if name_and_type.len() != 2 {
                return Err("Invalid field name and type format".to_string());
            }

            let field_name = name_and_type[0];
            let field_type_value = format!("{}={}", name_and_type[1], parts[1]);
            let field_value = FieldValue::from_str(&field_type_value)?;

            dynamic_struct.add_field(field_name, field_value);
        }

        Ok(dynamic_struct)
    }
}
