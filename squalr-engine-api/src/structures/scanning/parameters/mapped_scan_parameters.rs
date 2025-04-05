use crate::structures::data_types::built_in_types::byte_array::data_type_byte_array::DataTypeByteArray;
use crate::structures::data_types::built_in_types::u8::data_type_u8::DataTypeU8;
use crate::structures::data_types::built_in_types::u16be::data_type_u16be::DataTypeU16be;
use crate::structures::data_types::built_in_types::u32be::data_type_u32be::DataTypeU32be;
use crate::structures::data_types::built_in_types::u64be::data_type_u64be::DataTypeU64be;
use crate::structures::data_types::data_type_meta_data::DataTypeMetaData;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::memory::memory_alignment::MemoryAlignment;
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use crate::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::structures::scanning::parameters::user_scan_parameters_global::UserScanParametersGlobal;
use crate::structures::scanning::parameters::user_scan_parameters_local::UserScanParametersLocal;

#[derive(Debug, Clone)]
pub enum VectorizationSize {
    Vector16,
    Vector32,
    Vector64,
}

#[derive(Debug, Clone)]
pub struct ScanParametersCommon {
    data_type: DataTypeRef,
    data_value: DataValue,
    memory_alignment: MemoryAlignment,
    scan_compare_type: ScanCompareType,
    floating_point_tolerance: FloatingPointTolerance,
}

impl ScanParametersCommon {
    pub fn get_data_value(&self) -> &DataValue {
        &self.data_value
    }

    pub fn get_data_type(&self) -> &DataTypeRef {
        &self.data_type
    }

    pub fn get_memory_alignment(&self) -> MemoryAlignment {
        self.memory_alignment
    }

    pub fn get_compare_type(&self) -> &ScanCompareType {
        &self.scan_compare_type
    }

    pub fn get_floating_point_tolerance(&self) -> FloatingPointTolerance {
        self.floating_point_tolerance
    }
}

#[derive(Debug, Clone)]
pub struct ScanParametersCommonVector {
    parameters: ScanParametersCommon,
    vectorization_size: VectorizationSize,
}

impl ScanParametersCommonVector {
    pub fn get_common_params(&self) -> &ScanParametersCommon {
        &self.parameters
    }

    pub fn get_data_value(&self) -> &DataValue {
        self.parameters.get_data_value()
    }

    pub fn get_data_type(&self) -> &DataTypeRef {
        self.parameters.get_data_type()
    }

    pub fn get_memory_alignment(&self) -> MemoryAlignment {
        self.parameters.get_memory_alignment()
    }

    pub fn get_compare_type(&self) -> &ScanCompareType {
        self.parameters.get_compare_type()
    }

    pub fn get_floating_point_tolerance(&self) -> FloatingPointTolerance {
        self.parameters.get_floating_point_tolerance()
    }

    pub fn get_vectorization_size(&self) -> &VectorizationSize {
        &self.vectorization_size
    }
}

#[derive(Debug, Clone)]
pub struct ScanParametersVectorPeriodic {
    parameters: ScanParametersCommonVector,
    periodicity: u64,
}

impl ScanParametersVectorPeriodic {
    pub fn get_common_params(&self) -> &ScanParametersCommon {
        self.parameters.get_common_params()
    }

    pub fn get_data_value(&self) -> &DataValue {
        self.parameters.get_data_value()
    }

    pub fn get_data_type(&self) -> &DataTypeRef {
        self.parameters.get_data_type()
    }

    pub fn get_memory_alignment(&self) -> MemoryAlignment {
        self.parameters.get_memory_alignment()
    }

    pub fn get_compare_type(&self) -> &ScanCompareType {
        self.parameters.get_compare_type()
    }

    pub fn get_floating_point_tolerance(&self) -> FloatingPointTolerance {
        self.parameters.get_floating_point_tolerance()
    }

    pub fn get_vectorization_size(&self) -> &VectorizationSize {
        self.parameters.get_vectorization_size()
    }

    pub fn get_periodicity(&self) -> u64 {
        self.periodicity
    }
}

#[derive(Debug, Clone)]
pub enum ScanParametersScalar {
    SingleElement(ScanParametersCommon),
    ScalarIterative(ScanParametersCommon),
}

#[derive(Debug, Clone)]
pub enum ScanParametersVector {
    Aligned(ScanParametersCommonVector),
    Sparse(ScanParametersCommonVector),
    OverlappingBytewiseStaggered(ScanParametersCommonVector),
    OverlappingBytewisePeriodic(ScanParametersVectorPeriodic),
}

#[derive(Debug, Clone)]
pub enum ScanParametersByteArray {
    ByteArrayBooyerMoore(ScanParametersCommon),
}

/// Contains processed parameters that define a scan over a region of memory.
/// These transform user input to ensure that the scan is performed as efficiently as possible.
#[derive(Debug, Clone)]
pub enum MappedScanParameters {
    Scalar(ScanParametersScalar),
    Vector(ScanParametersVector),
    ByteArray(ScanParametersByteArray),
}

impl MappedScanParameters {
    /// Creates optimized scan paramaters for a given snapshot region filter, given user provided global/local scan parameters.
    /// Internally, the user parameters are processed into more optimal parameters that help select the most optimal scan implementation.
    pub fn new(
        snapshot_region_filter: &SnapshotRegionFilter,
        user_scan_parameters_global: &UserScanParametersGlobal,
        user_scan_parameters_local: &UserScanParametersLocal,
    ) -> Self {
        let mut common_params = ScanParametersCommon {
            data_type: user_scan_parameters_local.get_data_type().clone(),
            data_value: Self::get_data_value(user_scan_parameters_global, user_scan_parameters_local),
            memory_alignment: user_scan_parameters_local.get_memory_alignment_or_default(),
            scan_compare_type: user_scan_parameters_global.get_compare_type(),
            floating_point_tolerance: user_scan_parameters_global.get_floating_point_tolerance(),
        };

        // First try a single element scanner. This is valid even for cases like array of byte scans, as all data types support basic equality checks.
        if Self::is_single_element_scan(snapshot_region_filter, user_scan_parameters_local) {
            return MappedScanParameters::Scalar(ScanParametersScalar::SingleElement(common_params));
        }

        // Next handle byte array scans. These can potentially be remapped to primitive scans for performance gains.
        if common_params.data_type.get_data_type_id() == DataTypeByteArray::get_data_type_id() {
            if let Some(mapped_data_type) = Self::try_map_byte_array_to_primitive(&common_params.data_type) {
                common_params.data_type = mapped_data_type;
            } else {
                return MappedScanParameters::ByteArray(ScanParametersByteArray::ByteArrayBooyerMoore(common_params));
            }
        }

        // Now we decide whether to use a scalar or SIMD scan based on filter region size.
        let vectorization_size = match Self::get_vectorization_size(snapshot_region_filter) {
            None => {
                // The filter cannot fit into a vector! Revert to scalar scan.
                return MappedScanParameters::Scalar(ScanParametersScalar::ScalarIterative(common_params));
            }
            Some(vectorization_size) => vectorization_size,
        };

        // For discrete types (non-floating point), we can fall back on optimized scans.
        if common_params.data_type.is_discrete() {
            if let Some(periodicity) = Self::calculate_periodicity(user_scan_parameters_global, &common_params.data_type, &common_params.scan_compare_type) {
                match periodicity {
                    1 => {
                        return MappedScanParameters::Vector(ScanParametersVector::OverlappingBytewisePeriodic(ScanParametersVectorPeriodic {
                            parameters: ScanParametersCommonVector {
                                parameters: common_params,
                                vectorization_size,
                            },
                            periodicity,
                        }));
                    }
                    2 | 4 | 8 => {
                        return MappedScanParameters::Vector(ScanParametersVector::OverlappingBytewiseStaggered(ScanParametersCommonVector {
                            parameters: common_params,
                            vectorization_size,
                        }));
                    }
                    _ => {}
                }
            }
        }

        MappedScanParameters::Scalar(ScanParametersScalar::ScalarIterative(common_params))
    }

    fn get_data_value(
        user_scan_parameters_global: &UserScanParametersGlobal,
        user_scan_parameters_local: &UserScanParametersLocal,
    ) -> DataValue {
        match user_scan_parameters_global.get_data_value(user_scan_parameters_local) {
            Some(data_value) => data_value,
            None => DataValue::new("", vec![]),
        }
    }

    fn is_single_element_scan(
        snapshot_region_filter: &SnapshotRegionFilter,
        user_scan_parameters_local: &UserScanParametersLocal,
    ) -> bool {
        let element_count = snapshot_region_filter.get_element_count(
            user_scan_parameters_local.get_data_type(),
            user_scan_parameters_local.get_memory_alignment_or_default(),
        );

        element_count == 1
    }

    fn try_map_byte_array_to_primitive(data_type: &DataTypeRef) -> Option<DataTypeRef> {
        let original_data_type_size = data_type.get_size_in_bytes();

        // If applicable, try to reinterpret array of byte scans as a primitive type of the same size.
        // These are much more efficient than array of byte scans, so for scans of these sizes performance will be improved greatly.
        match original_data_type_size {
            8 => Some(DataTypeRef::new(DataTypeU64be::get_data_type_id(), DataTypeMetaData::None)),
            4 => Some(DataTypeRef::new(DataTypeU32be::get_data_type_id(), DataTypeMetaData::None)),
            2 => Some(DataTypeRef::new(DataTypeU16be::get_data_type_id(), DataTypeMetaData::None)),
            1 => Some(DataTypeRef::new(DataTypeU8::get_data_type_id(), DataTypeMetaData::None)),
            _ => None,
        }
    }

    fn get_vectorization_size(snapshot_region_filter: &SnapshotRegionFilter) -> Option<VectorizationSize> {
        let filter_region_size = snapshot_region_filter.get_region_size();

        if filter_region_size >= 64 {
            Some(VectorizationSize::Vector64)
        } else if filter_region_size >= 32 {
            Some(VectorizationSize::Vector32)
        } else if filter_region_size >= 16 {
            Some(VectorizationSize::Vector16)
        } else {
            None
        }
    }

    fn calculate_periodicity(
        user_scan_parameters_global: &UserScanParametersGlobal,
        data_type: &DataTypeRef,
        scan_compare_type: &ScanCompareType,
    ) -> Option<u64> {
        match scan_compare_type {
            ScanCompareType::Immediate(_scan_compare_type_immediate) => {
                if let Some(compare_immediate) = user_scan_parameters_global.get_compare_immediate() {
                    if let Ok(immediate_value) = data_type.deanonymize_value(compare_immediate) {
                        Some(Self::calculate_periodicity_from_immediate(immediate_value.get_value_bytes(), &data_type))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            ScanCompareType::Delta(_scan_compare_type_immediate) => {
                if let Some(compare_immediate) = user_scan_parameters_global.get_compare_immediate() {
                    if let Ok(immediate_value) = data_type.deanonymize_value(compare_immediate) {
                        Some(Self::calculate_periodicity_from_immediate(immediate_value.get_value_bytes(), &data_type))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Calculates the length of repeating byte patterns within a given data type and value combination.
    /// If there are no repeating patterns, the periodicity will be equal to the data type size.
    /// For example, 7C 01 7C 01 has a data typze size of 4, but a periodicity of 2.
    fn calculate_periodicity_from_immediate(
        immediate_value_bytes: &[u8],
        data_type: &DataTypeRef,
    ) -> u64 {
        // Assume optimal periodicity to begin with
        let mut period = 1;
        let data_type_size_bytes = data_type.get_size_in_bytes();

        // Loop through all remaining bytes, and increase the periodicity when we encounter a byte that violates the current assumption.
        for byte_index in 1..data_type_size_bytes as usize {
            if immediate_value_bytes[byte_index] != immediate_value_bytes[byte_index % period] {
                period = byte_index + 1;
            }
        }

        period as u64
    }
}
