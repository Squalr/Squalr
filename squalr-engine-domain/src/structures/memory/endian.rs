use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Endian {
    /// Little endian is the default for most systems.
    Little,

    /// Big endian is the default for some systems, such as GameCube (and thus any emulators for these systems).
    Big,
}

impl Default for Endian {
    fn default() -> Self {
        Endian::Little
    }
}

impl fmt::Display for Endian {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Endian::Little => write!(formatter, "le"),
            Endian::Big => write!(formatter, "be"),
        }
    }
}
