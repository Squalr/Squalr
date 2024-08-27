use crate::dynamic_struct::field_value::FieldValue;
use crate::dynamic_struct::to_bytes::ToBytes;
use crate::values::data_value::DataValue;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug)]
pub struct DynamicStruct {
    fields: HashMap<String, FieldValue>,
}

/// TODO: This belongs in projects. This does not need to be known nor exist at a common level.
impl DynamicStruct {
    pub fn new() -> Self {
        DynamicStruct { fields: HashMap::new() }
    }

    pub fn add_field(
        &mut self,
        name: &str,
        value: FieldValue,
    ) {
        self.fields.insert(name.to_string(), value);
    }

    pub fn get_size_in_bytes(&self) -> u64 {
        self.fields
            .values()
            .map(|field| field.get_size_in_bytes())
            .sum()
    }

    pub fn copy_from_bytes(
        &mut self,
        bytes: &[u8],
    ) {
        let mut offset = 0;
        for field in self.fields.values_mut() {
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

        for field in self.fields.values() {
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
