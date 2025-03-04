use crate::structures::data_types::comparisons::scalar_comparable::ScalarComparable;
use crate::structures::data_types::comparisons::vector_comparable::VectorComparable;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::endian::Endian;
use crate::structures::registries::data_type_registry::DataTypeRegistry;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::fmt::{self, Debug};
use std::str::FromStr;

/// Defines a generic scannable data type.
pub trait DataType: Debug + Send + Sync + ScalarComparable + VectorComparable {
    fn get_name(&self) -> &str;
    fn get_size_in_bytes(&self) -> u64;
    fn get_endian(&self) -> Endian;
    fn to_default_value(&self) -> Box<dyn DataValue>;

    fn clone_internal(&self) -> Box<dyn DataType>;
    fn serialize_internal(&self) -> Value;

    fn partial_eq_internal(
        &self,
        other: &Box<dyn DataType>,
    ) -> bool {
        self.get_name() == other.get_name()
    }
}

impl Clone for Box<dyn DataType> {
    fn clone(&self) -> Box<dyn DataType> {
        self.clone_internal()
    }
}

impl Serialize for Box<dyn DataType> {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("type_name", self.get_name())?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for Box<dyn DataType> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Value = Deserialize::deserialize(deserializer)?;

        // Extract type_name and data
        let type_name = value
            .get("type_name")
            .and_then(Value::as_str)
            .ok_or_else(|| serde::de::Error::missing_field("type_name"))?;

        // Lookup data type from registry.
        let registry = DataTypeRegistry::get_instance().get_registry();
        let data_type = registry.get(type_name).ok_or_else(|| {
            serde::de::Error::unknown_variant(type_name, &["i32" /*, other types... */]) // JIRA
        })?;

        Ok(data_type.value().clone())
    }
}

impl PartialEq for Box<dyn DataType> {
    fn eq(
        &self,
        other: &Box<dyn DataType>,
    ) -> bool {
        self.partial_eq_internal(other)
    }
}

impl FromStr for Box<dyn DataType> {
    type Err = serde_json::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let registry = DataTypeRegistry::get_instance().get_registry();
        let data_type = registry.get(string).ok_or_else(|| {
            serde::de::Error::unknown_variant(string, &["i32" /*, other types... */]) // JIRA
        })?;

        Ok(data_type.value().clone())
    }
}

impl fmt::Display for dyn DataType {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(formatter, "{}", self.get_name())
    }
}
