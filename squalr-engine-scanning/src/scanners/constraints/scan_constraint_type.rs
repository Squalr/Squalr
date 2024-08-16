use std::str::FromStr;
use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintType {
    Equal,
    NotEqual,
    Changed,
    Unchanged,
    Increased,
    Decreased,
    IncreasedByX,
    DecreasedByX,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

impl FromStr for ConstraintType {
    type Err = ParseConstraintTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "==" => Ok(ConstraintType::Equal),
            "!=" => Ok(ConstraintType::NotEqual),
            "c" => Ok(ConstraintType::Changed),
            "u" => Ok(ConstraintType::Unchanged),
            "+" => Ok(ConstraintType::Increased),
            "-" => Ok(ConstraintType::Decreased),
            "+x" => Ok(ConstraintType::IncreasedByX),
            "-x" => Ok(ConstraintType::DecreasedByX),
            ">" => Ok(ConstraintType::GreaterThan),
            ">=" => Ok(ConstraintType::GreaterThanOrEqual),
            "<" => Ok(ConstraintType::LessThan),
            "<=" => Ok(ConstraintType::LessThanOrEqual),
            _ => Err(ParseConstraintTypeError),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseConstraintTypeError;

impl fmt::Display for ParseConstraintTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid constraint type")
    }
}

impl std::error::Error for ParseConstraintTypeError {}

impl Display for ConstraintType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ConstraintType::Equal => "Equal",
                ConstraintType::NotEqual => "Not Equal",
                ConstraintType::Changed => "Changed",
                ConstraintType::Unchanged => "Unchanged",
                ConstraintType::Increased => "Increased",
                ConstraintType::Decreased => "Decreased",
                ConstraintType::IncreasedByX => "Increased By X",
                ConstraintType::DecreasedByX => "Decreased By X",
                ConstraintType::GreaterThan => "Greater Than",
                ConstraintType::GreaterThanOrEqual => "Greater Than Or Equal",
                ConstraintType::LessThan => "Less Than",
                ConstraintType::LessThanOrEqual => "Less Than Or Equal",
            }
        )
    }
}
