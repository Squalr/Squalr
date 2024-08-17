use crate::dynamic_struct::data_type::DataType;
use crate::dynamic_struct::endian::Endian;
use std::str::FromStr;
use std::cmp::Ordering;
use std::ptr;

pub type FieldMemoryLoadFunc = unsafe fn(&mut FieldValue, *const u8);

#[derive(Debug, Clone, PartialEq, PartialOrd)]
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

impl Eq for FieldValue {}

impl Ord for FieldValue {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (FieldValue::U8(a), FieldValue::U8(b)) => a.cmp(b),
            (FieldValue::U16(a, _), FieldValue::U16(b, _)) => a.cmp(b),
            (FieldValue::U32(a, _), FieldValue::U32(b, _)) => a.cmp(b),
            (FieldValue::U64(a, _), FieldValue::U64(b, _)) => a.cmp(b),
            (FieldValue::I8(a), FieldValue::I8(b)) => a.cmp(b),
            (FieldValue::I16(a, _), FieldValue::I16(b, _)) => a.cmp(b),
            (FieldValue::I32(a, _), FieldValue::I32(b, _)) => a.cmp(b),
            (FieldValue::I64(a, _), FieldValue::I64(b, _)) => a.cmp(b),
            (FieldValue::F32(a, _), FieldValue::F32(b, _)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            (FieldValue::F64(a, _), FieldValue::F64(b, _)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            (FieldValue::Bytes(a), FieldValue::Bytes(b)) => a.cmp(b),
            (FieldValue::BitField { value: a, bits: bits_a }, FieldValue::BitField { value: b, bits: bits_b }) => {
                a.cmp(b).then_with(|| bits_a.cmp(bits_b))
            }
            _ => panic!("Comparing unsupported types."),
        }
    }
}

impl Default for FieldValue {
    fn default() -> Self {
        FieldValue::U8(0)
    }
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
                let mut result = vec![];
                let total_bytes = ((*bits + 7) / 8) as usize;

                for index in 0..total_bytes {
                    result.push(value[index]);
                }
                
                result
            }
        }
    }

    pub fn size_in_bytes(&self) -> u64 {
        match self {
            FieldValue::U8(_) => std::mem::size_of::<u8>() as u64,
            FieldValue::U16(_, _) => std::mem::size_of::<u16>() as u64,
            FieldValue::U32(_, _) => std::mem::size_of::<u32>() as u64,
            FieldValue::U64(_, _) => std::mem::size_of::<u64>() as u64,
            FieldValue::I8(_) => std::mem::size_of::<i8>() as u64,
            FieldValue::I16(_, _) => std::mem::size_of::<i16>() as u64,
            FieldValue::I32(_, _) => std::mem::size_of::<i32>() as u64,
            FieldValue::I64(_, _) => std::mem::size_of::<i64>() as u64,
            FieldValue::F32(_, _) => std::mem::size_of::<f32>() as u64,
            FieldValue::F64(_, _) => std::mem::size_of::<f64>() as u64,
            FieldValue::Bytes(ref bytes) => bytes.len() as u64,
            FieldValue::BitField { bits, .. } => ((*bits + 7) / 8) as u64,
        }
    }

    pub fn copy_from_bytes(&mut self, bytes: &[u8]) {
        let value_ptr = bytes.as_ptr();
        let load_fn = self.get_load_memory_function_ptr();

        unsafe {
            load_fn(self, value_ptr);
        }
    }

    pub fn get_load_memory_function_ptr(&self) -> FieldMemoryLoadFunc {
        match self {
            FieldValue::U8(_) => Self::load_u8,
            FieldValue::I8(_) => Self::load_i8,
            FieldValue::U16(_, Endian::Little) => Self::load_u16_le,
            FieldValue::U16(_, Endian::Big) => Self::load_u16_be,
            FieldValue::I16(_, Endian::Little) => Self::load_i16_le,
            FieldValue::I16(_, Endian::Big) => Self::load_i16_be,
            FieldValue::U32(_, Endian::Little) => Self::load_u32_le,
            FieldValue::U32(_, Endian::Big) => Self::load_u32_be,
            FieldValue::I32(_, Endian::Little) => Self::load_i32_le,
            FieldValue::I32(_, Endian::Big) => Self::load_i32_be,
            FieldValue::U64(_, Endian::Little) => Self::load_u64_le,
            FieldValue::U64(_, Endian::Big) => Self::load_u64_be,
            FieldValue::I64(_, Endian::Little) => Self::load_i64_le,
            FieldValue::I64(_, Endian::Big) => Self::load_i64_be,
            FieldValue::F32(_, Endian::Little) => Self::load_f32_le,
            FieldValue::F32(_, Endian::Big) => Self::load_f32_be,
            FieldValue::F64(_, Endian::Little) => Self::load_f64_le,
            FieldValue::F64(_, Endian::Big) => Self::load_f64_be,
            FieldValue::Bytes(_) => Self::load_bytes,
            FieldValue::BitField { .. } => Self::load_bitfield,
        }
    }

    unsafe fn load_u8(field: &mut FieldValue, value_ptr: *const u8) {
        if let FieldValue::U8(ref mut value) = *field {
            *value = *value_ptr;
        }
    }
    
    unsafe fn load_i8(field: &mut FieldValue, value_ptr: *const u8) {
        if let FieldValue::I8(ref mut value) = *field {
            *value = *value_ptr as i8;
        }
    }
    
    unsafe fn load_u16_le(field: &mut FieldValue, value_ptr: *const u8) {
        if let FieldValue::U16(ref mut value, _) = *field {
            let bytes = &*(value_ptr as *const [u8; 2]);
            *value = u16::from_le_bytes(*bytes);
        }
    }
    
    unsafe fn load_u16_be(field: &mut FieldValue, value_ptr: *const u8) {
        if let FieldValue::U16(ref mut value, _) = *field {
            let bytes = &*(value_ptr as *const [u8; 2]);
            *value = u16::from_be_bytes(*bytes);
        }
    }
    
    unsafe fn load_i16_le(field: &mut FieldValue, value_ptr: *const u8) {
        if let FieldValue::I16(ref mut value, _) = *field {
            let bytes = &*(value_ptr as *const [u8; 2]);
            *value = i16::from_le_bytes(*bytes);
        }
    }
    
    unsafe fn load_i16_be(field: &mut FieldValue, value_ptr: *const u8) {
        if let FieldValue::I16(ref mut value, _) = *field {
            let bytes = &*(value_ptr as *const [u8; 2]);
            *value = i16::from_be_bytes(*bytes);
        }
    }

    unsafe fn load_u32_le(field: &mut FieldValue, value_ptr: *const u8) {
        if let FieldValue::U32(ref mut value, _) = *field {
            let bytes = &*(value_ptr as *const [u8; 4]);
            *value = u32::from_le_bytes(*bytes);
        }
    }
    
    unsafe fn load_u32_be(field: &mut FieldValue, value_ptr: *const u8) {
        if let FieldValue::U32(ref mut value, _) = *field {
            let bytes = &*(value_ptr as *const [u8; 4]);
            *value = u32::from_be_bytes(*bytes);
        }
    }
    
    unsafe fn load_i32_le(field: &mut FieldValue, value_ptr: *const u8) {
        if let FieldValue::I32(ref mut value, _) = *field {
            let bytes = &*(value_ptr as *const [u8; 4]);
            *value = i32::from_le_bytes(*bytes);
        }
    }
    
    unsafe fn load_i32_be(field: &mut FieldValue, value_ptr: *const u8) {
        if let FieldValue::I32(ref mut value, _) = *field {
            let bytes = &*(value_ptr as *const [u8; 4]);
            *value = i32::from_be_bytes(*bytes);
        }
    }
    
    unsafe fn load_u64_le(field: &mut FieldValue, value_ptr: *const u8) {
        if let FieldValue::U64(ref mut value, _) = *field {
            let bytes = &*(value_ptr as *const [u8; 8]);
            *value = u64::from_le_bytes(*bytes);
        }
    }
    
    unsafe fn load_u64_be(field: &mut FieldValue, value_ptr: *const u8) {
        if let FieldValue::U64(ref mut value, _) = *field {
            let bytes = &*(value_ptr as *const [u8; 8]);
            *value = u64::from_be_bytes(*bytes);
        }
    }
    
    unsafe fn load_i64_le(field: &mut FieldValue, value_ptr: *const u8) {
        if let FieldValue::I64(ref mut value, _) = *field {
            let bytes = &*(value_ptr as *const [u8; 8]);
            *value = i64::from_le_bytes(*bytes);
        }
    }
    
    unsafe fn load_i64_be(field: &mut FieldValue, value_ptr: *const u8) {
        if let FieldValue::I64(ref mut value, _) = *field {
            let bytes = &*(value_ptr as *const [u8; 8]);
            *value = i64::from_be_bytes(*bytes);
        }
    }
    
    unsafe fn load_f32_le(field: &mut FieldValue, value_ptr: *const u8) {
        if let FieldValue::F32(ref mut value, _) = *field {
            let bytes = &*(value_ptr as *const [u8; 4]);
            let bits = u32::from_le_bytes(*bytes);
            *value = f32::from_bits(bits);
        }
    }
    
    unsafe fn load_f32_be(field: &mut FieldValue, value_ptr: *const u8) {
        if let FieldValue::F32(ref mut value, _) = *field {
            let bytes = &*(value_ptr as *const [u8; 4]);
            let bits = u32::from_be_bytes(*bytes);
            *value = f32::from_bits(bits);
        }
    }
    
    unsafe fn load_f64_le(field: &mut FieldValue, value_ptr: *const u8) {
        if let FieldValue::F64(ref mut value, _) = *field {
            let bytes = &*(value_ptr as *const [u8; 8]);
            let bits = u64::from_le_bytes(*bytes);
            *value = f64::from_bits(bits);
        }
    }
    
    unsafe fn load_f64_be(field: &mut FieldValue, value_ptr: *const u8) {
        if let FieldValue::F64(ref mut value, _) = *field {
            let bytes = &*(value_ptr as *const [u8; 8]);
            let bits = u64::from_be_bytes(*bytes);
            *value = f64::from_bits(bits);
        }
    }
    
    unsafe fn load_bytes(field: &mut FieldValue, value_ptr: *const u8) {
        if let FieldValue::Bytes(ref mut value) = *field {
            ptr::copy_nonoverlapping(value_ptr, value.as_mut_ptr(), value.len());
        }
    }
    
    unsafe fn load_bitfield(field: &mut FieldValue, value_ptr: *const u8) {
        if let FieldValue::BitField { ref mut value, bits } = *field {
            let total_bytes = ((bits + 7) / 8) as usize;
            ptr::copy_nonoverlapping(value_ptr, value.as_mut_ptr(), total_bytes);
        }
    }

    pub fn as_u8(&self) -> Option<u8> {
        return match self {
            FieldValue::U8(v) => Some(*v),
            _ => None,
        };
    }

    pub fn as_i8(&self) -> Option<i8> {
        return match self {
            FieldValue::I8(v) => Some(*v),
            _ => None,
        };
    }

    pub fn as_u16(&self) -> Option<u16> {
        return match self {
            FieldValue::U16(v, _) => Some(*v),
            _ => None,
        };
    }

    pub fn as_i16(&self) -> Option<i16> {
        return match self {
            FieldValue::I16(v, _) => Some(*v),
            _ => None,
        };
    }

    pub fn as_u32(&self) -> Option<u32> {
        return match self {
            FieldValue::U32(v, _) => Some(*v),
            _ => None,
        };
    }

    pub fn as_i32(&self) -> Option<i32> {
        return match self {
            FieldValue::I32(v, _) => Some(*v),
            _ => None,
        };
    }

    pub fn as_u64(&self) -> Option<u64> {
        return match self {
            FieldValue::U64(v, _) => Some(*v),
            _ => None,
        };
    }

    pub fn as_i64(&self) -> Option<i64> {
        return match self {
            FieldValue::I64(v, _) => Some(*v),
            _ => None,
        };
    }

    pub fn as_f32(&self) -> Option<f32> {
        return match self {
            FieldValue::F32(v, _) => Some(*v),
            _ => None,
        };
    }

    pub fn as_f64(&self) -> Option<f64> {
        return match self {
            FieldValue::F64(v, _) => Some(*v),
            _ => None,
        };
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        return match self {
            FieldValue::Bytes(ref v) => Some(v.as_slice()),
            _ => None,
        };
    }

    pub fn as_bitfield(&self) -> Option<(&[u8], u16)> {
        return match self {
            FieldValue::BitField { ref value, bits } => Some((value.as_slice(), *bits)),
            _ => None,
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

        // Parse the data type using the DataType's FromStr implementation, if no value is provided, accept this and assume we want default values.
        let data_type = if has_value {
            DataType::from_str(value_and_type[1])?
        } else {
            DataType::from_str(value_and_type[0])?
        };

        if !has_value {
            return Ok(data_type.to_default_value());
        }

        let value_str = value_and_type[0];

        // Parse the value based on the parsed DataType
        let field_value = match data_type {
            DataType::U8 => value_str.parse::<u8>().map(FieldValue::U8).map_err(|e| e.to_string()),
            DataType::U16(endian) => value_str.parse::<u16>().map(|v| FieldValue::U16(v, endian)).map_err(|e| e.to_string()),
            DataType::U32(endian) => value_str.parse::<u32>().map(|v| FieldValue::U32(v, endian)).map_err(|e| e.to_string()),
            DataType::U64(endian) => value_str.parse::<u64>().map(|v| FieldValue::U64(v, endian)).map_err(|e| e.to_string()),
            DataType::I8 => value_str.parse::<i8>().map(FieldValue::I8).map_err(|e| e.to_string()),
            DataType::I16(endian) => value_str.parse::<i16>().map(|v| FieldValue::I16(v, endian)).map_err(|e| e.to_string()),
            DataType::I32(endian) => value_str.parse::<i32>().map(|v| FieldValue::I32(v, endian)).map_err(|e| e.to_string()),
            DataType::I64(endian) => value_str.parse::<i64>().map(|v| FieldValue::I64(v, endian)).map_err(|e| e.to_string()),
            DataType::F32(endian) => value_str.parse::<f32>().map(|v| FieldValue::F32(v, endian)).map_err(|e| e.to_string()),
            DataType::F64(endian) => value_str.parse::<f64>().map(|v| FieldValue::F64(v, endian)).map_err(|e| e.to_string()),
            DataType::Bytes => Ok(FieldValue::Bytes(value_str.as_bytes().to_vec())),
            DataType::BitField(bits) => {
                let bytes = hex::decode(value_str).map_err(|e| e.to_string())?;
                if bytes.len() * 8 < bits as usize {
                    return Err("Not enough bits in bitfield".to_string());
                }
                Ok(FieldValue::BitField { value: bytes, bits })
            }
        }?;

        return Ok(field_value);
    }
}
