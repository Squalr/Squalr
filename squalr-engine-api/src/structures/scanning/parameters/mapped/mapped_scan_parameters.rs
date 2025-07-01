use crate::structures::data_types::built_in_types::u8::data_type_u8::DataTypeU8;
use crate::structures::data_types::built_in_types::u16be::data_type_u16be::DataTypeU16be;
use crate::structures::data_types::built_in_types::u32be::data_type_u32be::DataTypeU32be;
use crate::structures::data_types::built_in_types::u64be::data_type_u64be::DataTypeU64be;
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
use crate::structures::scanning::parameters::element_scan::element_scan_parameters::ElementScanParameters;
use crate::structures::scanning::parameters::element_scan::element_scan_value::ElementScanValue;
use crate::structures::scanning::parameters::mapped::mapped_scan_type::MappedScanType;
use crate::structures::scanning::parameters::mapped::mapped_scan_type::ScanParametersScalar;
use crate::structures::scanning::parameters::mapped::vectorization_size::VectorizationSize;
use std::simd::LaneCount;
use std::simd::SupportedLaneCount;

/// Represents processed scan parameters derived from user provided scan parameters.
#[derive(Debug, Clone)]
pub struct MappedScanParameters {
    data_value_and_alignment: ElementScanValue,
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
        element_scan_parameters: &ElementScanParameters,
    ) -> Self {
        let data_type_ref = snapshot_region_filter_collection.get_data_type();

        Self {
            data_value_and_alignment: element_scan_parameters.get_data_value_and_alignment_for_data_type(data_type_ref),
            scan_compare_type: element_scan_parameters.get_compare_type(),
            floating_point_tolerance: element_scan_parameters.get_floating_point_tolerance(),
            vectorization_size: VectorizationSize::default(),
            periodicity: 0,
            mapped_scan_type: MappedScanType::Scalar(ScanParametersScalar::SingleElement),
        }
    }

    pub fn get_data_value(&self) -> &DataValue {
        &self.data_value_and_alignment.get_data_value()
    }

    pub fn get_data_value_mut(&mut self) -> &mut DataValue {
        self.data_value_and_alignment.get_data_value_mut()
    }

    pub fn get_data_type(&self) -> &DataTypeRef {
        &self.get_data_value().get_data_type()
    }

    pub fn get_memory_alignment(&self) -> MemoryAlignment {
        self.data_value_and_alignment.get_memory_alignment()
    }

    pub fn get_compare_type(&self) -> &ScanCompareType {
        &self.scan_compare_type
    }

    pub fn set_compare_type(
        &mut self,
        scan_compare_type: ScanCompareType,
    ) {
        self.scan_compare_type = scan_compare_type;
    }

    pub fn get_floating_point_tolerance(&self) -> FloatingPointTolerance {
        self.floating_point_tolerance
    }

    pub fn get_vectorization_size(&self) -> &VectorizationSize {
        &self.vectorization_size
    }

    pub fn set_vectorization_size(
        &mut self,
        vectorization_size: VectorizationSize,
    ) {
        self.vectorization_size = vectorization_size;
    }

    pub fn get_periodicity(&self) -> u64 {
        self.periodicity
    }

    pub fn set_periodicity(
        &mut self,
        periodicity: u64,
    ) {
        self.periodicity = periodicity;
    }

    pub fn get_mapped_scan_type(&self) -> &MappedScanType {
        &self.mapped_scan_type
    }

    pub fn set_mapped_scan_type(
        &mut self,
        mapped_scan_type: MappedScanType,
    ) {
        self.mapped_scan_type = mapped_scan_type;
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
}
