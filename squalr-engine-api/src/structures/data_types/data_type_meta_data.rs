use serde::{Deserialize, Serialize};

/// Represents additional data about a `DataType` that may further differentiate it.
/// For example, an array of bytes is a data type, but each instance has a specified length.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DataTypeMetaData {
    /// Represents no special meta data for the underlying data type.
    None,

    /// Represents meta data for whether a data value is displayed as hex.
    Primitive(),

    /// Represents a container (ie byte[]) of a specified byte-wise size for the underlying data type.
    SizedContainer(u64),

    /// Represents a known, fixed string. Used for referential data types, such as the 'data type data type'.
    FixedString(String),
}
