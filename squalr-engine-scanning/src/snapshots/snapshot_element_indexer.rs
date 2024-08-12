use crate::snapshots::snapshot_element_range::SnapshotElementRange;
use squalr_engine_common::dynamic_struct::field_value::FieldValue;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use squalr_engine_common::dynamic_struct::field_value::Endian;

pub struct SnapshotElementIndexer<'a> {
    element_range: &'a SnapshotElementRange<'a>,
    element_index: usize,
    alignment: MemoryAlignment,
}

impl<'a> SnapshotElementIndexer<'a> {
    pub fn new(element_range: &'a SnapshotElementRange, alignment: MemoryAlignment, element_index: usize) -> Self {
        Self {
            element_range,
            element_index,
            alignment,
        }
    }

    pub fn get_base_address(&self) -> u64 {
        self.element_range.get_base_element_address() 
            + (self.element_index * self.alignment as usize) as u64
    }

    pub fn load_current_value(&self, data_type: FieldValue) -> FieldValue {
        let offset = self.element_range.region_offset + self.element_index * self.alignment as usize;
        let pointer_base = &self.element_range.parent_region.borrow().current_values[offset..];
        self.load_values(data_type, pointer_base)
    }

    pub fn load_previous_value(&self, data_type: FieldValue) -> FieldValue {
        let offset = self.element_range.region_offset + self.element_index * self.alignment as usize;
        let pointer_base = &self.element_range.parent_region.borrow().previous_values[offset..];
        self.load_values(data_type, pointer_base)
    }

    fn load_values(&self, data_type: FieldValue, pointer_base: &[u8]) -> FieldValue {
        match data_type {
            FieldValue::U8(_) => FieldValue::U8(pointer_base[0]),
            FieldValue::I8(_) => FieldValue::I8(pointer_base[0] as i8),
            FieldValue::U16(_, Endian::Little) => FieldValue::U16(u16::from_le_bytes(pointer_base[..2].try_into().unwrap()), Endian::Little),
            FieldValue::U16(_, Endian::Big) => FieldValue::U16(u16::from_be_bytes(pointer_base[..2].try_into().unwrap()), Endian::Big),
            FieldValue::I16(_, Endian::Little) => FieldValue::I16(i16::from_le_bytes(pointer_base[..2].try_into().unwrap()), Endian::Little),
            FieldValue::I16(_, Endian::Big) => FieldValue::I16(i16::from_be_bytes(pointer_base[..2].try_into().unwrap()), Endian::Big),
            FieldValue::U32(_, Endian::Little) => FieldValue::U32(u32::from_le_bytes(pointer_base[..4].try_into().unwrap()), Endian::Little),
            FieldValue::U32(_, Endian::Big) => FieldValue::U32(u32::from_be_bytes(pointer_base[..4].try_into().unwrap()), Endian::Big),
            FieldValue::I32(_, Endian::Little) => FieldValue::I32(i32::from_le_bytes(pointer_base[..4].try_into().unwrap()), Endian::Little),
            FieldValue::I32(_, Endian::Big) => FieldValue::I32(i32::from_be_bytes(pointer_base[..4].try_into().unwrap()), Endian::Big),
            FieldValue::U64(_, Endian::Little) => FieldValue::U64(u64::from_le_bytes(pointer_base[..8].try_into().unwrap()), Endian::Little),
            FieldValue::U64(_, Endian::Big) => FieldValue::U64(u64::from_be_bytes(pointer_base[..8].try_into().unwrap()), Endian::Big),
            FieldValue::I64(_, Endian::Little) => FieldValue::I64(i64::from_le_bytes(pointer_base[..8].try_into().unwrap()), Endian::Little),
            FieldValue::I64(_, Endian::Big) => FieldValue::I64(i64::from_be_bytes(pointer_base[..8].try_into().unwrap()), Endian::Big),
            FieldValue::F32(_, Endian::Little) => FieldValue::F32(f32::from_le_bytes(pointer_base[..4].try_into().unwrap()), Endian::Little),
            FieldValue::F32(_, Endian::Big) => FieldValue::F32(f32::from_be_bytes(pointer_base[..4].try_into().unwrap()), Endian::Big),
            FieldValue::F64(_, Endian::Little) => FieldValue::F64(f64::from_le_bytes(pointer_base[..8].try_into().unwrap()), Endian::Little),
            FieldValue::F64(_, Endian::Big) => FieldValue::F64(f64::from_be_bytes(pointer_base[..8].try_into().unwrap()), Endian::Big),
            FieldValue::Bytes(_) => FieldValue::Bytes(pointer_base.to_vec()),
            _ => panic!("Unsupported data type"),
        }
    }

    pub fn has_current_value(&self) -> bool {
        !self.element_range.parent_region.borrow().current_values.is_empty()
    }

    pub fn has_previous_value(&self) -> bool {
        !self.element_range.parent_region.borrow().previous_values.is_empty()
    }
}
