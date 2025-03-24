use crate::registries::data_types::data_type_registry::DataTypeRegistry;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnDelta;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnImmediate;
use crate::structures::data_types::comparisons::scalar_comparable::ScalarCompareFnRelative;
use crate::structures::data_types::data_type_meta_data::DataTypeMetaData;
use crate::structures::data_types::generics::vector_comparer::VectorComparer;
use crate::structures::data_values::anonymous_value::AnonymousValue;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::scanning::comparisons::scan_compare_type_delta::ScanCompareTypeDelta;
use crate::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use crate::structures::scanning::comparisons::scan_compare_type_relative::ScanCompareTypeRelative;
use crate::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use crate::structures::scanning::scan_parameters_local::ScanParametersLocal;
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
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DataTypeRef {
    data_type_id: String,
    data_type_meta_data: DataTypeMetaData,
}

impl DataTypeRef {
    /// Creates a new reference to a registered `DataType`. The type must be registered to collect important metadata.
    /// If the type is not yet registered, or does not exist, then this will return `None`.
    pub fn new(
        data_type_id: &str,
        data_type_meta_data: DataTypeMetaData,
    ) -> Self {
        Self {
            data_type_id: data_type_id.to_string(),
            data_type_meta_data,
        }
    }

    /// Determines if the `DataType` this struct represents is currently registered and available.
    pub fn is_valid(&self) -> bool {
        let registry = DataTypeRegistry::get_instance().get_registry();

        registry.get(self.get_data_type_id()).is_some()
    }

    pub fn get_data_type_id(&self) -> &str {
        &self.data_type_id
    }

    pub fn get_icon_id(&self) -> String {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_data_type_id()) {
            Some(data_type) => data_type.get_icon_id().to_string(),
            None => String::new(),
        }
    }

    pub fn get_size_in_bytes(&self) -> u64 {
        match self.data_type_meta_data {
            // For standard types, return the default / primitive size from the data type in the registry.
            DataTypeMetaData::None => {
                let registry = DataTypeRegistry::get_instance().get_registry();

                match registry.get(self.get_data_type_id()) {
                    Some(data_type) => data_type.get_default_size_in_bytes(),
                    None => 0,
                }
            }
            // For container types, return the size of the container.
            DataTypeMetaData::SizedContainer(size) => size,
        }
    }

    pub fn deanonymize_value(
        &self,
        anonymous_value: &AnonymousValue,
    ) -> Result<DataValue, String> {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_data_type_id()) {
            Some(data_type) => {
                let deanonymized_value = data_type.deanonymize_value(anonymous_value);

                match deanonymized_value {
                    Ok(value) => Ok(DataValue::new(self.get_data_type_id(), value)),
                    Err(err) => Err(err.to_string()),
                }
            }
            None => Err("Cannot deanonymize value: data type is not registered.".into()),
        }
    }

    pub fn get_default_value(&self) -> Option<DataValue> {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_data_type_id()) {
            Some(data_type) => Some(data_type.get_default_value(&self.data_type_meta_data)),
            None => None,
        }
    }

    pub fn get_scalar_compare_func_immediate(
        &self,
        scan_compare_type: &ScanCompareTypeImmediate,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnImmediate> {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_data_type_id()) {
            Some(data_type) => match scan_compare_type {
                ScanCompareTypeImmediate::Equal => data_type.get_compare_equal(scan_parameters_global, scan_parameters_local),
                ScanCompareTypeImmediate::NotEqual => data_type.get_compare_not_equal(scan_parameters_global, scan_parameters_local),
                ScanCompareTypeImmediate::GreaterThan => data_type.get_compare_greater_than(scan_parameters_global, scan_parameters_local),
                ScanCompareTypeImmediate::GreaterThanOrEqual => data_type.get_compare_greater_than_or_equal(scan_parameters_global, scan_parameters_local),
                ScanCompareTypeImmediate::LessThan => data_type.get_compare_less_than(scan_parameters_global, scan_parameters_local),
                ScanCompareTypeImmediate::LessThanOrEqual => data_type.get_compare_less_than_or_equal(scan_parameters_global, scan_parameters_local),
            },
            None => None,
        }
    }

    pub fn get_scalar_compare_func_relative(
        &self,
        scan_compare_type: &ScanCompareTypeRelative,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnRelative> {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_data_type_id()) {
            Some(data_type) => match scan_compare_type {
                ScanCompareTypeRelative::Changed => data_type.get_compare_changed(scan_parameters_global, scan_parameters_local),
                ScanCompareTypeRelative::Unchanged => data_type.get_compare_unchanged(scan_parameters_global, scan_parameters_local),
                ScanCompareTypeRelative::Increased => data_type.get_compare_increased(scan_parameters_global, scan_parameters_local),
                ScanCompareTypeRelative::Decreased => data_type.get_compare_decreased(scan_parameters_global, scan_parameters_local),
            },
            None => None,
        }
    }

    pub fn get_scalar_compare_func_delta(
        &self,
        scan_compare_type: &ScanCompareTypeDelta,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnDelta> {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_data_type_id()) {
            Some(data_type) => match scan_compare_type {
                ScanCompareTypeDelta::IncreasedByX => data_type.get_compare_increased_by(scan_parameters_global, scan_parameters_local),
                ScanCompareTypeDelta::DecreasedByX => data_type.get_compare_decreased_by(scan_parameters_global, scan_parameters_local),
            },
            None => None,
        }
    }

    pub fn get_vector_compare_func_immediate<const N: usize>(
        &self,
        scan_compare_type_immediate: &ScanCompareTypeImmediate,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
    {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_data_type_id()) {
            Some(data_type) => <LaneCount<N> as VectorComparer<N>>::get_vector_compare_func_immediate(
                &data_type,
                &scan_compare_type_immediate,
                scan_parameters_global,
                scan_parameters_local,
            ),
            None => None,
        }
    }

    pub fn get_vector_compare_func_relative<const N: usize>(
        &self,
        scan_compare_type_relative: &ScanCompareTypeRelative,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
    {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_data_type_id()) {
            Some(data_type) => <LaneCount<N> as VectorComparer<N>>::get_vector_compare_func_relative(
                &data_type,
                &scan_compare_type_relative,
                scan_parameters_global,
                scan_parameters_local,
            ),
            None => None,
        }
    }

    pub fn get_vector_compare_func_delta<const N: usize>(
        &self,
        scan_compare_type_delta: &ScanCompareTypeDelta,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
    {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_data_type_id()) {
            Some(data_type) => <LaneCount<N> as VectorComparer<N>>::get_vector_compare_func_delta(
                &data_type,
                &scan_compare_type_delta,
                scan_parameters_global,
                scan_parameters_local,
            ),
            None => None,
        }
    }
}

impl FromStr for DataTypeRef {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = string.split(';').collect();

        if parts.len() <= 0 {
            return Err("Invalid data type ref format, expected {data_type}{;optional_container_size}".into());
        }

        // Parse out any sized container data if it was present.
        let data_type_meta_data = if parts.len() < 2 {
            DataTypeMetaData::None
        } else {
            DataTypeMetaData::SizedContainer(match parts[1].trim().parse::<u64>() {
                Ok(container_size) => container_size,
                Err(err) => {
                    return Err(format!("Failed to parse address: {}", err));
                }
            })
        };

        Ok(DataTypeRef::new(parts[0], data_type_meta_data))
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
