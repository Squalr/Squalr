use crate::structures::{
    data_types::{
        built_in_types::string::utf8::data_type_string_utf8::DataTypeStringUtf8, built_in_types::u64::data_type_u64::DataTypeU64, data_type_ref::DataTypeRef,
    },
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

    pub fn build(
        symbol_tree_node: &SymbolTreeNode,
        include_symbol_claim_metadata: bool,
        symbol_size_in_bytes: Option<u64>,
        runtime_value_struct: Option<&ValuedStruct>,
        status_text: Option<&str>,
    ) -> DetailsProjection {
        let target = DetailsTarget::new(Self::TARGET_KIND_SYMBOL_TREE, symbol_tree_node.get_node_key());
        let mut fields = Self::build_metadata_fields(symbol_tree_node, include_symbol_claim_metadata, symbol_size_in_bytes);

        if let Some(runtime_value_struct) = runtime_value_struct {
            fields.extend(Self::build_runtime_value_fields(runtime_value_struct));
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
        let mut metadata_fields = Vec::new();

        if include_symbol_claim_metadata {
            metadata_fields.push(Self::build_text_metadata_field(
                Self::METADATA_DISPLAY_NAME,
                "Display Name",
                symbol_tree_node.get_display_name(),
                false,
            ));
        }

        metadata_fields.push(DetailsField::new(
            DetailsFieldId::new(format!("{}{}", Self::FIELD_ID_METADATA_PREFIX, Self::METADATA_TYPE)),
            "Data Type",
            DetailsValue::Text(symbol_tree_node.get_display_type_id()),
            true,
            DetailsEditorHint::DataType,
            Some(DataTypeRef::new(DataTypeStringUtf8::DATA_TYPE_ID)),
            ContainerType::None,
            DetailsFieldSource::SymbolLayoutMetadata {
                metadata_name: Self::METADATA_TYPE.to_string(),
            },
        ));
        metadata_fields.extend(Self::build_location_fields(symbol_tree_node, symbol_size_in_bytes));

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

    fn build_runtime_value_fields(runtime_value_struct: &ValuedStruct) -> Vec<DetailsField> {
        runtime_value_struct
            .get_fields()
            .iter()
            .enumerate()
            .filter_map(|(field_index, valued_struct_field)| {
                let field_name = Self::normalize_symbol_value_field_name(valued_struct_field.get_name(), field_index);
                let ValuedStructFieldData::Value(data_value) = valued_struct_field.get_field_data() else {
                    return None;
                };

                Some(DetailsField::new(
                    DetailsFieldId::new(format!("{}{}", Self::FIELD_ID_VALUE_PREFIX, field_name)),
                    field_name.clone(),
                    DetailsValue::DataValue(data_value.clone()),
                    false,
                    DetailsEditorHint::Value,
                    Some(data_value.get_data_type_ref().clone()),
                    ContainerType::None,
                    DetailsFieldSource::ProjectSymbolRuntimeValue { field_path: vec![field_name] },
                ))
            })
            .collect()
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
            false,
        )
    }

    #[test]
    fn build_includes_symbol_claim_metadata_and_location_fields() {
        let symbol_tree_node = create_symbol_claim_node();
        let details_projection = SymbolTreeDetailsProjection::build(&symbol_tree_node, true, Some(4), None, None);

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
    fn build_normalizes_empty_runtime_value_names() {
        let symbol_tree_node = create_symbol_claim_node();
        let runtime_value_struct = ValuedStruct::new_anonymous(vec![
            DataTypeU32::get_value_from_primitive(100).to_named_valued_struct_field(String::new(), false),
        ]);
        let details_projection = SymbolTreeDetailsProjection::build(&symbol_tree_node, false, None, Some(&runtime_value_struct), None);
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
    fn build_adds_status_fields_for_fallback_projection() {
        let symbol_tree_node = create_symbol_claim_node();
        let details_projection = SymbolTreeDetailsProjection::build(&symbol_tree_node, true, None, None, Some("Unable to read symbol."));

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
        let details_projection = SymbolTreeDetailsProjection::build(&symbol_tree_node, false, None, Some(&runtime_value_struct), None);

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
