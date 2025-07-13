use crate::registries::data_types::data_type_registry::DataTypeRegistry;
use crate::structures::data_types::generics::vector_comparer::VectorComparer;
use crate::structures::data_values::anonymous_value::AnonymousValue;
use crate::structures::data_values::anonymous_value_container::AnonymousValueContainer;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::scanning::comparisons::scan_compare_type_delta::ScanCompareTypeDelta;
use crate::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use crate::structures::scanning::comparisons::scan_compare_type_relative::ScanCompareTypeRelative;
use crate::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnImmediate;
use crate::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnRelative;
use crate::structures::scanning::parameters::mapped::mapped_scan_parameters::MappedScanParameters;
use serde::{Deserialize, Serialize};
use std::simd::LaneCount;
use std::simd::Simd;
use std::simd::SupportedLaneCount;
use std::{
    fmt::{self, Debug},
    str::FromStr,
};

/// Represents a handle to a data type. This is kept as a weak reference, as DataTypes can be registered/unregistered by plugins.
/// As such, `DataType` is a `Box<dyn>` type, so it is much easier to abstract them behind `DataTypeRef` and just pass around handles.
/// This is also important for serialization/deserialization, as if a plugin that defines a type is disabled, we can still deserialize it.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataTypeRef {
    data_type_id: String,
}

impl DataTypeRef {
    /// Creates a new reference to a registered `DataType` with the explicit.
    pub fn new(data_type_id: &str) -> Self {
        Self {
            data_type_id: data_type_id.to_string(),
        }
    }

    /// Determines if the `DataType` this struct represents is currently registered and available.
    pub fn is_valid(&self) -> bool {
        DataTypeRegistry::get_instance()
            .get(self.get_data_type_id())
            .is_some()
    }

    pub fn get_data_type_id(&self) -> &str {
        &self.data_type_id
    }

    pub fn get_icon_id(&self) -> String {
        match DataTypeRegistry::get_instance().get(self.get_data_type_id()) {
            Some(data_type) => data_type.get_icon_id().to_string(),
            None => String::new(),
        }
    }

    pub fn get_unit_size_in_bytes(&self) -> u64 {
        match DataTypeRegistry::get_instance().get(self.get_data_type_id()) {
            Some(data_type) => data_type.get_unit_size_in_bytes(),
            None => 0,
        }
    }

    pub fn validate_value(
        &self,
        anonymous_value: &AnonymousValue,
    ) -> bool {
        let anonymous_value_container = anonymous_value.get_value();

        match DataTypeRegistry::get_instance().get(self.get_data_type_id()) {
            Some(data_type) => {
                if !data_type.validate_value(anonymous_value_container) {
                    return false;
                }
            }
            None => return false,
        }

        true
    }

    pub fn deanonymize_value(
        &self,
        anonymous_value_container: &AnonymousValueContainer,
    ) -> Result<DataValue, String> {
        match DataTypeRegistry::get_instance().get(self.get_data_type_id()) {
            Some(data_type) => {
                let deanonymized_value = data_type.deanonymize_value(anonymous_value_container);

                match deanonymized_value {
                    Ok(value) => Ok(value),
                    Err(error) => Err(error.to_string()),
                }
            }
            None => Err("Cannot deanonymize value: data type is not registered.".into()),
        }
    }

    /// Gets a value indicating whether this value is signed, ie can be negative.
    pub fn is_signed(&self) -> bool {
        match DataTypeRegistry::get_instance().get(self.get_data_type_id()) {
            Some(data_type) => data_type.is_signed(),
            None => false,
        }
    }

    /// Gets a value indicating whether this value is discrete, ie non-floating point.
    pub fn is_floating_point(&self) -> bool {
        match DataTypeRegistry::get_instance().get(self.get_data_type_id()) {
            Some(data_type) => data_type.is_floating_point(),
            None => false,
        }
    }

    pub fn get_default_value(&self) -> Option<DataValue> {
        match DataTypeRegistry::get_instance().get(self.get_data_type_id()) {
            Some(data_type) => Some(data_type.get_default_value(self.clone())),
            None => None,
        }
    }

    pub fn get_scalar_compare_func_immediate(
        &self,
        scan_compare_type: &ScanCompareTypeImmediate,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        match DataTypeRegistry::get_instance().get(self.get_data_type_id()) {
            Some(data_type) => match scan_compare_type {
                ScanCompareTypeImmediate::Equal => data_type.get_compare_equal(mapped_scan_parameters),
                ScanCompareTypeImmediate::NotEqual => data_type.get_compare_not_equal(mapped_scan_parameters),
                ScanCompareTypeImmediate::GreaterThan => data_type.get_compare_greater_than(mapped_scan_parameters),
                ScanCompareTypeImmediate::GreaterThanOrEqual => data_type.get_compare_greater_than_or_equal(mapped_scan_parameters),
                ScanCompareTypeImmediate::LessThan => data_type.get_compare_less_than(mapped_scan_parameters),
                ScanCompareTypeImmediate::LessThanOrEqual => data_type.get_compare_less_than_or_equal(mapped_scan_parameters),
            },
            None => None,
        }
    }

    pub fn get_scalar_compare_func_relative(
        &self,
        scan_compare_type: &ScanCompareTypeRelative,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnRelative> {
        match DataTypeRegistry::get_instance().get(self.get_data_type_id()) {
            Some(data_type) => match scan_compare_type {
                ScanCompareTypeRelative::Changed => data_type.get_compare_changed(mapped_scan_parameters),
                ScanCompareTypeRelative::Unchanged => data_type.get_compare_unchanged(mapped_scan_parameters),
                ScanCompareTypeRelative::Increased => data_type.get_compare_increased(mapped_scan_parameters),
                ScanCompareTypeRelative::Decreased => data_type.get_compare_decreased(mapped_scan_parameters),
            },
            None => None,
        }
    }

    pub fn get_scalar_compare_func_delta(
        &self,
        scan_compare_type: &ScanCompareTypeDelta,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnRelative> {
        match DataTypeRegistry::get_instance().get(self.get_data_type_id()) {
            Some(data_type) => match scan_compare_type {
                ScanCompareTypeDelta::IncreasedByX => data_type.get_compare_increased_by(mapped_scan_parameters),
                ScanCompareTypeDelta::DecreasedByX => data_type.get_compare_decreased_by(mapped_scan_parameters),
                ScanCompareTypeDelta::MultipliedByX => data_type.get_compare_multiplied_by(mapped_scan_parameters),
                ScanCompareTypeDelta::DividedByX => data_type.get_compare_divided_by(mapped_scan_parameters),
                ScanCompareTypeDelta::ModuloByX => data_type.get_compare_modulo_by(mapped_scan_parameters),
                ScanCompareTypeDelta::ShiftLeftByX => data_type.get_compare_shift_left_by(mapped_scan_parameters),
                ScanCompareTypeDelta::ShiftRightByX => data_type.get_compare_shift_right_by(mapped_scan_parameters),
                ScanCompareTypeDelta::LogicalAndByX => data_type.get_compare_logical_and_by(mapped_scan_parameters),
                ScanCompareTypeDelta::LogicalOrByX => data_type.get_compare_logical_or_by(mapped_scan_parameters),
                ScanCompareTypeDelta::LogicalXorByX => data_type.get_compare_logical_xor_by(mapped_scan_parameters),
            },
            None => None,
        }
    }

    pub fn get_vector_compare_func_immediate<const N: usize>(
        &self,
        scan_compare_type_immediate: &ScanCompareTypeImmediate,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
    {
        match DataTypeRegistry::get_instance().get(self.get_data_type_id()) {
            Some(data_type) => {
                <LaneCount<N> as VectorComparer<N>>::get_vector_compare_func_immediate(&data_type, &scan_compare_type_immediate, mapped_scan_parameters)
            }
            None => None,
        }
    }

    pub fn get_vector_compare_func_relative<const N: usize>(
        &self,
        scan_compare_type_relative: &ScanCompareTypeRelative,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
    {
        match DataTypeRegistry::get_instance().get(self.get_data_type_id()) {
            Some(data_type) => {
                <LaneCount<N> as VectorComparer<N>>::get_vector_compare_func_relative(&data_type, &scan_compare_type_relative, mapped_scan_parameters)
            }
            None => None,
        }
    }

    pub fn get_vector_compare_func_delta<const N: usize>(
        &self,
        scan_compare_type_delta: &ScanCompareTypeDelta,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
    {
        match DataTypeRegistry::get_instance().get(self.get_data_type_id()) {
            Some(data_type) => <LaneCount<N> as VectorComparer<N>>::get_vector_compare_func_delta(&data_type, &scan_compare_type_delta, mapped_scan_parameters),
            None => None,
        }
    }
}

impl FromStr for DataTypeRef {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let data_type_id = string;

        Ok(DataTypeRef::new(data_type_id))
    }
}

impl fmt::Display for DataTypeRef {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(formatter, "{}", self.get_data_type_id())
    }
}
