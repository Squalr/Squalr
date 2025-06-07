use crate::structures::data_types::built_in_types::u8::data_type_u8::DataTypeU8;
use crate::structures::data_types::built_in_types::u16be::data_type_u16be::DataTypeU16be;
use crate::structures::data_types::built_in_types::u32be::data_type_u32be::DataTypeU32be;
use crate::structures::data_types::built_in_types::u64be::data_type_u64be::DataTypeU64be;
use crate::structures::data_types::data_type_meta_data::DataTypeMetaData;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use crate::structures::data_types::generics::vector_comparer::VectorComparer;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::memory::memory_alignment::MemoryAlignment;
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use crate::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use crate::structures::scanning::comparisons::scan_function_scalar::ScanFunctionScalar;
use crate::structures::scanning::comparisons::scan_function_vector::ScanFunctionVector;
use crate::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::structures::scanning::filters::snapshot_region_filter_collection::SnapshotRegionFilterCollection;
use crate::structures::scanning::parameters::mapped::mapped_scan_type::MappedScanType;
use crate::structures::scanning::parameters::mapped::mapped_scan_type::ScanParametersByteArray;
use crate::structures::scanning::parameters::mapped::mapped_scan_type::ScanParametersScalar;
use crate::structures::scanning::parameters::mapped::mapped_scan_type::ScanParametersVector;
use crate::structures::scanning::parameters::mapped::vectorization_size::VectorizationSize;
use crate::structures::scanning::parameters::user::user_scan_parameters::UserScanParameters;
use std::simd::LaneCount;
use std::simd::SupportedLaneCount;

/// Represents processed scan parameters derived from user provided scan parameters.
#[derive(Debug, Clone)]
pub struct MappedScanParameters {
    data_value: DataValue,
    memory_alignment: MemoryAlignment,
    scan_compare_type: ScanCompareType,
    floating_point_tolerance: FloatingPointTolerance,
    vectorization_size: VectorizationSize,
    periodicity: u64,
    mapped_scan_type: MappedScanType,
}

impl MappedScanParameters {
    /// Creates optimized scan paramaters for a given snapshot region filter, given user provided scan parameters.
    /// Internally, the user parameters are processed into more optimal parameters that help select the most optimal scan implementation.
    pub fn new(
        snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        snapshot_region_filter: &SnapshotRegionFilter,
        user_scan_parameters: &UserScanParameters,
    ) -> Self {
        let data_type_ref = snapshot_region_filter_collection.get_data_type();
        let memory_alignment = snapshot_region_filter_collection.get_memory_alignment();
        let mut mapped_params = Self {
            data_value: user_scan_parameters.get_compare_immediate_for_data_type(data_type_ref),
            memory_alignment: snapshot_region_filter_collection.get_memory_alignment(),
            scan_compare_type: user_scan_parameters.get_compare_type(),
            floating_point_tolerance: user_scan_parameters.get_floating_point_tolerance(),
            vectorization_size: VectorizationSize::default(),
            periodicity: 0,
            mapped_scan_type: MappedScanType::Scalar(ScanParametersScalar::SingleElement),
        };

        // First try a single element scanner. This is valid even for cases like array of byte scans, as all data types support basic equality checks.
        if Self::is_single_element_scan(snapshot_region_filter, data_type_ref, memory_alignment) {
            return mapped_params;
        }

        // Try to map the scan value to primitive scans for performance gains.
        // For example, a byte array scan of 2 bytes can be mapped to a u16 scan.
        match Self::try_map_to_primitive(mapped_params.get_compare_type(), &mapped_params.get_data_value()) {
            Some(mapped_data_type_ref) => {
                // Mapping onto a primitive type map was successful. Update our new internal data type, and proceed with this as the new type.
                mapped_params.data_value.remap_data_type(mapped_data_type_ref);
            }
            None => {
                if Self::can_remap_to_byte_array(mapped_params.get_compare_type(), &mapped_params.get_data_value()) {
                    // JIRA: Okay but this breaks if they scan for an array of floats, since float comparisons are actually non-discrete.
                    if mapped_params.data_value.get_data_type().is_discrete() {
                        log::warn!(
                            "Float array type scans are currently not fully supported! These scans currently lack tolerance checks and perform byte-wise exact comparisons. Scan accuracy may suffer."
                        )
                    }

                    // Perform a byte array scan, since we were unable to map the byte array to a primitive type.
                    // These are the only acceptable options, either the type is a primitive, or its a byte array.
                    mapped_params.mapped_scan_type = MappedScanType::ByteArray(ScanParametersByteArray::ByteArrayBooyerMoore);

                    return mapped_params;
                }
            }
        }

        // Now we decide whether to use a scalar or SIMD scan based on filter region size.
        mapped_params.vectorization_size =
            match Self::create_vectorization_size(snapshot_region_filter, &mapped_params.get_data_type(), mapped_params.memory_alignment) {
                None => {
                    // The filter cannot fit into a vector! Revert to scalar scan.
                    mapped_params.mapped_scan_type = MappedScanType::Scalar(ScanParametersScalar::ScalarIterative);

                    return mapped_params;
                }
                Some(vectorization_size) => vectorization_size,
            };

        let data_type_size = mapped_params.get_data_type().get_size_in_bytes();
        let memory_alignment_size = mapped_params.get_memory_alignment() as u64;

        if data_type_size > memory_alignment_size {
            // For discrete, multi-byte, primitive types (non-floating point), we can fall back on optimized scans if explicitly performing == or != scans.
            if mapped_params.data_value.get_data_type().is_discrete()
                && mapped_params.data_value.get_size_in_bytes() > 1
                && Self::is_checking_equal_or_not_equal(&mapped_params.scan_compare_type)
            {
                if let Some(periodicity) = Self::calculate_periodicity(mapped_params.get_data_value(), &mapped_params.scan_compare_type) {
                    mapped_params.periodicity = periodicity;

                    match periodicity {
                        1 => {
                            // Better for debug mode.
                            // mapped_params.mapped_scan_type = MappedScanType::Vector(ScanParametersVector::OverlappingBytewisePeriodic);

                            // Better for release mode.
                            mapped_params.mapped_scan_type = MappedScanType::Vector(ScanParametersVector::OverlappingBytewiseStaggered);

                            return mapped_params;
                        }
                        2 | 4 | 8 => {
                            mapped_params.mapped_scan_type = MappedScanType::Vector(ScanParametersVector::OverlappingBytewiseStaggered);

                            return mapped_params;
                        }
                        _ => {}
                    }
                }
            }
            mapped_params.mapped_scan_type = MappedScanType::Vector(ScanParametersVector::Overlapping);
            mapped_params
        } else if data_type_size < memory_alignment_size {
            mapped_params.mapped_scan_type = MappedScanType::Vector(ScanParametersVector::Sparse);
            mapped_params
        } else {
            mapped_params.mapped_scan_type = MappedScanType::Vector(ScanParametersVector::Aligned);
            mapped_params
        }
    }

    pub fn get_data_value(&self) -> &DataValue {
        &self.data_value
    }

    pub fn get_data_type(&self) -> &DataTypeRef {
        self.data_value.get_data_type()
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

    pub fn get_vectorization_size(&self) -> &VectorizationSize {
        &self.vectorization_size
    }

    pub fn get_periodicity(&self) -> u64 {
        self.periodicity
    }

    pub fn get_mapped_scan_type(&self) -> &MappedScanType {
        &self.mapped_scan_type
    }

    pub fn get_scan_function_scalar(&self) -> Option<ScanFunctionScalar> {
        match self.get_compare_type() {
            ScanCompareType::Immediate(scan_compare_type_immediate) => {
                if let Some(compare_func) = self
                    .get_data_type()
                    .get_scalar_compare_func_immediate(&scan_compare_type_immediate, &self)
                {
                    return Some(ScanFunctionScalar::Immediate(compare_func));
                }
            }
            ScanCompareType::Relative(scan_compare_type_relative) => {
                if let Some(compare_func) = self
                    .get_data_type()
                    .get_scalar_compare_func_relative(&scan_compare_type_relative, &self)
                {
                    return Some(ScanFunctionScalar::RelativeOrDelta(compare_func));
                }
            }
            ScanCompareType::Delta(scan_compare_type_delta) => {
                if let Some(compare_func) = self
                    .get_data_type()
                    .get_scalar_compare_func_delta(&scan_compare_type_delta, &self)
                {
                    return Some(ScanFunctionScalar::RelativeOrDelta(compare_func));
                }
            }
        }

        None
    }

    pub fn get_scan_function_vector<const N: usize>(&self) -> Option<ScanFunctionVector<N>>
    where
        LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
    {
        match self.get_compare_type() {
            ScanCompareType::Immediate(scan_compare_type_immediate) => {
                if let Some(compare_func) = self
                    .get_data_type()
                    .get_vector_compare_func_immediate(&scan_compare_type_immediate, &self)
                {
                    return Some(ScanFunctionVector::Immediate(compare_func));
                }
            }
            ScanCompareType::Relative(scan_compare_type_relative) => {
                if let Some(compare_func) = self
                    .get_data_type()
                    .get_vector_compare_func_relative(&scan_compare_type_relative, &self)
                {
                    return Some(ScanFunctionVector::RelativeOrDelta(compare_func));
                }
            }
            ScanCompareType::Delta(scan_compare_type_delta) => {
                if let Some(compare_func) = self
                    .get_data_type()
                    .get_vector_compare_func_delta(&scan_compare_type_delta, &self)
                {
                    return Some(ScanFunctionVector::RelativeOrDelta(compare_func));
                }
            }
        }

        None
    }

    fn is_single_element_scan(
        snapshot_region_filter: &SnapshotRegionFilter,
        data_type: &DataTypeRef,
        memory_alignment: MemoryAlignment,
    ) -> bool {
        let element_count = snapshot_region_filter.get_element_count(data_type, memory_alignment);

        element_count == 1
    }

    fn try_map_to_primitive(
        scan_compare_type: &ScanCompareType,
        data_value: &DataValue,
    ) -> Option<DataTypeRef> {
        // Only immediate scans can be remapped, if the scan is relative, then the original data type is crucial.
        match scan_compare_type {
            ScanCompareType::Relative(_) | ScanCompareType::Delta(_) => return None,
            ScanCompareType::Immediate(_) => {}
        };

        // Non discrete / floating point types cannot be remapped. For example, if we have an array of two f32 values,
        // we absolutely cannot remap this to a single u64 (nor an f64) since these require tolerance comparisons.
        if !data_value.get_data_type().is_discrete() {
            return None;
        }

        let data_type_size = data_value.get_size_in_bytes();
        let data_type_default_size = data_value.get_data_type().get_default_size_in_bytes();

        // If the data type size is the default for that type, and its already a valid primitive size,
        // there is no need to perform a remapping. We do this check to avoid meaningless remappings,
        // such as remapping i16 to u16, even though this is technically acceptable.
        if data_type_size == data_type_default_size {
            match data_type_size {
                1 | 2 | 4 | 8 => return None,
                _ => {}
            };
        }

        // If applicable, try to reinterpret array of byte scans as a primitive type of the same size.
        // These are much more efficient than array of byte scans, so for scans of these sizes performance will be improved greatly.
        match data_type_size {
            8 => Some(DataTypeRef::new(DataTypeU64be::get_data_type_id(), DataTypeMetaData::None)),
            4 => Some(DataTypeRef::new(DataTypeU32be::get_data_type_id(), DataTypeMetaData::None)),
            2 => Some(DataTypeRef::new(DataTypeU16be::get_data_type_id(), DataTypeMetaData::None)),
            1 => Some(DataTypeRef::new(DataTypeU8::get_data_type_id(), DataTypeMetaData::None)),
            _ => None,
        }
    }

    fn can_remap_to_byte_array(
        scan_compare_type: &ScanCompareType,
        data_value: &DataValue,
    ) -> bool {
        match scan_compare_type {
            ScanCompareType::Relative(_) | ScanCompareType::Delta(_) => return false,
            ScanCompareType::Immediate(_) => {}
        };

        if data_value.get_data_type().is_discrete() {
            return false;
        }

        true
    }

    fn create_vectorization_size(
        snapshot_region_filter: &SnapshotRegionFilter,
        data_type: &DataTypeRef,
        memory_alignment: MemoryAlignment,
    ) -> Option<VectorizationSize> {
        // Rather than using the snapshot_region_filter.get_region_size() directly, we try to be smart about ensuring
        // There is enough space to actually read a full vector of elements.
        // For example, if scanning for i32, 1-byte aligned, a single region of 64 bytes is not actually very helpful.
        // This is because we would actually want to overlap based on alignment, and thus would need at least 67 bytes.
        // This is derived from scanning for four i32 values at alignments 0, 1, 2, and 3.
        let element_count = snapshot_region_filter.get_element_count(data_type, memory_alignment);
        let usable_region_size = element_count * (memory_alignment as u64);

        if usable_region_size >= 64 {
            Some(VectorizationSize::Vector64)
        } else if usable_region_size >= 32 {
            Some(VectorizationSize::Vector32)
        } else if usable_region_size >= 16 {
            Some(VectorizationSize::Vector16)
        } else {
            None
        }
    }

    fn is_checking_equal_or_not_equal(scan_compare_type: &ScanCompareType) -> bool {
        match scan_compare_type {
            ScanCompareType::Immediate(scan_compare_type_immediate) => match scan_compare_type_immediate {
                ScanCompareTypeImmediate::Equal => true,
                ScanCompareTypeImmediate::NotEqual => true,
                _ => false,
            },
            _ => false,
        }
    }

    fn calculate_periodicity(
        data_value: &DataValue,
        scan_compare_type: &ScanCompareType,
    ) -> Option<u64> {
        match scan_compare_type {
            ScanCompareType::Immediate(_scan_compare_type_immediate) => Some(Self::calculate_periodicity_from_immediate(
                &data_value.get_value_bytes(),
                data_value.get_data_type(),
            )),
            ScanCompareType::Delta(_scan_compare_type_immediate) => Some(Self::calculate_periodicity_from_immediate(
                &data_value.get_value_bytes(),
                data_value.get_data_type(),
            )),
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
