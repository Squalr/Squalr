use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use crate::structures::scanning::data_value_and_alignment::DataValueAndAlignment;
use crate::structures::scanning::memory_read_mode::MemoryReadMode;

/// Represents the global scan arguments that are used by all current scans, regardless of `DataType`.
#[derive(Debug, Clone)]
pub struct UserScanParameters {
    compare_type: ScanCompareType,
    data_values_and_alignments: Vec<DataValueAndAlignment>,
    floating_point_tolerance: FloatingPointTolerance,
    memory_read_mode: MemoryReadMode,
    is_single_thread_scan: bool,

    /// If this debug flag is provided, the scan will be performed twice. Once with a specialized scan, and once with the default scan.
    /// An assertion will be made that the default scan produced the exact same result as the specialized scan.
    debug_perform_validation_scan: bool,
}

impl UserScanParameters {
    pub fn new(
        compare_type: ScanCompareType,
        data_values_and_alignments: Vec<DataValueAndAlignment>,
        floating_point_tolerance: FloatingPointTolerance,
        memory_read_mode: MemoryReadMode,
        is_single_thread_scan: bool,
        debug_perform_validation_scan: bool,
    ) -> Self {
        Self {
            compare_type,
            data_values_and_alignments,
            floating_point_tolerance,
            memory_read_mode,
            is_single_thread_scan,
            debug_perform_validation_scan,
        }
    }

    pub fn get_compare_type(&self) -> ScanCompareType {
        self.compare_type.clone()
    }

    pub fn get_data_values_and_alignments(&self) -> &Vec<DataValueAndAlignment> {
        &self.data_values_and_alignments
    }

    pub fn get_compare_immediate_for_data_type(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> DataValue {
        match self.get_compare_type() {
            ScanCompareType::Immediate(_) | ScanCompareType::Delta(_) => {
                for data_value_and_alignment in &self.data_values_and_alignments {
                    if data_value_and_alignment.get_data_type() == data_type_ref {
                        return data_value_and_alignment.get_data_value().clone();
                    }
                }
                DataValue::new(data_type_ref.clone(), vec![])
            }
            ScanCompareType::Relative(_) => DataValue::new(data_type_ref.clone(), vec![]),
        }
    }

    pub fn get_floating_point_tolerance(&self) -> FloatingPointTolerance {
        self.floating_point_tolerance
    }

    pub fn get_memory_read_mode(&self) -> MemoryReadMode {
        self.memory_read_mode
    }

    pub fn is_single_thread_scan(&self) -> bool {
        self.is_single_thread_scan
    }

    pub fn get_debug_perform_validation_scan(&self) -> bool {
        self.debug_perform_validation_scan
    }

    pub fn is_valid(&self) -> bool {
        match self.get_compare_type() {
            ScanCompareType::Immediate(_) | ScanCompareType::Delta(_) => {
                for data_value_and_alignment in &self.data_values_and_alignments {
                    if data_value_and_alignment.get_data_value().get_size_in_bytes() <= 0 {
                        return false;
                    }
                }
                true
            }
            ScanCompareType::Relative(_) => true,
        }
    }

    pub fn is_valid_for_data_type(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> bool {
        match self.get_compare_type() {
            ScanCompareType::Immediate(_) | ScanCompareType::Delta(_) => {
                for data_value_and_alignment in &self.data_values_and_alignments {
                    if data_value_and_alignment.get_data_type() == data_type_ref {
                        return data_value_and_alignment.get_data_value().get_size_in_bytes() > 0;
                    }
                }
                false
            }
            ScanCompareType::Relative(_) => true,
        }
    }
}
