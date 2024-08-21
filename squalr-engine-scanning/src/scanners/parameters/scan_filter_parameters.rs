use squalr_engine_memory::memory_alignment::MemoryAlignment;
use squalr_engine_common::values::data_type::DataType;
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

/// Defines a set of parameters for scan filters, which can be considered as "windows" into a snapshot that 
/// are used to aggregate scan results for a given data type and alignment.
#[derive(Debug, Clone)]
pub struct ScanFilterParameters {
    alignment: Option<MemoryAlignment>,
    data_type: DataType,
}

impl Default for ScanFilterParameters {
    fn default(
    ) -> Self {
        ScanFilterParameters::new()
    }
}

impl ScanFilterParameters {
    pub fn new() -> Self {
        Self {
            alignment: None,
            data_type: DataType::default(),
        }
    }

    pub fn new_with_value(
        alignment: Option<MemoryAlignment>,
        data_type: DataType,
    ) -> Self {
        Self {
            alignment: alignment,
            data_type: data_type,
        }
    }

    pub fn get_memory_alignment(
        &self
    ) -> &Option<MemoryAlignment>{
        return &self.alignment;
    }

    pub fn get_memory_alignment_or_default(
        &self,
        data_type: &DataType,
    ) -> MemoryAlignment{
        if let Some(alignment) = &self.alignment {
            return alignment.to_owned();
        }

        return MemoryAlignment::from(data_type.size_in_bytes() as i32);
    }

    pub fn get_data_type(
        &self
    ) -> &DataType{
        return &self.data_type;
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
        f: &mut fmt::Formatter<'_>
    ) -> fmt::Result {
        match self {
            ScanFilterParametersParseError::InvalidFormat => write!(f, "Invalid format"),
            ScanFilterParametersParseError::InvalidAlignment(e) => write!(f, "Invalid alignment: {}", e),
            ScanFilterParametersParseError::InvalidDataType => write!(f, "Invalid data type"),
        }
    }
}

impl From<ParseIntError> for ScanFilterParametersParseError {
    fn from(
        e: ParseIntError
    ) -> Self {
        ScanFilterParametersParseError::InvalidAlignment(e)
    }
}

impl FromStr for ScanFilterParameters {
    type Err = ScanFilterParametersParseError;

    fn from_str(
        s: &str
    ) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('=').collect();

        // Check if there is at least one part, and at most two
        if parts.len() < 1 || parts.len() > 2 {
            return Err(ScanFilterParametersParseError::InvalidFormat);
        }

        // Parse the data type from the first part
        let data_type = parts[0].trim().parse::<DataType>()
            .map_err(|_| ScanFilterParametersParseError::InvalidDataType)?;

        // Handle the optional alignment part
        let alignment = if parts.len() == 2 {
            match parts[1].trim() {
                "" => None,  // No alignment provided
                alignment_str => {
                    let alignment_value: i32 = alignment_str.parse()?;
                    Some(MemoryAlignment::from(alignment_value))
                }
            }
        } else {
            None
        };

        // Create a new ScanFilterParameters with the parsed values
        Ok(ScanFilterParameters::new_with_value(alignment, data_type))
    }
}
