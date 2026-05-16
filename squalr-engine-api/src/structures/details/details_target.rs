use serde::{Deserialize, Serialize};
use std::fmt;

/// Identifies the object being inspected by a details projection.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DetailsTarget {
    target_kind: String,
    target_id: String,
}

impl DetailsTarget {
    pub fn new(
        target_kind: impl Into<String>,
        target_id: impl Into<String>,
    ) -> Self {
        Self {
            target_kind: target_kind.into(),
            target_id: target_id.into(),
        }
    }

    pub fn get_target_kind(&self) -> &str {
        &self.target_kind
    }

    pub fn get_target_id(&self) -> &str {
        &self.target_id
    }
}

impl fmt::Display for DetailsTarget {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(formatter, "{}:{}", self.target_kind, self.target_id)
    }
}
