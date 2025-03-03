use crate::structures::dynamic_struct::dynamic_struct_field::DynamicStructField;
use crate::structures::dynamic_struct::to_bytes::ToBytes;
use crate::structures::values::data_value::DataValue;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

// TODO: Think over whether this belongs in common or projects.
// AnonymousValue, DataValue, etc may cover common use cases.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynamicStruct {
    fields: Vec<DynamicStructField>,
}

impl DynamicStruct {
    pub fn new() -> Self {
        DynamicStruct { fields: vec![] }
    }

    pub fn add_field(
        &mut self,
        struct_field: DynamicStructField,
    ) {
        self.fields.push(struct_field);
    }

    pub fn get_size_in_bytes(&self) -> u64 {
        self.fields.iter().map(|field| field.get_size_in_bytes()).sum()
    }

    pub fn copy_from_bytes(
        &mut self,
        bytes: &[u8],
    ) {
        let mut offset = 0;
        for field in &mut self.fields {
            let size = field.get_size_in_bytes() as usize;
            field.copy_from_bytes(&bytes[offset..offset + size]);
            offset += size;
        }
    }
}

impl ToBytes for DynamicStruct {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        let mut bit_offset = 0;
        let current_byte = 0u8;

        for field in &self.fields {
            let field_bytes = field.to_bytes();
            match field.data_value {
                DataValue::BitField { value: _, bits } => {
                    for bit in 0..bits {
                        let byte_index = (bit_offset + bit) / 8;
                        let bit_index = (bit_offset + bit) % 8;
                        if byte_index >= bytes.len() as u16 {
                            bytes.push(0);
                        }
                        let mask = 1 << bit_index;
                        bytes[byte_index as usize] |= field_bytes[bit_index as usize] & mask;
                    }
                    bit_offset += bits;
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
            let struct_field = DynamicStructField::from_str(&field)?;

            dynamic_struct.add_field(struct_field);
        }

        Ok(dynamic_struct)
    }
}
