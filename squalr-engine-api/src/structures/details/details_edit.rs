use crate::structures::details::{DetailsFieldId, DetailsFieldSource, DetailsTarget, DetailsValue};
use serde::{Deserialize, Serialize};

/// A user edit against a stable details field id.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DetailsEdit {
    target: DetailsTarget,
    field_id: DetailsFieldId,
    source: DetailsFieldSource,
    value: DetailsValue,
}

impl DetailsEdit {
    pub fn new(
        target: DetailsTarget,
        field_id: DetailsFieldId,
        value: DetailsValue,
    ) -> Self {
        Self {
            target,
            field_id,
            source: DetailsFieldSource::Unknown,
            value,
        }
    }

    pub fn new_with_source(
        target: DetailsTarget,
        field_id: DetailsFieldId,
        source: DetailsFieldSource,
        value: DetailsValue,
    ) -> Self {
        Self {
            target,
            field_id,
            source,
            value,
        }
    }

    pub fn get_target(&self) -> &DetailsTarget {
        &self.target
    }

    pub fn get_field_id(&self) -> &DetailsFieldId {
        &self.field_id
    }

    pub fn get_source(&self) -> &DetailsFieldSource {
        &self.source
    }

    pub fn get_value(&self) -> &DetailsValue {
        &self.value
    }
}
