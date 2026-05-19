use crate::structures::{
    data_types::{
        built_in_types::{string::utf8::data_type_string_utf8::DataTypeStringUtf8, u64::data_type_u64::DataTypeU64},
        data_type_ref::DataTypeRef,
    },
    data_values::container_type::ContainerType,
    details::{DetailsEdit, DetailsEditorHint, DetailsField, DetailsFieldId, DetailsFieldSource, DetailsProjection, DetailsTarget, DetailsValue},
    structs::symbolic_struct_definition::SymbolicLayoutKind,
};

/// Semantic element selector for a symbol-layout field.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SymbolLayoutDetailsFieldElementKind {
    #[default]
    BuiltInDataType,
    SymbolLayout,
}

impl SymbolLayoutDetailsFieldElementKind {
    pub const ALL: [Self; 2] = [Self::BuiltInDataType, Self::SymbolLayout];

    pub fn label(&self) -> &'static str {
        match self {
            Self::BuiltInDataType => "Data Type",
            Self::SymbolLayout => "Symbol Layout",
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            Self::BuiltInDataType => "data_type",
            Self::SymbolLayout => "symbol_layout",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        let trimmed_key = key.trim();

        Self::ALL
            .iter()
            .copied()
            .find(|element_kind| element_kind.key() == trimmed_key)
    }
}

/// Semantic container selector for a symbol-layout field.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SymbolLayoutDetailsFieldContainerKind {
    #[default]
    Element,
    Array,
    FixedArray,
    DynamicArray,
    Pointer,
    FixedPointerArray,
    DynamicPointerArray,
}

impl SymbolLayoutDetailsFieldContainerKind {
    pub const ALL: [Self; 7] = [
        Self::Element,
        Self::Array,
        Self::FixedArray,
        Self::DynamicArray,
        Self::Pointer,
        Self::FixedPointerArray,
        Self::DynamicPointerArray,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Element => "Element",
            Self::Array => "Array",
            Self::FixedArray => "Fixed Array",
            Self::DynamicArray => "Dynamic Array",
            Self::Pointer => "Pointer",
            Self::FixedPointerArray => "Fixed Pointer Array",
            Self::DynamicPointerArray => "Dynamic Pointer Array",
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            Self::Element => "element",
            Self::Array => "array",
            Self::FixedArray => "fixed_array",
            Self::DynamicArray => "dynamic_array",
            Self::Pointer => "pointer",
            Self::FixedPointerArray => "fixed_pointer_array",
            Self::DynamicPointerArray => "dynamic_pointer_array",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        let trimmed_key = key.trim();

        Self::ALL
            .iter()
            .copied()
            .find(|container_kind| container_kind.key() == trimmed_key)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SymbolLayoutFieldDetails {
    pub field_name: String,
    pub data_type_id: String,
    pub element_kind: SymbolLayoutDetailsFieldElementKind,
    pub container_kind: SymbolLayoutDetailsFieldContainerKind,
    pub fixed_array_length: Option<u64>,
    pub count_resolver_id: Option<String>,
    pub display_count_resolver_id: Option<String>,
    pub active_when_resolver_id: Option<String>,
    pub pointer_size_label: Option<String>,
    pub offset_resolver_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SymbolLayoutDetailsEditOperation {
    UpdateLayoutKind(SymbolicLayoutKind),
    UpdateFieldName(String),
    UpdateFieldElementKind(SymbolLayoutDetailsFieldElementKind),
    UpdateFieldDataType(String),
    UpdateFieldSymbolLayout(String),
    UpdateFieldContainerKind(SymbolLayoutDetailsFieldContainerKind),
    UpdateFieldFixedArrayLength(u64),
    UpdateFieldCountResolver(String),
    UpdateFieldDisplayCountResolver(String),
    UpdateFieldActiveWhenResolver(String),
    UpdateFieldPointerSize(String),
    UpdateFieldOffsetResolver(String),
    NoOp,
    Reject(String),
}

pub struct SymbolLayoutDetails;

impl SymbolLayoutDetails {
    pub const TARGET_KIND_LAYOUT: &'static str = "symbol_layout";
    pub const TARGET_KIND_FIELD: &'static str = "symbol_layout_field";
    pub const TARGET_KIND_UNASSIGNED: &'static str = "symbol_layout_unassigned";

    pub const FIELD_ID_LAYOUT_ID: &'static str = "layout.id";
    pub const FIELD_ID_LAYOUT_KIND: &'static str = "layout.kind";
    pub const FIELD_ID_FIELD_NAME: &'static str = "field.name";
    pub const FIELD_ID_FIELD_ELEMENT_KIND: &'static str = "field.element_type";
    pub const FIELD_ID_FIELD_DATA_TYPE: &'static str = "field.data_type";
    pub const FIELD_ID_FIELD_SYMBOL_LAYOUT: &'static str = "field.symbol_layout";
    pub const FIELD_ID_FIELD_CONTAINER_KIND: &'static str = "field.container_kind";
    pub const FIELD_ID_FIELD_FIXED_ARRAY_LENGTH: &'static str = "field.fixed_array_length";
    pub const FIELD_ID_FIELD_COUNT_RESOLVER: &'static str = "field.count_resolver";
    pub const FIELD_ID_FIELD_DISPLAY_COUNT_RESOLVER: &'static str = "field.display_count_resolver";
    pub const FIELD_ID_FIELD_ACTIVE_WHEN_RESOLVER: &'static str = "field.active_when_resolver";
    pub const FIELD_ID_FIELD_POINTER_SIZE: &'static str = "field.pointer_size";
    pub const FIELD_ID_FIELD_OFFSET_RESOLVER: &'static str = "field.offset_resolver";
    pub const FIELD_ID_UNASSIGNED_KIND: &'static str = "unassigned.kind";
    pub const FIELD_ID_UNASSIGNED_LAYOUT: &'static str = "unassigned.layout";
    pub const FIELD_ID_UNASSIGNED_OFFSET: &'static str = "unassigned.offset";
    pub const FIELD_ID_UNASSIGNED_SIZE: &'static str = "unassigned.size";

    pub fn build_layout_projection(
        layout_id: &str,
        layout_kind: SymbolicLayoutKind,
    ) -> DetailsProjection {
        DetailsProjection::new(
            DetailsTarget::new(Self::TARGET_KIND_LAYOUT, layout_id),
            layout_id,
            vec![
                Self::text_field(Self::FIELD_ID_LAYOUT_ID, "Name", layout_id, false),
                Self::layout_metadata_field(Self::FIELD_ID_LAYOUT_KIND, "Layout Kind", layout_kind.key(), false, DetailsEditorHint::Text),
            ],
        )
    }

    pub fn build_field_projection(
        layout_id: &str,
        field_index: usize,
        layout_kind: SymbolicLayoutKind,
        field_details: &SymbolLayoutFieldDetails,
    ) -> DetailsProjection {
        let mut fields = vec![Self::layout_metadata_field(
            Self::FIELD_ID_FIELD_NAME,
            "Name",
            &field_details.field_name,
            false,
            DetailsEditorHint::Text,
        )];

        if layout_kind.is_union() {
            fields.push(Self::layout_metadata_field(
                Self::FIELD_ID_FIELD_SYMBOL_LAYOUT,
                "Symbol Layout",
                &field_details.data_type_id,
                false,
                DetailsEditorHint::Text,
            ));
            fields.push(Self::layout_metadata_field(
                Self::FIELD_ID_FIELD_ACTIVE_WHEN_RESOLVER,
                "Active When Resolver",
                field_details
                    .active_when_resolver_id
                    .as_deref()
                    .unwrap_or_default(),
                false,
                DetailsEditorHint::Text,
            ));

            return DetailsProjection::new(
                DetailsTarget::new(Self::TARGET_KIND_FIELD, Self::field_target_id(layout_id, field_index)),
                field_details.field_name.clone(),
                fields,
            );
        }

        fields.push(Self::layout_metadata_field(
            Self::FIELD_ID_FIELD_ELEMENT_KIND,
            "Element Type",
            field_details.element_kind.key(),
            false,
            DetailsEditorHint::Text,
        ));
        fields.push(Self::layout_metadata_field(
            match field_details.element_kind {
                SymbolLayoutDetailsFieldElementKind::BuiltInDataType => Self::FIELD_ID_FIELD_DATA_TYPE,
                SymbolLayoutDetailsFieldElementKind::SymbolLayout => Self::FIELD_ID_FIELD_SYMBOL_LAYOUT,
            },
            field_details.element_kind.label(),
            &field_details.data_type_id,
            false,
            DetailsEditorHint::DataType,
        ));
        fields.push(Self::layout_metadata_field(
            Self::FIELD_ID_FIELD_CONTAINER_KIND,
            "Container",
            field_details.container_kind.key(),
            false,
            DetailsEditorHint::Text,
        ));

        if let Some(fixed_array_length) = field_details.fixed_array_length {
            fields.push(DetailsField::new(
                DetailsFieldId::new(Self::FIELD_ID_FIELD_FIXED_ARRAY_LENGTH),
                "Length",
                DetailsValue::UnsignedInteger(fixed_array_length.max(1)),
                false,
                DetailsEditorHint::Value,
                Some(DataTypeRef::new(DataTypeU64::DATA_TYPE_ID)),
                ContainerType::None,
                Self::layout_metadata_source(Self::FIELD_ID_FIELD_FIXED_ARRAY_LENGTH),
            ));
        }

        if let Some(count_resolver_id) = &field_details.count_resolver_id {
            fields.push(Self::layout_metadata_field(
                Self::FIELD_ID_FIELD_COUNT_RESOLVER,
                "Count Resolver",
                count_resolver_id,
                false,
                DetailsEditorHint::Text,
            ));
        }

        if let Some(display_count_resolver_id) = &field_details.display_count_resolver_id {
            fields.push(Self::layout_metadata_field(
                Self::FIELD_ID_FIELD_DISPLAY_COUNT_RESOLVER,
                "Display Count Resolver",
                display_count_resolver_id,
                false,
                DetailsEditorHint::Text,
            ));
        }

        if let Some(pointer_size_label) = &field_details.pointer_size_label {
            fields.push(Self::layout_metadata_field(
                Self::FIELD_ID_FIELD_POINTER_SIZE,
                "Pointer Size",
                pointer_size_label,
                false,
                DetailsEditorHint::PointerSize,
            ));
        }

        if let Some(offset_resolver_id) = &field_details.offset_resolver_id {
            fields.push(Self::layout_metadata_field(
                Self::FIELD_ID_FIELD_OFFSET_RESOLVER,
                "Offset Resolver",
                offset_resolver_id,
                false,
                DetailsEditorHint::Text,
            ));
        }

        DetailsProjection::new(
            DetailsTarget::new(Self::TARGET_KIND_FIELD, Self::field_target_id(layout_id, field_index)),
            field_details.field_name.clone(),
            fields,
        )
    }

    pub fn build_unassigned_projection(
        layout_id: &str,
        offset_in_bytes: u64,
        size_in_bytes: u64,
    ) -> DetailsProjection {
        DetailsProjection::new(
            DetailsTarget::new(Self::TARGET_KIND_UNASSIGNED, format!("{}:{}:{}", layout_id, offset_in_bytes, size_in_bytes)),
            "UNASSIGNED",
            vec![
                Self::readonly_text_field(Self::FIELD_ID_UNASSIGNED_KIND, "Kind", "UNASSIGNED"),
                Self::readonly_text_field(Self::FIELD_ID_UNASSIGNED_LAYOUT, "Layout", layout_id),
                DetailsField::new(
                    DetailsFieldId::new(Self::FIELD_ID_UNASSIGNED_OFFSET),
                    "Offset",
                    DetailsValue::UnsignedInteger(offset_in_bytes),
                    true,
                    DetailsEditorHint::Value,
                    Some(DataTypeRef::new(DataTypeU64::DATA_TYPE_ID)),
                    ContainerType::None,
                    DetailsFieldSource::Unknown,
                ),
                DetailsField::new(
                    DetailsFieldId::new(Self::FIELD_ID_UNASSIGNED_SIZE),
                    "Size",
                    DetailsValue::UnsignedInteger(size_in_bytes),
                    true,
                    DetailsEditorHint::Value,
                    Some(DataTypeRef::new(DataTypeU64::DATA_TYPE_ID)),
                    ContainerType::None,
                    DetailsFieldSource::Unknown,
                ),
            ],
        )
    }

    pub fn plan_edit(details_edit: &DetailsEdit) -> SymbolLayoutDetailsEditOperation {
        let field_id = details_edit.get_field_id().get_field_id();

        match field_id {
            Self::FIELD_ID_LAYOUT_KIND => Self::text_value(details_edit)
                .and_then(|text| SymbolicLayoutKind::from_key(&text))
                .map(SymbolLayoutDetailsEditOperation::UpdateLayoutKind)
                .unwrap_or_else(|| SymbolLayoutDetailsEditOperation::Reject(String::from("Unknown symbol layout kind."))),
            Self::FIELD_ID_FIELD_NAME => SymbolLayoutDetailsEditOperation::UpdateFieldName(Self::text_value(details_edit).unwrap_or_default()),
            Self::FIELD_ID_FIELD_ELEMENT_KIND => Self::text_value(details_edit)
                .and_then(|text| SymbolLayoutDetailsFieldElementKind::from_key(&text))
                .map(SymbolLayoutDetailsEditOperation::UpdateFieldElementKind)
                .unwrap_or_else(|| SymbolLayoutDetailsEditOperation::Reject(String::from("Unknown symbol layout field element type."))),
            Self::FIELD_ID_FIELD_DATA_TYPE => SymbolLayoutDetailsEditOperation::UpdateFieldDataType(Self::text_value(details_edit).unwrap_or_default()),
            Self::FIELD_ID_FIELD_SYMBOL_LAYOUT => SymbolLayoutDetailsEditOperation::UpdateFieldSymbolLayout(Self::text_value(details_edit).unwrap_or_default()),
            Self::FIELD_ID_FIELD_CONTAINER_KIND => Self::text_value(details_edit)
                .and_then(|text| SymbolLayoutDetailsFieldContainerKind::from_key(&text))
                .map(SymbolLayoutDetailsEditOperation::UpdateFieldContainerKind)
                .unwrap_or_else(|| SymbolLayoutDetailsEditOperation::Reject(String::from("Unknown symbol layout field container kind."))),
            Self::FIELD_ID_FIELD_FIXED_ARRAY_LENGTH => SymbolLayoutDetailsEditOperation::UpdateFieldFixedArrayLength(
                Self::u64_value(details_edit)
                    .unwrap_or_else(|| {
                        Self::text_value(details_edit)
                            .and_then(|text| text.trim().parse::<u64>().ok())
                            .unwrap_or(1)
                    })
                    .max(1),
            ),
            Self::FIELD_ID_FIELD_COUNT_RESOLVER => {
                SymbolLayoutDetailsEditOperation::UpdateFieldCountResolver(Self::text_value(details_edit).unwrap_or_default())
            }
            Self::FIELD_ID_FIELD_DISPLAY_COUNT_RESOLVER => {
                SymbolLayoutDetailsEditOperation::UpdateFieldDisplayCountResolver(Self::text_value(details_edit).unwrap_or_default())
            }
            Self::FIELD_ID_FIELD_ACTIVE_WHEN_RESOLVER => {
                SymbolLayoutDetailsEditOperation::UpdateFieldActiveWhenResolver(Self::text_value(details_edit).unwrap_or_default())
            }
            Self::FIELD_ID_FIELD_POINTER_SIZE => SymbolLayoutDetailsEditOperation::UpdateFieldPointerSize(Self::text_value(details_edit).unwrap_or_default()),
            Self::FIELD_ID_FIELD_OFFSET_RESOLVER => {
                SymbolLayoutDetailsEditOperation::UpdateFieldOffsetResolver(Self::text_value(details_edit).unwrap_or_default())
            }
            _ => SymbolLayoutDetailsEditOperation::NoOp,
        }
    }

    fn layout_metadata_field(
        field_id: &'static str,
        label: &'static str,
        value: &str,
        is_read_only: bool,
        editor_hint: DetailsEditorHint,
    ) -> DetailsField {
        DetailsField::new(
            DetailsFieldId::new(field_id),
            label,
            DetailsValue::Text(value.to_string()),
            is_read_only,
            editor_hint,
            Some(DataTypeRef::new(DataTypeStringUtf8::DATA_TYPE_ID)),
            ContainerType::None,
            Self::layout_metadata_source(field_id),
        )
    }

    fn text_field(
        field_id: &'static str,
        label: &'static str,
        value: &str,
        is_read_only: bool,
    ) -> DetailsField {
        DetailsField::new(
            DetailsFieldId::new(field_id),
            label,
            DetailsValue::Text(value.to_string()),
            is_read_only,
            DetailsEditorHint::Text,
            Some(DataTypeRef::new(DataTypeStringUtf8::DATA_TYPE_ID)),
            ContainerType::None,
            DetailsFieldSource::Unknown,
        )
    }

    fn readonly_text_field(
        field_id: &'static str,
        label: &'static str,
        value: &str,
    ) -> DetailsField {
        Self::text_field(field_id, label, value, true)
    }

    fn layout_metadata_source(field_id: &'static str) -> DetailsFieldSource {
        DetailsFieldSource::SymbolLayoutMetadata {
            metadata_name: field_id.to_string(),
        }
    }

    fn field_target_id(
        layout_id: &str,
        field_index: usize,
    ) -> String {
        format!("{}:{}", layout_id, field_index)
    }

    fn text_value(details_edit: &DetailsEdit) -> Option<String> {
        match details_edit.get_value() {
            DetailsValue::Text(text) => Some(text.clone()),
            DetailsValue::DataValue(data_value) => String::from_utf8(data_value.get_value_bytes().clone()).ok(),
            DetailsValue::AnonymousValue(anonymous_value_string) => Some(anonymous_value_string.get_anonymous_value_string().to_string()),
            DetailsValue::Boolean(value) => Some(value.to_string()),
            DetailsValue::UnsignedInteger(value) => Some(value.to_string()),
            DetailsValue::SignedInteger(value) => Some(value.to_string()),
            DetailsValue::Empty => Some(String::new()),
        }
    }

    fn u64_value(details_edit: &DetailsEdit) -> Option<u64> {
        match details_edit.get_value() {
            DetailsValue::UnsignedInteger(value) => Some(*value),
            DetailsValue::DataValue(data_value) => match data_value.get_value_bytes().len() {
                8 => <[u8; 8]>::try_from(data_value.get_value_bytes().as_slice())
                    .ok()
                    .map(u64::from_le_bytes),
                4 => <[u8; 4]>::try_from(data_value.get_value_bytes().as_slice())
                    .ok()
                    .map(|value_bytes| u32::from_le_bytes(value_bytes) as u64),
                _ => None,
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SymbolLayoutDetails, SymbolLayoutDetailsEditOperation, SymbolLayoutDetailsFieldContainerKind, SymbolLayoutDetailsFieldElementKind,
        SymbolLayoutFieldDetails,
    };
    use crate::structures::{
        data_types::built_in_types::string::utf8::data_type_string_utf8::DataTypeStringUtf8,
        details::{DetailsEdit, DetailsFieldId, DetailsTarget, DetailsValue},
        structs::symbolic_struct_definition::SymbolicLayoutKind,
    };

    #[test]
    fn field_projection_uses_stable_ids_for_symbol_layout_editor_metadata() {
        let projection = SymbolLayoutDetails::build_field_projection(
            "player.stats",
            0,
            SymbolicLayoutKind::Struct,
            &SymbolLayoutFieldDetails {
                field_name: "health".to_string(),
                data_type_id: "u32".to_string(),
                element_kind: SymbolLayoutDetailsFieldElementKind::BuiltInDataType,
                container_kind: SymbolLayoutDetailsFieldContainerKind::Element,
                fixed_array_length: None,
                count_resolver_id: None,
                display_count_resolver_id: None,
                active_when_resolver_id: None,
                pointer_size_label: None,
                offset_resolver_id: None,
            },
        );

        assert!(
            projection
                .get_field(&DetailsFieldId::new(SymbolLayoutDetails::FIELD_ID_FIELD_NAME))
                .is_some()
        );
        assert!(
            projection
                .get_field(&DetailsFieldId::new(SymbolLayoutDetails::FIELD_ID_FIELD_DATA_TYPE))
                .is_some()
        );
    }

    #[test]
    fn edit_planner_routes_field_name_without_parsing_display_label() {
        let details_edit = DetailsEdit::new(
            DetailsTarget::new(SymbolLayoutDetails::TARGET_KIND_FIELD, "player.stats:0"),
            DetailsFieldId::new(SymbolLayoutDetails::FIELD_ID_FIELD_NAME),
            DetailsValue::DataValue(DataTypeStringUtf8::get_value_from_primitive_string("new_health")),
        );

        assert_eq!(
            SymbolLayoutDetails::plan_edit(&details_edit),
            SymbolLayoutDetailsEditOperation::UpdateFieldName("new_health".to_string())
        );
    }

    #[test]
    fn edit_planner_routes_layout_kind() {
        let details_edit = DetailsEdit::new(
            DetailsTarget::new(SymbolLayoutDetails::TARGET_KIND_LAYOUT, "player.stats"),
            DetailsFieldId::new(SymbolLayoutDetails::FIELD_ID_LAYOUT_KIND),
            DetailsValue::Text("union".to_string()),
        );

        assert_eq!(
            SymbolLayoutDetails::plan_edit(&details_edit),
            SymbolLayoutDetailsEditOperation::UpdateLayoutKind(SymbolicLayoutKind::Union)
        );
    }
}
