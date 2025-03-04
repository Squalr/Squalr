use crate::structures::data_types::data_type::DataType;
use crate::structures::memory_alignment::MemoryAlignment;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

/// Defines a set of parameters for scan filters, which can be considered as "windows" into a snapshot that
/// are used to aggregate scan results for a given data type and alignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanFilterParameters {
    alignment: Option<MemoryAlignment>,
    data_type: Box<dyn DataType>,
}

impl ScanFilterParameters {
    pub fn new(
        alignment: Option<MemoryAlignment>,
        data_type: Box<dyn DataType>,
    ) -> Self {
        Self { alignment, data_type }
    }

    pub fn get_memory_alignment(&self) -> &Option<MemoryAlignment> {
        &self.alignment
    }

    pub fn get_memory_alignment_or_default(&self) -> MemoryAlignment {
        if let Some(alignment) = &self.alignment {
            alignment.to_owned()
        } else {
            // Squalr is fast, so we can just default to an alignment of 1 to prevent missing anything important.
            MemoryAlignment::Alignment1
        }
    }

    pub fn get_data_type(&self) -> &Box<dyn DataType> {
        &self.data_type
    }
}

#[derive(Debug)]
pub enum ScanFilterParametersParseError {
    InvalidFormat,
    InvalidAlignment(ParseIntError),
    InvalidDataType,
}

impl fmt::Display for ScanFilterParametersParseError {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            ScanFilterParametersParseError::InvalidFormat => write!(formatter, "Invalid format"),
            ScanFilterParametersParseError::InvalidAlignment(err) => write!(formatter, "Invalid alignment: {}", err),
            ScanFilterParametersParseError::InvalidDataType => write!(formatter, "Invalid data type"),
        }
    }
}

impl From<ParseIntError> for ScanFilterParametersParseError {
    fn from(e: ParseIntError) -> Self {
        ScanFilterParametersParseError::InvalidAlignment(e)
    }
}

impl FromStr for ScanFilterParameters {
    type Err = ScanFilterParametersParseError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = string.split('=').collect();

        // Check if there is at least one part, and at most two
        if parts.len() < 1 || parts.len() > 2 {
            return Err(ScanFilterParametersParseError::InvalidFormat);
        }

        // Parse the data type from the first part
        let data_type = parts[0]
            .trim()
            .parse::<Box<dyn DataType>>()
            .map_err(|_| ScanFilterParametersParseError::InvalidDataType)?;

        // Handle the optional alignment part
        let alignment = if parts.len() == 2 {
            match parts[1].trim() {
                // No alignment provided
                "" => None,
                alignment_str => {
                    let alignment_value: i32 = alignment_str.parse()?;
                    Some(MemoryAlignment::from(alignment_value))
                }
            }
        } else {
            None
        };

        // Create a new ScanFilterParameters with the parsed values
        Ok(ScanFilterParameters::new(alignment, data_type))
    }
}
