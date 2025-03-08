use serde::{Deserialize, Serialize};

/// Represents additional data about a `DataType` that may further differentiate it.
/// For example, an array of bytes is a data type, but each instance has a specified length.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DataTypeMetaData {
    None,
    SizedContainer(u64),
}
