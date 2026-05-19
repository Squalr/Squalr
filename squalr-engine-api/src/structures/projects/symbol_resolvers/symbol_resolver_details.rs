use crate::structures::{
    data_types::{
        built_in_types::{i64::data_type_i64::DataTypeI64, string::utf8::data_type_string_utf8::DataTypeStringUtf8},
        data_type_ref::DataTypeRef,
    },
    data_values::{container_type::ContainerType, data_value::DataValue},
    details::{DetailsEdit, DetailsEditorHint, DetailsField, DetailsFieldId, DetailsFieldSource, DetailsProjection, DetailsTarget, DetailsValue},
    memory::symbolic_pointer_chain::SymbolicPointerChainLink,
    structs::symbolic_resolver_definition::{SymbolicResolverBinaryOperator, SymbolicResolverNode},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SymbolResolverDetailsNodeKind {
    Literal,
    LocalField,
    RelativeSymbolField,
    GlobalSymbolField,
    RelativePointerChain,
    GlobalPointerChain,
    TypeSize,
    Operation,
    Conditional,
}

impl SymbolResolverDetailsNodeKind {
    pub const ALL: [Self; 9] = [
        Self::Literal,
        Self::LocalField,
        Self::RelativeSymbolField,
        Self::GlobalSymbolField,
        Self::RelativePointerChain,
        Self::GlobalPointerChain,
        Self::TypeSize,
        Self::Operation,
        Self::Conditional,
    ];

    pub fn from_node(resolver_node: &SymbolicResolverNode) -> Self {
        match resolver_node {
            SymbolicResolverNode::Literal(_) => Self::Literal,
            SymbolicResolverNode::LocalField { .. } => Self::LocalField,
            SymbolicResolverNode::RelativeSymbolField { .. } => Self::RelativeSymbolField,
            SymbolicResolverNode::GlobalSymbolField { .. } => Self::GlobalSymbolField,
            SymbolicResolverNode::RelativePointerChain { .. } => Self::RelativePointerChain,
            SymbolicResolverNode::GlobalPointerChain { .. } => Self::GlobalPointerChain,
            SymbolicResolverNode::TypeSize { .. } => Self::TypeSize,
            SymbolicResolverNode::Binary { .. } => Self::Operation,
            SymbolicResolverNode::Conditional { .. } => Self::Conditional,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Literal => "Literal",
            Self::LocalField => "Local Field",
            Self::RelativeSymbolField => "Relative Symbol Field",
            Self::GlobalSymbolField => "Global Symbol Field",
            Self::RelativePointerChain => "Relative Pointer Chain",
            Self::GlobalPointerChain => "Global Pointer Chain",
            Self::TypeSize => "Type Size",
            Self::Operation => "Operation",
            Self::Conditional => "Conditional",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Literal => "literal",
            Self::LocalField => "local_field",
            Self::RelativeSymbolField => "relative_symbol_field",
            Self::GlobalSymbolField => "global_symbol_field",
            Self::RelativePointerChain => "relative_pointer_chain",
            Self::GlobalPointerChain => "global_pointer_chain",
            Self::TypeSize => "type_size",
            Self::Operation => "operation",
            Self::Conditional => "conditional",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        let trimmed_key = key.trim();

        Self::ALL
            .iter()
            .copied()
            .find(|node_kind| node_kind.key() == trimmed_key)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SymbolResolverDetailsEditOperation {
    UpdateResolverId(String),
    UpdateNodeKind(SymbolResolverDetailsNodeKind),
    UpdateLiteralValue(i64),
    UpdateLocalField(String),
    UpdateRelativeSymbolPath(String),
    UpdateGlobalModule(String),
    UpdateGlobalSymbolPath(String),
    UpdateDataType(String),
    UpdateOperator(SymbolicResolverBinaryOperator),
    NoOp,
    Reject(String),
}

pub struct SymbolResolverDetails;

impl SymbolResolverDetails {
    pub const TARGET_KIND_RESOLVER: &'static str = "symbol_resolver";
    pub const TARGET_KIND_NODE: &'static str = "symbol_resolver_node";

    pub const FIELD_ID_RESOLVER_ID: &'static str = "resolver.id";
    pub const FIELD_ID_NODE_KIND: &'static str = "node.kind";
    pub const FIELD_ID_LITERAL_VALUE: &'static str = "literal_value";
    pub const FIELD_ID_LOCAL_FIELD: &'static str = "local_field";
    pub const FIELD_ID_RELATIVE_SYMBOL_PATH: &'static str = "relative_symbol_path";
    pub const FIELD_ID_GLOBAL_MODULE: &'static str = "global_module";
    pub const FIELD_ID_GLOBAL_SYMBOL_PATH: &'static str = "global_symbol_path";
    pub const FIELD_ID_DATA_TYPE: &'static str = "data_type";
    pub const FIELD_ID_OPERATOR: &'static str = "operator";

    pub fn build_resolver_projection(resolver_id: &str) -> DetailsProjection {
        DetailsProjection::new(
            DetailsTarget::new(Self::TARGET_KIND_RESOLVER, resolver_id),
            resolver_id,
            vec![Self::text_field(
                Self::FIELD_ID_RESOLVER_ID,
                "Name",
                resolver_id,
                false,
            )],
        )
    }

    pub fn build_node_projection(
        resolver_id: &str,
        node_path: &[usize],
        resolver_node: &SymbolicResolverNode,
    ) -> DetailsProjection {
        let mut fields = vec![Self::text_field(
            Self::FIELD_ID_NODE_KIND,
            "Type",
            SymbolResolverDetailsNodeKind::from_node(resolver_node).key(),
            false,
        )];

        match resolver_node {
            SymbolicResolverNode::Literal(value) => {
                fields.push(DetailsField::new(
                    DetailsFieldId::new(Self::FIELD_ID_LITERAL_VALUE),
                    "Literal Value",
                    DetailsValue::DataValue(DataTypeI64::get_value_from_primitive(Self::clamp_i128_to_i64(*value))),
                    false,
                    DetailsEditorHint::Value,
                    Some(DataTypeRef::new(DataTypeI64::DATA_TYPE_ID)),
                    ContainerType::None,
                    Self::resolver_metadata_source(Self::FIELD_ID_LITERAL_VALUE),
                ));
            }
            SymbolicResolverNode::LocalField { field_name } => {
                fields.push(Self::text_field(Self::FIELD_ID_LOCAL_FIELD, "Local Field", field_name, false));
            }
            SymbolicResolverNode::RelativeSymbolField { symbol_path } => {
                fields.push(Self::text_field(
                    Self::FIELD_ID_RELATIVE_SYMBOL_PATH,
                    "Relative Path",
                    &symbol_path.to_string(),
                    false,
                ));
            }
            SymbolicResolverNode::GlobalSymbolField { module_name, symbol_path } => {
                fields.push(Self::text_field(Self::FIELD_ID_GLOBAL_MODULE, "Module", module_name, false));
                fields.push(Self::text_field(Self::FIELD_ID_GLOBAL_SYMBOL_PATH, "Path", &symbol_path.to_string(), false));
            }
            SymbolicResolverNode::RelativePointerChain { pointer_chain } => {
                fields.push(Self::text_field(
                    Self::FIELD_ID_RELATIVE_SYMBOL_PATH,
                    "Relative Path",
                    &SymbolicPointerChainLink::display_text_list(pointer_chain.get_links()),
                    false,
                ));
            }
            SymbolicResolverNode::GlobalPointerChain { pointer_chain } => {
                fields.push(Self::text_field(Self::FIELD_ID_GLOBAL_MODULE, "Module", pointer_chain.get_module_name(), false));
                fields.push(Self::text_field(
                    Self::FIELD_ID_GLOBAL_SYMBOL_PATH,
                    "Path",
                    &SymbolicPointerChainLink::display_text_list(pointer_chain.get_links()),
                    false,
                ));
            }
            SymbolicResolverNode::TypeSize { data_type_ref } => {
                fields.push(Self::text_field(Self::FIELD_ID_DATA_TYPE, "Data Type", data_type_ref.get_data_type_id(), false));
            }
            SymbolicResolverNode::Binary { operator, .. } => {
                fields.push(Self::text_field(Self::FIELD_ID_OPERATOR, "Operator", operator.key(), false));
            }
            SymbolicResolverNode::Conditional { .. } => {}
        }

        DetailsProjection::new(
            DetailsTarget::new(Self::TARGET_KIND_NODE, Self::node_target_id(resolver_id, node_path)),
            resolver_id,
            fields,
        )
    }

    pub fn plan_edit(details_edit: &DetailsEdit) -> SymbolResolverDetailsEditOperation {
        match details_edit.get_field_id().get_field_id() {
            Self::FIELD_ID_RESOLVER_ID => SymbolResolverDetailsEditOperation::UpdateResolverId(Self::text_value(details_edit).unwrap_or_default()),
            Self::FIELD_ID_NODE_KIND => Self::text_value(details_edit)
                .and_then(|text| SymbolResolverDetailsNodeKind::from_key(&text))
                .map(SymbolResolverDetailsEditOperation::UpdateNodeKind)
                .unwrap_or_else(|| SymbolResolverDetailsEditOperation::Reject(String::from("Unknown symbol resolver node kind."))),
            Self::FIELD_ID_LITERAL_VALUE => SymbolResolverDetailsEditOperation::UpdateLiteralValue(Self::i64_value(details_edit).unwrap_or_else(|| {
                Self::text_value(details_edit)
                    .and_then(|text| {
                        i64::from_str_radix(text.trim().trim_start_matches("0x"), 16)
                            .ok()
                            .or_else(|| text.trim().parse::<i64>().ok())
                    })
                    .unwrap_or(0)
            })),
            Self::FIELD_ID_LOCAL_FIELD => SymbolResolverDetailsEditOperation::UpdateLocalField(Self::text_value(details_edit).unwrap_or_default()),
            Self::FIELD_ID_RELATIVE_SYMBOL_PATH => {
                SymbolResolverDetailsEditOperation::UpdateRelativeSymbolPath(Self::text_value(details_edit).unwrap_or_default())
            }
            Self::FIELD_ID_GLOBAL_MODULE => SymbolResolverDetailsEditOperation::UpdateGlobalModule(Self::text_value(details_edit).unwrap_or_default()),
            Self::FIELD_ID_GLOBAL_SYMBOL_PATH => SymbolResolverDetailsEditOperation::UpdateGlobalSymbolPath(Self::text_value(details_edit).unwrap_or_default()),
            Self::FIELD_ID_DATA_TYPE => SymbolResolverDetailsEditOperation::UpdateDataType(Self::text_value(details_edit).unwrap_or_default()),
            Self::FIELD_ID_OPERATOR => Self::text_value(details_edit)
                .and_then(|text| SymbolicResolverBinaryOperator::from_key(&text))
                .map(SymbolResolverDetailsEditOperation::UpdateOperator)
                .unwrap_or_else(|| SymbolResolverDetailsEditOperation::Reject(String::from("Unknown symbol resolver operator."))),
            _ => SymbolResolverDetailsEditOperation::NoOp,
        }
    }

    fn text_field(
        field_id: &'static str,
        label: &'static str,
        value: &str,
        is_read_only: bool,
    ) -> DetailsField {
        let editor_hint = if field_id == Self::FIELD_ID_DATA_TYPE {
            DetailsEditorHint::DataType
        } else {
            DetailsEditorHint::Text
        };

        DetailsField::new(
            DetailsFieldId::new(field_id),
            label,
            DetailsValue::Text(value.to_string()),
            is_read_only,
            editor_hint,
            Some(DataTypeRef::new(DataTypeStringUtf8::DATA_TYPE_ID)),
            ContainerType::None,
            Self::resolver_metadata_source(field_id),
        )
    }

    fn resolver_metadata_source(metadata_name: &'static str) -> DetailsFieldSource {
        DetailsFieldSource::SymbolResolverMetadata {
            metadata_name: metadata_name.to_string(),
        }
    }

    fn node_target_id(
        resolver_id: &str,
        node_path: &[usize],
    ) -> String {
        let node_path_key = node_path
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(".");

        format!("{}|{}", resolver_id, node_path_key)
    }

    fn text_value(details_edit: &DetailsEdit) -> Option<String> {
        match details_edit.get_value() {
            DetailsValue::Text(text) => Some(text.clone()),
            DetailsValue::AnonymousValue(anonymous_value_string) => Some(anonymous_value_string.get_anonymous_value_string().to_string()),
            DetailsValue::DataValue(data_value) => String::from_utf8(data_value.get_value_bytes().clone()).ok(),
            DetailsValue::Boolean(value) => Some(value.to_string()),
            DetailsValue::UnsignedInteger(value) => Some(value.to_string()),
            DetailsValue::SignedInteger(value) => Some(value.to_string()),
            DetailsValue::Empty => None,
        }
    }

    fn i64_value(details_edit: &DetailsEdit) -> Option<i64> {
        match details_edit.get_value() {
            DetailsValue::SignedInteger(value) => Some(*value),
            DetailsValue::UnsignedInteger(value) => i64::try_from(*value).ok(),
            DetailsValue::DataValue(data_value) => Self::read_i64_data_value(data_value),
            DetailsValue::Text(_) | DetailsValue::AnonymousValue(_) | DetailsValue::Boolean(_) | DetailsValue::Empty => None,
        }
    }

    fn read_i64_data_value(data_value: &DataValue) -> Option<i64> {
        let value_bytes: [u8; 8] = data_value.get_value_bytes().as_slice().try_into().ok()?;

        Some(i64::from_le_bytes(value_bytes))
    }

    fn clamp_i128_to_i64(value: i128) -> i64 {
        value.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
    }
}

#[cfg(test)]
mod tests {
    use super::{SymbolResolverDetails, SymbolResolverDetailsEditOperation, SymbolResolverDetailsNodeKind};
    use crate::structures::{
        details::{DetailsEdit, DetailsFieldId, DetailsValue},
        structs::symbolic_resolver_definition::{SymbolicResolverBinaryOperator, SymbolicResolverNode},
    };

    #[test]
    fn node_projection_uses_stable_field_ids_for_resolver_metadata() {
        let resolver_node = SymbolicResolverNode::new_binary(
            SymbolicResolverBinaryOperator::Multiply,
            SymbolicResolverNode::new_literal(2),
            SymbolicResolverNode::new_literal(4),
        );
        let details_projection = SymbolResolverDetails::build_node_projection("scale", &[0], &resolver_node);

        assert_eq!(details_projection.get_target().get_target_kind(), SymbolResolverDetails::TARGET_KIND_NODE);
        assert!(
            details_projection
                .get_field(&DetailsFieldId::new(SymbolResolverDetails::FIELD_ID_NODE_KIND))
                .is_some()
        );
        assert!(
            details_projection
                .get_field(&DetailsFieldId::new(SymbolResolverDetails::FIELD_ID_OPERATOR))
                .is_some()
        );
    }

    #[test]
    fn edit_planner_routes_resolver_node_kind_without_display_label_parsing_by_gui() {
        let edit = DetailsEdit::new(
            crate::structures::details::DetailsTarget::new(SymbolResolverDetails::TARGET_KIND_NODE, "scale|0"),
            DetailsFieldId::new(SymbolResolverDetails::FIELD_ID_NODE_KIND),
            DetailsValue::Text("type_size".to_string()),
        );

        assert_eq!(
            SymbolResolverDetails::plan_edit(&edit),
            SymbolResolverDetailsEditOperation::UpdateNodeKind(SymbolResolverDetailsNodeKind::TypeSize)
        );
    }

    #[test]
    fn edit_planner_routes_operator_without_display_label_parsing_by_gui() {
        let edit = DetailsEdit::new(
            crate::structures::details::DetailsTarget::new(SymbolResolverDetails::TARGET_KIND_NODE, "scale|0"),
            DetailsFieldId::new(SymbolResolverDetails::FIELD_ID_OPERATOR),
            DetailsValue::Text("multiply".to_string()),
        );

        assert_eq!(
            SymbolResolverDetails::plan_edit(&edit),
            SymbolResolverDetailsEditOperation::UpdateOperator(SymbolicResolverBinaryOperator::Multiply)
        );
    }
}
