use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::memory_alignment::MemoryAlignment;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

/// Defines a unique pair of `DataType` and `MemoryAlignment` used in a scan within a larger scan job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanParametersLocal {
    data_type: DataTypeRef,
    alignment: Option<MemoryAlignment>,
}

impl ScanParametersLocal {
    pub fn new(
        data_type: DataTypeRef,
        alignment: Option<MemoryAlignment>,
    ) -> Self {
        Self { data_type, alignment }
    }

    pub fn get_memory_alignment(&self) -> &Option<MemoryAlignment> {
        &self.alignment
    }

    pub fn get_memory_alignment_or_default(&self) -> MemoryAlignment {
        if let Some(alignment) = self.alignment {
            alignment
        } else {
            // Squalr is fast, so we can just default to an alignment of 1 to prevent missing anything important.
            MemoryAlignment::Alignment1
        }
    }

    pub fn get_data_type(&self) -> &DataTypeRef {
        &self.data_type
    }
}

#[derive(Debug)]
pub enum ScanParametersLocalParseError {
    InvalidFormat,
    InvalidAlignment(ParseIntError),
    InvalidDataType,
}

impl fmt::Display for ScanParametersLocalParseError {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            ScanParametersLocalParseError::InvalidFormat => write!(formatter, "Invalid format"),
            ScanParametersLocalParseError::InvalidAlignment(err) => write!(formatter, "Invalid alignment: {}", err),
            ScanParametersLocalParseError::InvalidDataType => write!(formatter, "Invalid data type"),
        }
    }
}

impl From<ParseIntError> for ScanParametersLocalParseError {
    fn from(e: ParseIntError) -> Self {
        ScanParametersLocalParseError::InvalidAlignment(e)
    }
}

impl FromStr for ScanParametersLocal {
    type Err = ScanParametersLocalParseError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = string.split('=').collect();

        // Check if there is at least one part, and at most two.
        if parts.len() < 1 || parts.len() > 2 {
            return Err(ScanParametersLocalParseError::InvalidFormat);
        }

        // Parse the data type from the first part.
        let data_type = parts[0]
            .trim()
            .parse::<DataTypeRef>()
            .map_err(|_| ScanParametersLocalParseError::InvalidDataType)?;

        // Handle the optional alignment part.
        let alignment = if parts.len() == 2 {
            match parts[1].trim() {
                // No alignment provided.
                "" => None,
                alignment_str => {
                    let alignment_value: i32 = alignment_str.parse()?;
                    Some(MemoryAlignment::from(alignment_value))
                }
            }
        } else {
            None
        };

        // Create a new ScanParametersLocal with the parsed values.
        Ok(ScanParametersLocal::new(data_type, alignment))
    }
}
