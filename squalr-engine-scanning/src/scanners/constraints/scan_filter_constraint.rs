use squalr_engine_memory::memory_alignment::MemoryAlignment;
use squalr_engine_common::values::data_type::DataType;
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct ScanFilterConstraint {
    alignment: Option<MemoryAlignment>,
    data_type: DataType,
}

impl Default for ScanFilterConstraint {
    fn default(
    ) -> Self {
        ScanFilterConstraint::new()
    }
}

impl ScanFilterConstraint {
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
pub enum ScanFilterConstraintParseError {
    InvalidFormat,
    InvalidAlignment(ParseIntError),
    InvalidDataType,
}

impl fmt::Display for ScanFilterConstraintParseError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>
    ) -> fmt::Result {
        match self {
            ScanFilterConstraintParseError::InvalidFormat => write!(f, "Invalid format"),
            ScanFilterConstraintParseError::InvalidAlignment(e) => write!(f, "Invalid alignment: {}", e),
            ScanFilterConstraintParseError::InvalidDataType => write!(f, "Invalid data type"),
        }
    }
}

impl From<ParseIntError> for ScanFilterConstraintParseError {
    fn from(
        e: ParseIntError
    ) -> Self {
        ScanFilterConstraintParseError::InvalidAlignment(e)
    }
}

impl FromStr for ScanFilterConstraint {
    type Err = ScanFilterConstraintParseError;

    fn from_str(
        s: &str
    ) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('=').collect();

        // Check if there is at least one part, and at most two
        if parts.len() < 1 || parts.len() > 2 {
            return Err(ScanFilterConstraintParseError::InvalidFormat);
        }

        // Parse the data type from the first part
        let data_type = parts[0].trim().parse::<DataType>()
            .map_err(|_| ScanFilterConstraintParseError::InvalidDataType)?;

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

        // Create a new ScanFilterConstraint with the parsed values
        Ok(ScanFilterConstraint::new_with_value(alignment, data_type))
    }
}
