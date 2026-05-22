use crate::structures::details::{DetailsFieldId, DetailsFieldSource, DetailsTarget, DetailsValue};
use serde::{Deserialize, Serialize};

/// A semantic operation produced after planning a details edit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DetailsEditOperation {
    Noop {
        reason: Option<String>,
    },
    Reject {
        reason: String,
    },
    UpdateStoredField {
        target: DetailsTarget,
        source: DetailsFieldSource,
        value: DetailsValue,
    },
    WriteRuntimeValue {
        target: DetailsTarget,
        field_id: DetailsFieldId,
        source: DetailsFieldSource,
        value: DetailsValue,
    },
    RefreshProjection {
        target: DetailsTarget,
    },
}

/// Ordered operations needed to apply one details edit.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DetailsEditPlan {
    operations: Vec<DetailsEditOperation>,
}

impl DetailsEditPlan {
    pub fn new(operations: Vec<DetailsEditOperation>) -> Self {
        Self { operations }
    }

    pub fn noop(reason: Option<String>) -> Self {
        Self {
            operations: vec![DetailsEditOperation::Noop { reason }],
        }
    }

    pub fn reject(reason: impl Into<String>) -> Self {
        Self {
            operations: vec![DetailsEditOperation::Reject { reason: reason.into() }],
        }
    }

    pub fn get_operations(&self) -> &[DetailsEditOperation] {
        &self.operations
    }
}
