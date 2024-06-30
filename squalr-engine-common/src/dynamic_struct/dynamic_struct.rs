use std::collections::HashMap;
use crate::dynamic_struct::to_bytes::ToBytes;

pub enum Endian {
    Little,
    Big,
}

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
    BitField { value: Vec<u8>, bits: usize },
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

pub struct DynamicStruct {
    fields: HashMap<String, FieldValue>,
    word_size: usize,
}

impl DynamicStruct {
    pub fn new(word_size: usize) -> Self {
        DynamicStruct {
            fields: HashMap::new(),
            word_size,
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
        let mut current_byte = 0u8;

        for field in self.fields.values() {
            let field_bytes = field.to_bytes();
            match field {
                FieldValue::BitField { value: _, bits } => {
                    for bit in 0..*bits {
                        let byte_index = (bit_offset + bit) / 8;
                        let bit_index = (bit_offset + bit) % 8;
                        if byte_index >= bytes.len() {
                            bytes.push(0);
                        }
                        let mask = 1 << bit_index;
                        bytes[byte_index] |= field_bytes[bit_index as usize] & mask;
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

        // Pad to the word size
        while bytes.len() % self.word_size != 0 {
            bytes.push(0);
        }

        bytes
    }
}
