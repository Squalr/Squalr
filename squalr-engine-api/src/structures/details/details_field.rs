use crate::structures::{
    data_types::data_type_ref::DataTypeRef,
    data_values::{
        anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType,
        data_value::DataValue,
    },
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
    DisplayFormat,
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
    DisplayFormat(AnonymousValueStringFormat),
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    preferred_display_format: Option<AnonymousValueStringFormat>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    allowed_display_formats: Vec<AnonymousValueStringFormat>,
    #[serde(default = "default_allow_display_format_edit")]
    allow_display_format_edit: bool,
}

fn default_allow_display_format_edit() -> bool {
    true
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
            preferred_display_format: None,
            allowed_display_formats: Vec::new(),
            allow_display_format_edit: true,
        }
    }

    pub fn with_preferred_display_format(
        mut self,
        preferred_display_format: Option<AnonymousValueStringFormat>,
    ) -> Self {
        self.preferred_display_format = preferred_display_format;

        self
    }

    pub fn with_allowed_display_formats(
        mut self,
        allowed_display_formats: Vec<AnonymousValueStringFormat>,
    ) -> Self {
        self.allowed_display_formats = allowed_display_formats;

        self
    }

    pub fn with_allow_display_format_edit(
        mut self,
        allow_display_format_edit: bool,
    ) -> Self {
        self.allow_display_format_edit = allow_display_format_edit;

        self
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

    pub fn get_preferred_display_format(&self) -> Option<AnonymousValueStringFormat> {
        self.preferred_display_format
    }

    pub fn get_allowed_display_formats(&self) -> &[AnonymousValueStringFormat] {
        &self.allowed_display_formats
    }

    pub fn get_allow_display_format_edit(&self) -> bool {
        self.allow_display_format_edit
    }
}
