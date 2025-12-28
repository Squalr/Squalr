use crate::registries::symbols::symbol_registry::SymbolRegistry;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;

#[derive(Debug, Clone)]
pub struct ScanConstraint {
    scan_compare_type: ScanCompareType,
    data_value: DataValue,
    periodicity: u64,
}

impl ScanConstraint {
    pub fn new(
        scan_compare_type: ScanCompareType,
        data_value: DataValue,
        symbol_registry: &SymbolRegistry,
    ) -> Self {
        let periodicity = Self::calculate_periodicity(symbol_registry, &data_value, &scan_compare_type);

        Self {
            scan_compare_type,
            data_value,
            periodicity,
        }
    }

    pub fn get_scan_compare_type(&self) -> ScanCompareType {
        self.scan_compare_type
    }

    pub fn set_scan_compare_type(
        &mut self,
        scan_compare_type: ScanCompareType,
    ) {
        self.scan_compare_type = scan_compare_type
    }

    pub fn get_data_value(&self) -> &DataValue {
        &self.data_value
    }

    pub fn get_data_value_mut(&mut self) -> &mut DataValue {
        &mut self.data_value
    }

    pub fn set_data_value(
        &mut self,
        data_value: DataValue,
        symbol_registry: &SymbolRegistry,
    ) {
        self.data_value = data_value;
        self.periodicity = Self::calculate_periodicity(symbol_registry, &self.data_value, &self.scan_compare_type)
    }

    /// Updates the data type in place without updating the value bytes.
    pub fn set_data_type_in_place(
        &mut self,
        data_type_ref: DataTypeRef,
        symbol_registry: &SymbolRegistry,
    ) {
        self.data_value.set_data_type_in_place(data_type_ref);
        self.periodicity = Self::calculate_periodicity(symbol_registry, &self.data_value, &self.scan_compare_type)
    }

    fn calculate_periodicity(
        symbol_registry: &SymbolRegistry,
        data_value: &DataValue,
        scan_compare_type: &ScanCompareType,
    ) -> u64 {
        match scan_compare_type {
            ScanCompareType::Immediate(_scan_compare_type_immediate) => {
                Self::calculate_periodicity_from_immediate(symbol_registry, &data_value.get_value_bytes(), data_value.get_data_type_ref())
            }
            ScanCompareType::Delta(_scan_compare_type_immediate) => {
                Self::calculate_periodicity_from_immediate(symbol_registry, &data_value.get_value_bytes(), data_value.get_data_type_ref())
            }
            _ => 0,
        }
    }

    /// Calculates the length of repeating byte patterns within a given data type and value combination.
    /// If there are no repeating patterns, the periodicity will be equal to the data type size.
    /// For example, 7C 01 7C 01 has a data typze size of 4, but a periodicity of 2.
    fn calculate_periodicity_from_immediate(
        symbol_registry: &SymbolRegistry,
        immediate_value_bytes: &[u8],
        data_type_ref: &DataTypeRef,
    ) -> u64 {
        // Assume optimal periodicity to begin with
        let mut period = 1;
        let data_type_size_bytes = symbol_registry.get_unit_size_in_bytes(data_type_ref);

        // Loop through all remaining bytes, and increase the periodicity when we encounter a byte that violates the current assumption.
        for byte_index in 1..data_type_size_bytes as usize {
            if immediate_value_bytes[byte_index] != immediate_value_bytes[byte_index % period as usize] {
                period = byte_index as u64 + 1;
            }
        }

        period as u64
    }
}
