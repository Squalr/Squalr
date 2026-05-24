use crate::structures::details::{DetailsField, DetailsFieldId, DetailsTarget};
use serde::{Deserialize, Serialize};

/// Current reflected details for one inspectable target.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DetailsProjection {
    target: DetailsTarget,
    title: String,
    fields: Vec<DetailsField>,
}

impl DetailsProjection {
    pub fn new(
        target: DetailsTarget,
        title: impl Into<String>,
        fields: Vec<DetailsField>,
    ) -> Self {
        Self {
            target,
            title: title.into(),
            fields,
        }
    }

    pub fn get_target(&self) -> &DetailsTarget {
        &self.target
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn get_fields(&self) -> &[DetailsField] {
        &self.fields
    }

    pub fn get_field(
        &self,
        field_id: &DetailsFieldId,
    ) -> Option<&DetailsField> {
        self.fields
            .iter()
            .find(|details_field| details_field.get_id() == field_id)
    }
}
