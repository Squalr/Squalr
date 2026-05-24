use crate::structures::{
    data_types::{
        built_in_types::{bool8::data_type_bool8::DataTypeBool8, string::utf8::data_type_string_utf8::DataTypeStringUtf8, u64::data_type_u64::DataTypeU64},
        data_type_ref::DataTypeRef,
    },
    data_values::anonymous_value_string_format::AnonymousValueStringFormat,
    data_values::container_type::ContainerType,
    details::{DetailsEditorHint, DetailsField, DetailsFieldId, DetailsFieldSource, DetailsProjection, DetailsTarget, DetailsValue},
    projects::symbol_tree::symbol_tree_node::{SymbolTreeNode, SymbolTreeNodeKind},
    structs::{valued_struct::ValuedStruct, valued_struct_field::ValuedStructFieldData},
};

pub struct SymbolTreeDetailsProjection;

impl SymbolTreeDetailsProjection {
    pub const TARGET_KIND_SYMBOL_TREE: &'static str = "symbol_tree";
    pub const FIELD_ID_METADATA_PREFIX: &'static str = "metadata.";
    pub const FIELD_ID_VALUE_PREFIX: &'static str = "value.";
    pub const METADATA_DISPLAY_NAME: &'static str = "display_name";
    pub const METADATA_TYPE: &'static str = "type";
    pub const METADATA_ADDRESS: &'static str = "address";
    pub const METADATA_MODULE: &'static str = "module";
    pub const METADATA_SIZE: &'static str = "size";
    pub const METADATA_PATH: &'static str = "path";
    pub const METADATA_LOCATOR: &'static str = "locator";
    pub const METADATA_STATUS: &'static str = "status";
    pub const METADATA_STRING_BUFFER_SIZE: &'static str = "string_buffer_size";
    pub const METADATA_NULL_TERMINATED: &'static str = "null_terminated";

    pub fn build(
        symbol_tree_node: &SymbolTreeNode,
        include_symbol_claim_metadata: bool,
        symbol_size_in_bytes: Option<u64>,
        runtime_value_struct: Option<&ValuedStruct>,
        status_text: Option<&str>,
        preferred_display_format: Option<AnonymousValueStringFormat>,
    ) -> DetailsProjection {
        Self::build_with_metadata_type_id(
            symbol_tree_node,
            include_symbol_claim_metadata,
            symbol_size_in_bytes,
            runtime_value_struct,
            status_text,
            None,
            preferred_display_format,
        )
    }

    pub fn build_with_metadata_type_id(
        symbol_tree_node: &SymbolTreeNode,
        include_symbol_claim_metadata: bool,
        symbol_size_in_bytes: Option<u64>,
        runtime_value_struct: Option<&ValuedStruct>,
        status_text: Option<&str>,
        metadata_type_id: Option<&str>,
        preferred_display_format: Option<AnonymousValueStringFormat>,
    ) -> DetailsProjection {
        let target = DetailsTarget::new(Self::TARGET_KIND_SYMBOL_TREE, symbol_tree_node.get_node_key());
        let mut fields = Self::build_metadata_fields_with_type_id(symbol_tree_node, include_symbol_claim_metadata, symbol_size_in_bytes, metadata_type_id);

        if let Some(runtime_value_struct) = runtime_value_struct
            && Self::should_include_runtime_value_fields(symbol_tree_node)
        {
            fields.extend(Self::build_runtime_value_fields(runtime_value_struct, preferred_display_format));
        }

        if let Some(status_text) = status_text {
            fields.push(Self::build_text_metadata_field(
                Self::METADATA_LOCATOR,
                "Locator",
                &symbol_tree_node.get_locator().to_string(),
                true,
            ));
            fields.push(Self::build_text_metadata_field(Self::METADATA_STATUS, "Status", status_text, true));
        }

        DetailsProjection::new(target, symbol_tree_node.get_display_name(), fields)
    }

    pub fn build_metadata_fields(
        symbol_tree_node: &SymbolTreeNode,
        include_symbol_claim_metadata: bool,
        symbol_size_in_bytes: Option<u64>,
    ) -> Vec<DetailsField> {
        Self::build_metadata_fields_with_type_id(symbol_tree_node, include_symbol_claim_metadata, symbol_size_in_bytes, None)
    }

    pub fn build_external_value(
        symbol_tree_node: &SymbolTreeNode,
        include_symbol_claim_metadata: bool,
        symbol_size_in_bytes: Option<u64>,
        preferred_display_format: Option<AnonymousValueStringFormat>,
    ) -> DetailsProjection {
        let target = DetailsTarget::new(Self::TARGET_KIND_SYMBOL_TREE, symbol_tree_node.get_node_key());
        let mut fields = Self::build_metadata_fields(symbol_tree_node, include_symbol_claim_metadata, symbol_size_in_bytes);

        fields.push(
            DetailsField::new(
                DetailsFieldId::new(format!("{}value", Self::FIELD_ID_VALUE_PREFIX)),
                "Value",
                DetailsValue::Text(String::new()),
                true,
                DetailsEditorHint::Value,
                Some(DataTypeRef::new(&symbol_tree_node.get_display_type_id())),
                symbol_tree_node.get_container_type(),
                DetailsFieldSource::ProjectSymbolRuntimeValue {
                    field_path: vec![String::from("value")],
                },
            )
            .with_preferred_display_format(preferred_display_format)
            .with_allow_display_format_edit(false),
        );

        DetailsProjection::new(target, symbol_tree_node.get_display_name(), fields)
    }

    fn build_metadata_fields_with_type_id(
        symbol_tree_node: &SymbolTreeNode,
        include_symbol_claim_metadata: bool,
        symbol_size_in_bytes: Option<u64>,
        metadata_type_id: Option<&str>,
    ) -> Vec<DetailsField> {
        let mut metadata_fields = Vec::new();
        let metadata_data_type_id = metadata_type_id
            .map(str::to_string)
            .unwrap_or_else(|| symbol_tree_node.get_display_type_id());
        let metadata_data_type_ref = DataTypeRef::new(&metadata_data_type_id);

        if include_symbol_claim_metadata {
            metadata_fields.push(Self::build_text_metadata_field(
                Self::METADATA_DISPLAY_NAME,
                "Display Name",
                symbol_tree_node.get_display_name(),
                true,
            ));
        }

        metadata_fields.push(DetailsField::new(
            DetailsFieldId::new(format!("{}{}", Self::FIELD_ID_METADATA_PREFIX, Self::METADATA_TYPE)),
            "Data Type",
            DetailsValue::Text(metadata_data_type_ref.get_base_data_type_id().to_string()),
            true,
            DetailsEditorHint::DataType,
            Some(DataTypeRef::new(DataTypeStringUtf8::DATA_TYPE_ID)),
            ContainerType::None,
            DetailsFieldSource::SymbolLayoutMetadata {
                metadata_name: Self::METADATA_TYPE.to_string(),
            },
        ));
        metadata_fields.extend(Self::build_string_metadata_fields(
            &metadata_data_type_ref,
            symbol_tree_node.get_container_type(),
        ));
        metadata_fields.extend(Self::build_location_fields(symbol_tree_node, symbol_size_in_bytes));

        metadata_fields
    }

    fn build_string_metadata_fields(
        data_type_ref: &DataTypeRef,
        container_type: ContainerType,
    ) -> Vec<DetailsField> {
        if data_type_ref.get_base_data_type_id() != DataTypeStringUtf8::DATA_TYPE_ID {
            return Vec::new();
        }

        let mut metadata_fields = Vec::new();

        if let ContainerType::ArrayFixed(string_buffer_size) = container_type {
            metadata_fields.push(DetailsField::new(
                DetailsFieldId::new(format!("{}{}", Self::FIELD_ID_METADATA_PREFIX, Self::METADATA_STRING_BUFFER_SIZE)),
                "String Size",
                DetailsValue::UnsignedInteger(string_buffer_size),
                true,
                DetailsEditorHint::Value,
                Some(DataTypeRef::new(DataTypeU64::DATA_TYPE_ID)),
                ContainerType::None,
                DetailsFieldSource::SymbolLayoutMetadata {
                    metadata_name: Self::METADATA_STRING_BUFFER_SIZE.to_string(),
                },
            ));
        }

        metadata_fields.push(DetailsField::new(
            DetailsFieldId::new(format!("{}{}", Self::FIELD_ID_METADATA_PREFIX, Self::METADATA_NULL_TERMINATED)),
            "Null Terminated",
            DetailsValue::Boolean(data_type_ref.has_flag(DataTypeStringUtf8::FLAG_NULL_TERMINATED)),
            true,
            DetailsEditorHint::Boolean,
            Some(DataTypeRef::new(DataTypeBool8::DATA_TYPE_ID)),
            ContainerType::None,
            DetailsFieldSource::SymbolLayoutMetadata {
                metadata_name: Self::METADATA_NULL_TERMINATED.to_string(),
            },
        ));

        metadata_fields
    }

    fn build_location_fields(
        symbol_tree_node: &SymbolTreeNode,
        symbol_size_in_bytes: Option<u64>,
    ) -> Vec<DetailsField> {
        let mut location_fields = Vec::new();
        let locator = symbol_tree_node.get_locator();

        location_fields.push(DetailsField::new(
            DetailsFieldId::new(format!("{}{}", Self::FIELD_ID_METADATA_PREFIX, Self::METADATA_ADDRESS)),
            "Address",
            DetailsValue::UnsignedInteger(locator.get_focus_address()),
            true,
            DetailsEditorHint::Address,
            Some(DataTypeRef::new(DataTypeU64::DATA_TYPE_ID)),
            ContainerType::None,
            DetailsFieldSource::SymbolLayoutMetadata {
                metadata_name: Self::METADATA_ADDRESS.to_string(),
            },
        ));
        location_fields.push(Self::build_text_metadata_field(
            Self::METADATA_MODULE,
            "Module",
            locator.get_focus_module_name(),
            true,
        ));

        if let Some(symbol_size_in_bytes) = symbol_size_in_bytes {
            location_fields.push(DetailsField::new(
                DetailsFieldId::new(format!("{}{}", Self::FIELD_ID_METADATA_PREFIX, Self::METADATA_SIZE)),
                "Size",
                DetailsValue::UnsignedInteger(symbol_size_in_bytes),
                true,
                DetailsEditorHint::Value,
                Some(DataTypeRef::new(DataTypeU64::DATA_TYPE_ID)),
                ContainerType::None,
                DetailsFieldSource::SymbolLayoutMetadata {
                    metadata_name: Self::METADATA_SIZE.to_string(),
                },
            ));
        }

        if !symbol_tree_node.get_full_path().is_empty() {
            location_fields.push(Self::build_text_metadata_field(
                Self::METADATA_PATH,
                "Path",
                symbol_tree_node.get_full_path(),
                true,
            ));
        }

        location_fields
    }

    fn build_runtime_value_fields(
        runtime_value_struct: &ValuedStruct,
        preferred_display_format: Option<AnonymousValueStringFormat>,
    ) -> Vec<DetailsField> {
        runtime_value_struct
            .get_fields()
            .iter()
            .enumerate()
            .filter_map(|(field_index, valued_struct_field)| {
                let field_name = Self::normalize_symbol_value_field_name(valued_struct_field.get_name(), field_index);
                let ValuedStructFieldData::Value(data_value) = valued_struct_field.get_field_data() else {
                    return None;
                };

                Some(
                    DetailsField::new(
                        DetailsFieldId::new(format!("{}{}", Self::FIELD_ID_VALUE_PREFIX, field_name)),
                        field_name.clone(),
                        DetailsValue::DataValue(data_value.clone()),
                        false,
                        DetailsEditorHint::Value,
                        Some(data_value.get_data_type_ref().clone()),
                        ContainerType::None,
                        DetailsFieldSource::ProjectSymbolRuntimeValue { field_path: vec![field_name] },
                    )
                    .with_preferred_display_format(preferred_display_format)
                    .with_allow_display_format_edit(false),
                )
            })
            .collect()
    }

    pub fn should_include_runtime_value_fields(symbol_tree_node: &SymbolTreeNode) -> bool {
        if matches!(symbol_tree_node.get_kind(), SymbolTreeNodeKind::UnassignedSegment { .. }) {
            return false;
        }

        !(symbol_tree_node.can_expand() && symbol_tree_node.get_container_type() == ContainerType::None)
    }

    fn build_text_metadata_field(
        metadata_name: &str,
        label: &str,
        value: &str,
        is_read_only: bool,
    ) -> DetailsField {
        DetailsField::new(
            DetailsFieldId::new(format!("{}{}", Self::FIELD_ID_METADATA_PREFIX, metadata_name)),
            label,
            DetailsValue::Text(value.to_string()),
            is_read_only,
            DetailsEditorHint::Text,
            Some(DataTypeRef::new(DataTypeStringUtf8::DATA_TYPE_ID)),
            ContainerType::None,
            DetailsFieldSource::SymbolLayoutMetadata {
                metadata_name: metadata_name.to_string(),
            },
        )
    }

    fn normalize_symbol_value_field_name(
        field_name: &str,
        field_index: usize,
    ) -> String {
        if field_name.trim().is_empty() {
            if field_index == 0 {
                String::from("value")
            } else {
                format!("value_{}", field_index)
            }
        } else {
            field_name.to_string()
        }
    }

    pub fn include_symbol_claim_metadata(symbol_tree_node: &SymbolTreeNode) -> bool {
        matches!(symbol_tree_node.get_kind(), SymbolTreeNodeKind::SymbolClaim { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::SymbolTreeDetailsProjection;
    use crate::structures::{
        data_types::built_in_types::u32::data_type_u32::DataTypeU32,
        data_values::{anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
        details::{DetailsEditorHint, DetailsFieldId, DetailsFieldSource, DetailsValue},
        projects::{
            project_symbol_locator::ProjectSymbolLocator,
            symbol_tree::symbol_tree_node::{SymbolTreeNode, SymbolTreeNodeKind},
        },
        structs::{
            valued_struct::ValuedStruct,
            valued_struct_field::{ValuedStructField, ValuedStructFieldData},
        },
    };

    fn create_symbol_claim_node() -> SymbolTreeNode {
        SymbolTreeNode::new(
            String::from("claim:absolute:1234"),
            SymbolTreeNodeKind::SymbolClaim {
                symbol_locator_key: String::from("absolute:1234"),
            },
            0,
            String::from("Health"),
            String::from("Player.Health"),
            String::from("absolute:1234"),
            ProjectSymbolLocator::new_absolute_address(0x1234),
            String::from("u32"),
            Default::default(),
            false,
        )
    }

    fn create_unassigned_segment_node() -> SymbolTreeNode {
        SymbolTreeNode::new(
            String::from("unassigned:game.exe:0:20"),
            SymbolTreeNodeKind::UnassignedSegment {
                module_name: String::from("game.exe"),
                offset: 0,
                length: 0x20,
            },
            1,
            String::from("UNASSIGNED_00000000"),
            String::from("game.exe.UNASSIGNED_00000000"),
            String::new(),
            ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0),
            String::from("UNASSIGNED"),
            ContainerType::ArrayFixed(0x20),
            false,
        )
    }

    fn create_struct_symbol_claim_node() -> SymbolTreeNode {
        SymbolTreeNode::new(
            String::from("claim:absolute:1234"),
            SymbolTreeNodeKind::SymbolClaim {
                symbol_locator_key: String::from("absolute:1234"),
            },
            0,
            String::from("Player"),
            String::from("Player"),
            String::from("absolute:1234"),
            ProjectSymbolLocator::new_absolute_address(0x1234),
            String::from("player"),
            Default::default(),
            true,
        )
    }

    #[test]
    fn build_includes_symbol_claim_metadata_and_location_fields() {
        let symbol_tree_node = create_symbol_claim_node();
        let details_projection = SymbolTreeDetailsProjection::build(&symbol_tree_node, true, Some(4), None, None, None);

        assert_eq!(
            details_projection.get_target().get_target_kind(),
            SymbolTreeDetailsProjection::TARGET_KIND_SYMBOL_TREE
        );
        assert_eq!(
            details_projection
                .get_field(&DetailsFieldId::new("metadata.display_name"))
                .expect("Expected display name field.")
                .get_source(),
            &DetailsFieldSource::SymbolLayoutMetadata {
                metadata_name: String::from("display_name")
            }
        );
        assert!(
            details_projection
                .get_field(&DetailsFieldId::new("metadata.display_name"))
                .expect("Expected display name field.")
                .get_is_read_only()
        );
        assert_eq!(
            details_projection
                .get_field(&DetailsFieldId::new("metadata.address"))
                .expect("Expected address field.")
                .get_value(),
            &DetailsValue::UnsignedInteger(0x1234)
        );
        assert_eq!(
            details_projection
                .get_field(&DetailsFieldId::new("metadata.size"))
                .expect("Expected size field.")
                .get_value(),
            &DetailsValue::UnsignedInteger(4)
        );
    }

    #[test]
    fn build_uses_metadata_type_override_for_module_roots() {
        let symbol_tree_node = SymbolTreeNode::new(
            String::from("module:winmine.exe"),
            SymbolTreeNodeKind::ModuleSpace {
                module_name: String::from("winmine.exe"),
                size: 0x2000,
            },
            0,
            String::from("winmine.exe"),
            String::from("winmine.exe"),
            String::from("winmine.exe"),
            ProjectSymbolLocator::new_module_offset(String::from("winmine.exe"), 0),
            String::from("u8"),
            ContainerType::ArrayFixed(0x2000),
            false,
        );
        let details_projection =
            SymbolTreeDetailsProjection::build_with_metadata_type_id(&symbol_tree_node, false, Some(0x2000), None, None, Some("winmine.exe"), None);

        assert_eq!(
            details_projection
                .get_field(&DetailsFieldId::new("metadata.type"))
                .expect("Expected data type field.")
                .get_value(),
            &DetailsValue::Text(String::from("winmine.exe"))
        );
    }

    #[test]
    fn build_external_value_uses_details_runtime_source_for_arrays() {
        let symbol_tree_node = SymbolTreeNode::new(
            String::from("claim:absolute:1234"),
            SymbolTreeNodeKind::SymbolClaim {
                symbol_locator_key: String::from("absolute:1234"),
            },
            0,
            String::from("Buffer"),
            String::from("Buffer"),
            String::from("absolute:1234"),
            ProjectSymbolLocator::new_absolute_address(0x1234),
            String::from("u8"),
            ContainerType::ArrayFixed(16),
            false,
        );
        let details_projection =
            SymbolTreeDetailsProjection::build_external_value(&symbol_tree_node, true, Some(16), Some(AnonymousValueStringFormat::Hexadecimal));
        let value_field = details_projection
            .get_field(&DetailsFieldId::new("value.value"))
            .expect("Expected external value field.");

        assert_eq!(value_field.get_label(), "Value");
        assert!(value_field.get_is_read_only());
        assert_eq!(value_field.get_editor_hint(), &DetailsEditorHint::Value);
        assert_eq!(value_field.get_container_type(), ContainerType::ArrayFixed(16));
        assert_eq!(
            value_field.get_source(),
            &DetailsFieldSource::ProjectSymbolRuntimeValue {
                field_path: vec![String::from("value")]
            }
        );
        assert_eq!(value_field.get_preferred_display_format(), Some(AnonymousValueStringFormat::Hexadecimal));
        assert!(!value_field.get_allow_display_format_edit());
    }

    #[test]
    fn build_normalizes_empty_runtime_value_names() {
        let symbol_tree_node = create_symbol_claim_node();
        let runtime_value_struct = ValuedStruct::new_anonymous(vec![
            DataTypeU32::get_value_from_primitive(100).to_named_valued_struct_field(String::new(), false),
        ]);
        let details_projection = SymbolTreeDetailsProjection::build(&symbol_tree_node, false, None, Some(&runtime_value_struct), None, None);
        let value_field = details_projection
            .get_field(&DetailsFieldId::new("value.value"))
            .expect("Expected normalized value field.");

        assert_eq!(value_field.get_label(), "value");
        assert_eq!(value_field.get_editor_hint(), &DetailsEditorHint::Value);
        assert_eq!(
            value_field.get_source(),
            &DetailsFieldSource::ProjectSymbolRuntimeValue {
                field_path: vec![String::from("value")]
            }
        );
    }

    #[test]
    fn build_marks_symbol_runtime_display_format_read_only() {
        let symbol_tree_node = create_symbol_claim_node();
        let runtime_value_struct = ValuedStruct::new_anonymous(vec![
            DataTypeU32::get_value_from_primitive(100).to_named_valued_struct_field(String::from("health"), false),
        ]);
        let details_projection = SymbolTreeDetailsProjection::build(
            &symbol_tree_node,
            false,
            None,
            Some(&runtime_value_struct),
            None,
            Some(AnonymousValueStringFormat::Hexadecimal),
        );
        let value_field = details_projection
            .get_field(&DetailsFieldId::new("value.health"))
            .expect("Expected runtime value field.");

        assert_eq!(value_field.get_preferred_display_format(), Some(AnonymousValueStringFormat::Hexadecimal));
        assert!(!value_field.get_allow_display_format_edit());
    }

    #[test]
    fn build_omits_runtime_value_fields_for_unassigned_segments() {
        let symbol_tree_node = create_unassigned_segment_node();
        let runtime_value_struct = ValuedStruct::new_anonymous(vec![
            DataTypeU32::get_value_from_primitive(100).to_named_valued_struct_field(String::from("value"), false),
        ]);
        let details_projection = SymbolTreeDetailsProjection::build(&symbol_tree_node, false, Some(0x20), Some(&runtime_value_struct), None, None);

        assert!(
            details_projection
                .get_field(&DetailsFieldId::new("value.value"))
                .is_none()
        );
        assert!(
            details_projection
                .get_field(&DetailsFieldId::new("metadata.size"))
                .is_some()
        );
    }

    #[test]
    fn build_omits_runtime_value_fields_for_expandable_structs() {
        let symbol_tree_node = create_struct_symbol_claim_node();
        let runtime_value_struct = ValuedStruct::new_anonymous(vec![
            DataTypeU32::get_value_from_primitive(100).to_named_valued_struct_field(String::from("health"), false),
            DataTypeU32::get_value_from_primitive(200).to_named_valued_struct_field(String::from("mana"), false),
        ]);
        let details_projection = SymbolTreeDetailsProjection::build(&symbol_tree_node, true, Some(8), Some(&runtime_value_struct), None, None);

        assert!(
            details_projection
                .get_field(&DetailsFieldId::new("value.health"))
                .is_none()
        );
        assert!(
            details_projection
                .get_field(&DetailsFieldId::new("value.mana"))
                .is_none()
        );
        assert!(
            details_projection
                .get_field(&DetailsFieldId::new("metadata.size"))
                .is_some()
        );
    }

    #[test]
    fn build_adds_status_fields_for_fallback_projection() {
        let symbol_tree_node = create_symbol_claim_node();
        let details_projection = SymbolTreeDetailsProjection::build(&symbol_tree_node, true, None, None, Some("Unable to read symbol."), None);

        assert_eq!(
            details_projection
                .get_field(&DetailsFieldId::new("metadata.status"))
                .expect("Expected status field.")
                .get_value(),
            &DetailsValue::Text(String::from("Unable to read symbol."))
        );
    }

    #[test]
    fn build_omits_nested_runtime_structs_from_value_fields() {
        let symbol_tree_node = create_symbol_claim_node();
        let nested_runtime_struct = ValuedStruct::new_anonymous(vec![
            DataTypeU32::get_value_from_primitive(100).to_named_valued_struct_field(String::from("nested_value"), false),
        ]);
        let runtime_value_struct = ValuedStruct::new_anonymous(vec![
            ValuedStructField::new(
                String::from("nested"),
                ValuedStructFieldData::NestedStruct(Box::new(nested_runtime_struct)),
                false,
            ),
            DataTypeU32::get_value_from_primitive(200).to_named_valued_struct_field(String::from("leaf"), false),
        ]);
        let details_projection = SymbolTreeDetailsProjection::build(&symbol_tree_node, false, None, Some(&runtime_value_struct), None, None);

        assert!(
            details_projection
                .get_field(&DetailsFieldId::new("value.nested"))
                .is_none()
        );
        assert!(
            details_projection
                .get_field(&DetailsFieldId::new("value.leaf"))
                .is_some()
        );
    }
}
