use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Debug},
    hash::{Hash, Hasher},
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

    pub fn get_data_type_id(&self) -> &str {
        &self.data_type_id
    }
}

impl Hash for DataTypeRef {
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    ) {
        self.data_type_id.hash(state);
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
