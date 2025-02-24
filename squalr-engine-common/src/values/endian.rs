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
        if cfg!(target_endian = "little") {
            return Endian::Little;
        }

        return Endian::Big;
    }
}

impl fmt::Display for Endian {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Endian::Little => write!(f, "le"),
            Endian::Big => write!(f, "be"),
        }
    }
}
