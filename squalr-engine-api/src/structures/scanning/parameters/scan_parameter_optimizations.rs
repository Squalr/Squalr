use crate::structures::data_types::built_in_types::u8::data_type_u8::DataTypeU8;
use crate::structures::data_types::built_in_types::u16::data_type_u16::DataTypeU16;
use crate::structures::data_types::built_in_types::u16be::data_type_u16be::DataTypeU16be;
use crate::structures::data_types::built_in_types::u32::data_type_u32::DataTypeU32;
use crate::structures::data_types::built_in_types::u32be::data_type_u32be::DataTypeU32be;
use crate::structures::data_types::built_in_types::u64::data_type_u64::DataTypeU64;
use crate::structures::data_types::built_in_types::u64be::data_type_u64be::DataTypeU64be;
use crate::structures::data_types::data_type_meta_data::DataTypeMetaData;
use crate::structures::data_types::{built_in_types::byte_array::data_type_byte_array::DataTypeByteArray, data_type_ref::DataTypeRef};
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use crate::structures::scanning::parameters::scan_parameters_global::ScanParametersGlobal;
use crate::structures::scanning::parameters::scan_parameters_local::ScanParametersLocal;

/// Defines extra parameters generated to optimize a scan.
#[derive(Debug, Clone)]
pub struct ScanParameterOptimizations {
    data_type_override: Option<DataTypeRef>,
    periodicity: Option<u64>,
}

impl ScanParameterOptimizations {
    /// Remaps scan parameters into "functionally equivalent" paramters for performance gains.
    /// For example, an array of byte scan for 00 00 00 00 is better treated as a u32 scan of 0, as this is easily vectorized.
    pub fn new(
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Self {
        let original_data_type_size = scan_parameters_local.get_data_type().get_size_in_bytes();
        let mut data_type_override = None;
        let mut periodicity = None;

        if scan_parameters_local.get_data_type().get_data_type_id() == DataTypeByteArray::get_data_type_id() {
            // If applicable, try to reinterpret array of byte scans as a primitive type of the same size.
            // These are much more efficient than array of byte scans, so for scans of these sizes performance will be improved greatly.
            if let Some(new_data_type) = match original_data_type_size {
                8 => Some(DataTypeRef::new(DataTypeU64be::get_data_type_id(), DataTypeMetaData::None)),
                4 => Some(DataTypeRef::new(DataTypeU32be::get_data_type_id(), DataTypeMetaData::None)),
                2 => Some(DataTypeRef::new(DataTypeU16be::get_data_type_id(), DataTypeMetaData::None)),
                1 => Some(DataTypeRef::new(DataTypeU8::get_data_type_id(), DataTypeMetaData::None)),
                // If we can't reinterpret as a primitive, we will just fall back on the generic array of byte scan.
                _ => None,
            } {
                data_type_override = Some(new_data_type);
            }
        };

        // Grab the potentially updated data type / size now that we have finished remapping.
        let data_type = if let Some(data_type) = data_type_override.clone() {
            data_type
        } else {
            scan_parameters_local.get_data_type().clone()
        };
        let data_type_size = data_type.get_size_in_bytes();

        match scan_parameters_global.get_compare_type() {
            ScanCompareType::Immediate(_scan_compare_type_immediate) => {
                if let Some(compare_immediate) = scan_parameters_global.get_compare_immediate() {
                    if let Ok(immediate_value) = data_type.deanonymize_value(compare_immediate) {
                        periodicity = Some(Self::calculate_periodicity_from_immediate(immediate_value.get_value_bytes(), data_type_size))
                    }
                }
            }
            ScanCompareType::Delta(_scan_compare_type_immediate) => {
                if let Some(compare_immediate) = scan_parameters_global.get_compare_immediate() {
                    if let Ok(immediate_value) = data_type.deanonymize_value(compare_immediate) {
                        periodicity = Some(Self::calculate_periodicity_from_immediate(immediate_value.get_value_bytes(), data_type_size));
                    }
                }
            }
            _ => {}
        };

        if let Some(periodicity) = periodicity {
            match periodicity {
                1 => data_type_override = Some(DataTypeRef::new(DataTypeU8::get_data_type_id(), DataTypeMetaData::None)),
                2 => data_type_override = Some(DataTypeRef::new(DataTypeU16::get_data_type_id(), DataTypeMetaData::None)),
                4 => data_type_override = Some(DataTypeRef::new(DataTypeU32::get_data_type_id(), DataTypeMetaData::None)),
                8 => data_type_override = Some(DataTypeRef::new(DataTypeU64::get_data_type_id(), DataTypeMetaData::None)),
                _ => {}
            }
        }

        Self {
            data_type_override,
            periodicity,
        }
    }

    pub fn get_data_type_override(&self) -> &Option<DataTypeRef> {
        &self.data_type_override
    }

    pub fn get_periodicity(&self) -> Option<u64> {
        self.periodicity
    }

    /// Calculates the length of repeating byte patterns within a given data type and value combination.
    /// If there are no repeating patterns, the periodicity will be equal to the data type size.
    /// For example, 7C 01 7C 01 has a data typze size of 4, but a periodicity of 2.
    fn calculate_periodicity_from_immediate(
        immediate_value_bytes: &[u8],
        data_type_size_bytes: u64,
    ) -> u64 {
        // Assume optimal periodicity to begin with
        let mut period = 1;

        // Loop through all remaining bytes, and increase the periodicity when we encounter a byte that violates the current assumption.
        for byte_index in 1..data_type_size_bytes as usize {
            if immediate_value_bytes[byte_index] != immediate_value_bytes[byte_index % period] {
                period = byte_index + 1;
            }
        }

        period as u64
    }
}
