use crate::registries::data_types::data_type_registry::DataTypeRegistry;
use crate::structures::data_types::data_type_meta_data::DataTypeMetaData;
use crate::structures::data_types::generics::vector_comparer::VectorComparer;
use crate::structures::data_values::anonymous_value::AnonymousValue;
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
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DataTypeRef {
    data_type_id: String,
    data_type_meta_data: DataTypeMetaData,
}

impl DataTypeRef {
    /// Creates a new reference to a registered `DataType` with the explicit
    pub fn new(
        data_type_id: &str,
        data_type_meta_data: DataTypeMetaData,
    ) -> Self {
        Self {
            data_type_id: data_type_id.to_string(),
            data_type_meta_data,
        }
    }

    /// Creates a new reference to a registered `DataType`. The type must be registered to collect important metadata.
    /// If the type is not yet registered, or does not exist, then this will return `None`.
    pub fn new_from_anonymous_value(
        data_type_id: &str,
        anonymous_value: &AnonymousValue,
    ) -> Self {
        let registry = DataTypeRegistry::get_instance().get_registry();
        let data_type_meta_data = match registry.get(data_type_id) {
            Some(data_type) => data_type.get_meta_data_for_anonymous_value(anonymous_value),
            None => {
                log::error!(
                    "Failed to resolve data type when initializing meta data from anonymous value: {}: {}",
                    data_type_id,
                    anonymous_value
                );
                DataTypeMetaData::None
            }
        };

        Self {
            data_type_id: data_type_id.to_string(),
            data_type_meta_data,
        }
    }

    /// Creates a new reference to a registered `DataType`. The type must be registered to collect important metadata.
    /// If the type is not yet registered, or does not exist, then this will return `None`.
    pub fn new_from_data_type_defaults(data_type_id: &str) -> Self {
        let registry = DataTypeRegistry::get_instance().get_registry();

        let data_type_meta_data = match registry.get(data_type_id) {
            Some(data_type) => data_type.get_default_meta_data(),
            None => {
                log::error!("Failed to resolve data type when initializing defaultmeta data: {}", data_type_id);
                DataTypeMetaData::None
            }
        };

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

    pub fn get_meta_data(&self) -> &DataTypeMetaData {
        &self.data_type_meta_data
    }

    pub fn get_icon_id(&self) -> String {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_data_type_id()) {
            Some(data_type) => data_type.get_icon_id().to_string(),
            None => String::new(),
        }
    }

    pub fn get_size_in_bytes(&self) -> u64 {
        match &self.data_type_meta_data {
            // For standard types, return the default / primitive size from the data type in the registry.
            DataTypeMetaData::None | DataTypeMetaData::Primitive(_) => {
                let registry = DataTypeRegistry::get_instance().get_registry();

                match registry.get(self.get_data_type_id()) {
                    Some(data_type) => data_type.get_default_size_in_bytes(),
                    None => 0,
                }
            }
            // For container types, return the size of the container.
            DataTypeMetaData::SizedContainer(size) => *size,
            // For encoded string types, return the size of the container.
            DataTypeMetaData::EncodedString(size, _encoding) => *size,
            // For fixed string types, return the size of the string.
            DataTypeMetaData::FixedString(string) => string.len() as u64,
        }
    }

    pub fn deanonymize_value(
        &self,
        anonymous_value: &AnonymousValue,
    ) -> Result<DataValue, String> {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_data_type_id()) {
            Some(data_type) => {
                let deanonymized_value = data_type.deanonymize_value(anonymous_value, self.clone());

                match deanonymized_value {
                    Ok(value) => Ok(value),
                    Err(err) => Err(err.to_string()),
                }
            }
            None => Err("Cannot deanonymize value: data type is not registered.".into()),
        }
    }

    pub fn is_discrete(&self) -> bool {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_data_type_id()) {
            Some(data_type) => data_type.is_discrete(),
            None => false,
        }
    }

    pub fn get_default_value(&self) -> Option<DataValue> {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_data_type_id()) {
            Some(data_type) => Some(data_type.get_default_value(self.clone())),
            None => None,
        }
    }

    pub fn get_scalar_compare_func_immediate(
        &self,
        scan_compare_type: &ScanCompareTypeImmediate,
        scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnImmediate> {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_data_type_id()) {
            Some(data_type) => match scan_compare_type {
                ScanCompareTypeImmediate::Equal => data_type.get_compare_equal(scan_parameters),
                ScanCompareTypeImmediate::NotEqual => data_type.get_compare_not_equal(scan_parameters),
                ScanCompareTypeImmediate::GreaterThan => data_type.get_compare_greater_than(scan_parameters),
                ScanCompareTypeImmediate::GreaterThanOrEqual => data_type.get_compare_greater_than_or_equal(scan_parameters),
                ScanCompareTypeImmediate::LessThan => data_type.get_compare_less_than(scan_parameters),
                ScanCompareTypeImmediate::LessThanOrEqual => data_type.get_compare_less_than_or_equal(scan_parameters),
            },
            None => None,
        }
    }

    pub fn get_scalar_compare_func_relative(
        &self,
        scan_compare_type: &ScanCompareTypeRelative,
        scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnRelative> {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_data_type_id()) {
            Some(data_type) => match scan_compare_type {
                ScanCompareTypeRelative::Changed => data_type.get_compare_changed(scan_parameters),
                ScanCompareTypeRelative::Unchanged => data_type.get_compare_unchanged(scan_parameters),
                ScanCompareTypeRelative::Increased => data_type.get_compare_increased(scan_parameters),
                ScanCompareTypeRelative::Decreased => data_type.get_compare_decreased(scan_parameters),
            },
            None => None,
        }
    }

    pub fn get_scalar_compare_func_delta(
        &self,
        scan_compare_type: &ScanCompareTypeDelta,
        scan_parameters: &MappedScanParameters,
    ) -> Option<ScalarCompareFnRelative> {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_data_type_id()) {
            Some(data_type) => match scan_compare_type {
                ScanCompareTypeDelta::IncreasedByX => data_type.get_compare_increased_by(scan_parameters),
                ScanCompareTypeDelta::DecreasedByX => data_type.get_compare_decreased_by(scan_parameters),
                ScanCompareTypeDelta::MultipliedByX => data_type.get_compare_multiplied_by(scan_parameters),
                ScanCompareTypeDelta::DividedByX => data_type.get_compare_divided_by(scan_parameters),
                ScanCompareTypeDelta::ModuloByX => data_type.get_compare_modulo_by(scan_parameters),
                ScanCompareTypeDelta::ShiftLeftByX => data_type.get_compare_shift_left_by(scan_parameters),
                ScanCompareTypeDelta::ShiftRightByX => data_type.get_compare_shift_right_by(scan_parameters),
                ScanCompareTypeDelta::LogicalAndByX => data_type.get_compare_logical_and_by(scan_parameters),
                ScanCompareTypeDelta::LogicalOrByX => data_type.get_compare_logical_or_by(scan_parameters),
                ScanCompareTypeDelta::LogicalXorByX => data_type.get_compare_logical_xor_by(scan_parameters),
            },
            None => None,
        }
    }

    pub fn get_vector_compare_func_immediate<const N: usize>(
        &self,
        scan_compare_type_immediate: &ScanCompareTypeImmediate,
        scan_parameters: &MappedScanParameters,
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
    {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_data_type_id()) {
            Some(data_type) => {
                <LaneCount<N> as VectorComparer<N>>::get_vector_compare_func_immediate(&data_type, &scan_compare_type_immediate, scan_parameters)
            }
            None => None,
        }
    }

    pub fn get_vector_compare_func_relative<const N: usize>(
        &self,
        scan_compare_type_relative: &ScanCompareTypeRelative,
        scan_parameters: &MappedScanParameters,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
    {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_data_type_id()) {
            Some(data_type) => <LaneCount<N> as VectorComparer<N>>::get_vector_compare_func_relative(&data_type, &scan_compare_type_relative, scan_parameters),
            None => None,
        }
    }

    pub fn get_vector_compare_func_delta<const N: usize>(
        &self,
        scan_compare_type_delta: &ScanCompareTypeDelta,
        scan_parameters: &MappedScanParameters,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
    {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(self.get_data_type_id()) {
            Some(data_type) => <LaneCount<N> as VectorComparer<N>>::get_vector_compare_func_delta(&data_type, &scan_compare_type_delta, scan_parameters),
            None => None,
        }
    }
}

impl FromStr for DataTypeRef {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = string.splitn(2, ';').collect();

        if parts.len() <= 0 {
            return Err("Invalid data type ref format, expected {data_type};{conditional_data_type_meta_data}".into());
        }

        let data_type_id = parts[0];

        // Parse out any sized container data if it was present.
        let data_type_meta_data = if parts.len() < 2 {
            DataTypeMetaData::None
        } else {
            let registry = DataTypeRegistry::get_instance().get_registry();
            match registry.get(data_type_id) {
                Some(data_type) => data_type.get_meta_data_from_string(parts[1])?,
                None => {
                    return Err(format!("Failed to resolve data type when parsing meta data: {}", data_type_id));
                }
            }
        };

        Ok(DataTypeRef::new(data_type_id, data_type_meta_data))
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
