use crate::structures::{
    data_types::data_type_ref::DataTypeRef,
    data_values::{anonymous_value_string::AnonymousValueString, container_type::ContainerType, data_value::DataValue},
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Stable identifier for a field inside a details projection.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DetailsFieldId(String);

impl DetailsFieldId {
    pub fn new(field_id: impl Into<String>) -> Self {
        Self(field_id.into())
    }

    pub fn get_field_id(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DetailsFieldId {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// Describes the semantic editor a UI should choose for a field.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum DetailsEditorHint {
    #[default]
    Value,
    Address,
    DataType,
    PointerOffsets,
    PointerSize,
    Text,
    Boolean,
}

/// Identifies where a detail field came from so edits can be planned without parsing labels.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum DetailsFieldSource {
    #[default]
    Unknown,
    ProjectItemProperty {
        property_name: String,
    },
    ProjectItemRuntimeValue {
        field_path: Vec<String>,
    },
    ProjectItemAddressTarget {
        property_name: String,
    },
    ProjectSymbolRuntimeValue {
        field_path: Vec<String>,
    },
    SymbolLayoutMetadata {
        metadata_name: String,
    },
    SymbolResolverMetadata {
        metadata_name: String,
    },
}

/// Serializable value carried by a details field or details edit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DetailsValue {
    Empty,
    AnonymousValue(AnonymousValueString),
    DataValue(DataValue),
    Text(String),
    Boolean(bool),
    UnsignedInteger(u64),
    SignedInteger(i64),
}

impl Default for DetailsValue {
    fn default() -> Self {
        Self::Empty
    }
}

/// One projected field for an inspectable target.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DetailsField {
    id: DetailsFieldId,
    label: String,
    value: DetailsValue,
    is_read_only: bool,
    editor_hint: DetailsEditorHint,
    validation_data_type_ref: Option<DataTypeRef>,
    container_type: ContainerType,
    source: DetailsFieldSource,
}

impl DetailsField {
    pub fn new(
        id: DetailsFieldId,
        label: impl Into<String>,
        value: DetailsValue,
        is_read_only: bool,
        editor_hint: DetailsEditorHint,
        validation_data_type_ref: Option<DataTypeRef>,
        container_type: ContainerType,
        source: DetailsFieldSource,
    ) -> Self {
        Self {
            id,
            label: label.into(),
            value,
            is_read_only,
            editor_hint,
            validation_data_type_ref,
            container_type,
            source,
        }
    }

    pub fn get_id(&self) -> &DetailsFieldId {
        &self.id
    }

    pub fn get_label(&self) -> &str {
        &self.label
    }

    pub fn get_value(&self) -> &DetailsValue {
        &self.value
    }

    pub fn get_is_read_only(&self) -> bool {
        self.is_read_only
    }

    pub fn get_editor_hint(&self) -> &DetailsEditorHint {
        &self.editor_hint
    }

    pub fn get_validation_data_type_ref(&self) -> Option<&DataTypeRef> {
        self.validation_data_type_ref.as_ref()
    }

    pub fn get_container_type(&self) -> ContainerType {
        self.container_type
    }

    pub fn get_source(&self) -> &DetailsFieldSource {
        &self.source
    }
}
