use crate::structures::data_types::built_in_types::string::string_encodings::StringEncoding;
use serde::{Deserialize, Serialize};

/// Represents additional data about a `DataType` that may further differentiate it.
/// For example, an array of bytes is a data type, but each instance has a specified length.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DataTypeMetaData {
    /// Represents no special meta data for the underlying data type.
    None,

    /// Represents a container (ie byte[]) of a specified byte-wise size for the underlying data type.
    SizedContainer(u64),

    /// Represents a string of a specified length with a specified encoding. Specific to string data types.
    EncodedString(u64, StringEncoding),
}
