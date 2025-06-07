use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Represents additional data about a `DataType` that may further differentiate it.
/// For example, an array of bytes is a data type, but each instance has a specified length.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DataTypeMetaData {
    /// Represents no special meta data for the underlying data type.
    None,

    /// Represents meta data for whether the data type is primitive, and how many elements it contains (ie an array).
    Primitive(u64),

    /// Represents a container (ie a string) of a specified byte-wise size for the underlying data type.
    SizedContainer(u64),

    /// Represents a known, fixed string. Used for referential data types, such as the 'data type data type'.
    FixedString(String),
}

impl fmt::Display for DataTypeMetaData {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            DataTypeMetaData::None => write!(formatter, "none"),
            DataTypeMetaData::Primitive(count) => write!(formatter, "primitive({})", count),
            DataTypeMetaData::SizedContainer(size) => write!(formatter, "sized_container({})", size),
            DataTypeMetaData::FixedString(s) => write!(formatter, "fixed_string({})", s),
        }
    }
}

impl FromStr for DataTypeMetaData {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if string.eq_ignore_ascii_case("none") {
            Ok(DataTypeMetaData::None)
        } else if let Some(inner) = string
            .strip_prefix("primitive(")
            .and_then(|string| string.strip_suffix(')'))
        {
            let count = inner
                .parse::<u64>()
                .map_err(|err| format!("Failed to parse primitive count: {}", err))?;
            Ok(DataTypeMetaData::Primitive(count))
        } else if let Some(inner) = string
            .strip_prefix("sized_container(")
            .and_then(|string| string.strip_suffix(')'))
        {
            let size = inner
                .parse::<u64>()
                .map_err(|err| format!("Failed to parse sized_container size: {}", err))?;
            Ok(DataTypeMetaData::SizedContainer(size))
        } else if let Some(inner) = string
            .strip_prefix("fixed_string(")
            .and_then(|string| string.strip_suffix(')'))
        {
            Ok(DataTypeMetaData::FixedString(inner.to_string()))
        } else {
            Err(format!("Unknown data type metadata format: {}", string))
        }
    }
}
