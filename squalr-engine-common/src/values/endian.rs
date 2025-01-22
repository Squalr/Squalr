use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Endian {
    Little,
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
