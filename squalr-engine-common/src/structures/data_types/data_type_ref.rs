use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnDelta;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnImmediate;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnRelative;
use crate::structures::data_types::comparisons::vector_compare::VectorCompare;
use crate::structures::data_values::anonymous_value::AnonymousValue;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::registries::data_types::data_type_registry::DataTypeRegistry;
use crate::structures::scanning::scan_compare_type_delta::ScanCompareTypeDelta;
use crate::structures::scanning::scan_compare_type_immediate::ScanCompareTypeImmediate;
use crate::structures::scanning::scan_compare_type_relative::ScanCompareTypeRelative;
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
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct DataTypeRef {
    data_type_id: String,
}

impl DataTypeRef {
    /// Creates a new reference to a registered `DataType`.
    pub fn new(data_type_id: &str) -> Self {
        Self {
            data_type_id: data_type_id.to_string(),
        }
    }

    /// Determines if the `DataType` this struct represents is currently registered and available.
    pub fn is_valid(&self) -> bool {
        let registry = DataTypeRegistry::get_instance().get_registry();

        registry.get(self.get_id()).is_some()
    }

    pub fn get_id(&self) -> &str {
        &self.data_type_id
    }

    pub fn get_icon_id(&self) -> String {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_id()) {
            Some(data_type) => data_type.get_icon_id().to_string(),
            None => String::new(),
        }
    }

    pub fn get_default_size_in_bytes(&self) -> u64 {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_id()) {
            Some(data_type) => data_type.get_default_size_in_bytes(),
            None => 0,
        }
    }

    pub fn deanonymize_value(
        &self,
        anonymous_value: &AnonymousValue,
    ) -> Option<DataValue> {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_id()) {
            Some(data_type) => Some(DataValue::new(self.clone(), data_type.deanonymize_value(anonymous_value))),
            None => None,
        }
    }

    pub fn deanonymize_value_little_endian(
        &self,
        anonymous_value: &AnonymousValue,
    ) -> Option<DataValue> {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_id()) {
            Some(data_type) => Some(DataValue::new(self.clone(), data_type.deanonymize_value_little_endian(anonymous_value))),
            None => None,
        }
    }

    pub fn get_default_value(&self) -> Option<DataValue> {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_id()) {
            Some(data_type) => Some(data_type.get_default_value()),
            None => None,
        }
    }

    pub fn get_scalar_compare_func_immediate(
        &self,
        scan_compare_type: &ScanCompareTypeImmediate,
    ) -> Option<ScalarCompareFnImmediate> {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_id()) {
            Some(data_type) => Some(match scan_compare_type {
                ScanCompareTypeImmediate::Equal => data_type.get_compare_equal(),
                ScanCompareTypeImmediate::NotEqual => data_type.get_compare_not_equal(),
                ScanCompareTypeImmediate::GreaterThan => data_type.get_compare_greater_than(),
                ScanCompareTypeImmediate::GreaterThanOrEqual => data_type.get_compare_greater_than_or_equal(),
                ScanCompareTypeImmediate::LessThan => data_type.get_compare_less_than(),
                ScanCompareTypeImmediate::LessThanOrEqual => data_type.get_compare_less_than_or_equal(),
            }),
            None => None,
        }
    }

    pub fn get_scalar_compare_func_relative(
        &self,
        scan_compare_type: &ScanCompareTypeRelative,
    ) -> Option<ScalarCompareFnRelative> {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_id()) {
            Some(data_type) => Some(match scan_compare_type {
                ScanCompareTypeRelative::Changed => data_type.get_compare_changed(),
                ScanCompareTypeRelative::Unchanged => data_type.get_compare_unchanged(),
                ScanCompareTypeRelative::Increased => data_type.get_compare_increased(),
                ScanCompareTypeRelative::Decreased => data_type.get_compare_decreased(),
            }),
            None => None,
        }
    }

    pub fn get_scalar_compare_func_delta(
        &self,
        scan_compare_type: &ScanCompareTypeDelta,
    ) -> Option<ScalarCompareFnDelta> {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_id()) {
            Some(data_type) => Some(match scan_compare_type {
                ScanCompareTypeDelta::IncreasedByX => data_type.get_compare_increased_by(),
                ScanCompareTypeDelta::DecreasedByX => data_type.get_compare_decreased_by(),
            }),
            None => None,
        }
    }

    pub fn get_vector_compare_func_immediate<const N: usize>(
        &self,
        scan_compare_type_immediate: &ScanCompareTypeImmediate,
    ) -> Option<unsafe fn(*const u8, *const u8) -> Simd<u8, N>>
    where
        LaneCount<N>: SupportedLaneCount + VectorCompare<N>,
    {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_id()) {
            Some(data_type) => Some(<LaneCount<N> as VectorCompare<N>>::get_vector_compare_func_immediate(
                &data_type,
                &scan_compare_type_immediate,
            )),
            None => None,
        }
    }

    pub fn get_vector_compare_func_relative<const N: usize>(
        &self,
        scan_compare_type_relative: &ScanCompareTypeRelative,
    ) -> Option<unsafe fn(*const u8, *const u8) -> Simd<u8, N>>
    where
        LaneCount<N>: SupportedLaneCount + VectorCompare<N>,
    {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_id()) {
            Some(data_type) => Some(<LaneCount<N> as VectorCompare<N>>::get_vector_compare_func_relative(
                &data_type,
                &scan_compare_type_relative,
            )),
            None => None,
        }
    }

    pub fn get_vector_compare_func_delta<const N: usize>(
        &self,
        scan_compare_type_delta: &ScanCompareTypeDelta,
    ) -> Option<unsafe fn(*const u8, *const u8, *const u8) -> Simd<u8, N>>
    where
        LaneCount<N>: SupportedLaneCount + VectorCompare<N>,
    {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_id()) {
            Some(data_type) => Some(<LaneCount<N> as VectorCompare<N>>::get_vector_compare_func_delta(
                &data_type,
                &scan_compare_type_delta,
            )),
            None => None,
        }
    }
}

impl FromStr for DataTypeRef {
    type Err = serde_json::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        Ok(DataTypeRef::new(string))
    }
}

impl fmt::Display for DataTypeRef {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(formatter, "{}", self.get_id())
    }
}
