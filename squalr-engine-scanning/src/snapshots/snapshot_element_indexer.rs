use crate::snapshots::snapshot_element_range::SnapshotElementRange;
use squalr_engine_common::dynamic_struct::field_value::FieldValue;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use squalr_engine_common::dynamic_struct::field_value::Endian;
use std::sync::Arc;

pub struct SnapshotElementIndexer {
    element_range: Arc<SnapshotElementRange>,
    element_index: usize,
    alignment: MemoryAlignment,
}

impl SnapshotElementIndexer {
    pub fn new(element_range: Arc<SnapshotElementRange>, alignment: MemoryAlignment, element_index: usize) -> Self {
        Self {
            element_range,
            element_index,
            alignment,
        }
    }

    pub fn get_base_address(&self) -> u64 {
        return self.element_range.get_base_element_address() + (self.element_index * self.alignment as usize) as u64;
    }

    pub fn load_current_value(&self, data_type: FieldValue) -> Option<FieldValue> {
        let offset = self.element_range.get_region_offset() + self.element_index * self.alignment as usize;
        let parent_region = self.element_range.parent_region.read().unwrap();
        let current_values = parent_region.get_current_values();
        let pointer_base = current_values.read().unwrap();

        return Some(self.load_values(data_type, &pointer_base[offset..]));
    }

    pub fn load_previous_value(&self, data_type: FieldValue) -> Option<FieldValue> {
        let offset = self.element_range.get_region_offset() + self.element_index * self.alignment as usize;
        let parent_region = self.element_range.parent_region.read().unwrap();
        let previous_values = parent_region.get_previous_values();
        let pointer_base = previous_values.read().unwrap();

        return Some(self.load_values(data_type, &pointer_base[offset..]));
    }

    fn load_values(&self, data_type: FieldValue, pointer_base: &[u8]) -> FieldValue {
        return match data_type {
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
        };
    }

    pub fn has_current_value(&self) -> bool {
        return self.element_range.parent_region.read().unwrap().has_current_values();
    }

    pub fn has_previous_value(&self) -> bool {
        return self.element_range.parent_region.read().unwrap().has_previous_values();
    }
}
