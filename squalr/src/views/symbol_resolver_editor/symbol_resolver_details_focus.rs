use crate::{
    app_context::AppContext,
    views::{
        struct_viewer::view_data::{struct_viewer_focus_target::StructViewerFocusTarget, struct_viewer_view_data::StructViewerViewData},
        symbol_resolver_editor::symbol_resolver_command_dispatcher::SymbolResolverCommandDispatcher,
        symbol_resolver_editor::view_data::symbol_resolver_editor_view_data::{SymbolResolverEditDraft, SymbolResolverEditorViewData, SymbolResolverNodeKind},
    },
};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::registries::symbols::symbolic_resolver_descriptor::SymbolicResolverDescriptor;
use squalr_engine_api::structures::{
    data_types::{
        built_in_types::{i64::data_type_i64::DataTypeI64, string::utf8::data_type_string_utf8::DataTypeStringUtf8},
        data_type_ref::DataTypeRef,
    },
    memory::symbolic_pointer_chain::{SymbolicPointerChain, SymbolicPointerChainLink},
    projects::project_symbol_catalog::ProjectSymbolCatalog,
    structs::{
        symbolic_resolver_definition::{SymbolicResolverBinaryOperator, SymbolicResolverNode, SymbolicResolverRelativeSymbolPath},
        valued_struct::ValuedStruct,
        valued_struct_field::ValuedStructField,
    },
};
use std::sync::Arc;

#[derive(Clone)]
pub struct SymbolResolverDetailsFocus {
    app_context: Arc<AppContext>,
    symbol_resolver_editor_view_data: Dependency<SymbolResolverEditorViewData>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
}

impl SymbolResolverDetailsFocus {
    const DETAILS_FIELD_LOCAL_FIELD: &'static str = "local_field";

    pub fn new(
        app_context: Arc<AppContext>,
        symbol_resolver_editor_view_data: Dependency<SymbolResolverEditorViewData>,
        struct_viewer_view_data: Dependency<StructViewerViewData>,
    ) -> Self {
        Self {
            app_context,
            symbol_resolver_editor_view_data,
            struct_viewer_view_data,
        }
    }

    pub fn focus_current_selection(&self) {
        let (selected_node_path, draft) = self
            .symbol_resolver_editor_view_data
            .read("SymbolResolverEditor focus details selection")
            .map(|view_data| (view_data.get_selected_node_path().map(<[usize]>::to_vec), view_data.get_draft().cloned()))
            .unwrap_or((None, None));
        let Some(draft) = draft else {
            return;
        };

        self.focus_draft_selection(selected_node_path, &draft);
    }

    pub fn clear_if_symbol_resolver_focused(&self) {
        let is_symbol_resolver_focused = self
            .struct_viewer_view_data
            .read("SymbolResolverEditor check details focus")
            .and_then(|struct_viewer_view_data| struct_viewer_view_data.get_focus_target().cloned())
            .is_some_and(|focus_target| matches!(focus_target, StructViewerFocusTarget::SymbolResolverEditor { .. }));

        if is_symbol_resolver_focused {
            StructViewerViewData::clear_focus(self.struct_viewer_view_data.clone());
        }
    }

    pub fn focus_resolver(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        resolver_id: &str,
    ) {
        if project_symbol_catalog
            .find_symbolic_resolver_descriptor(resolver_id)
            .is_none()
        {
            self.clear_if_symbol_resolver_focused();
            return;
        }

        let details_struct = Self::build_resolver_details_struct(resolver_id);
        let selection_key = format!("resolver|{}", resolver_id);
        let edit_callback = Self::build_struct_viewer_resolver_name_edit_callback(
            self.app_context.clone(),
            self.symbol_resolver_editor_view_data.clone(),
            self.struct_viewer_view_data.clone(),
            resolver_id.to_string(),
        );

        StructViewerViewData::focus_valued_struct_with_focus_target(
            self.struct_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            details_struct,
            edit_callback,
            Some(StructViewerFocusTarget::SymbolResolverEditor { selection_key }),
        );
    }

    fn focus_draft_selection(
        &self,
        selected_node_path: Option<Vec<usize>>,
        draft: &SymbolResolverEditDraft,
    ) {
        let Some(details_struct) = Self::build_details_struct(draft, selected_node_path.as_deref()) else {
            return;
        };
        let selection_key = Self::build_struct_viewer_focus_target_key(draft, selected_node_path.as_deref());
        let edit_callback = Self::build_struct_viewer_edit_callback(
            self.app_context.clone(),
            self.symbol_resolver_editor_view_data.clone(),
            self.struct_viewer_view_data.clone(),
            selected_node_path,
            self.default_data_type_ref(),
        );

        StructViewerViewData::focus_valued_struct_with_focus_target(
            self.struct_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            details_struct,
            edit_callback,
            Some(StructViewerFocusTarget::SymbolResolverEditor { selection_key }),
        );
    }

    fn default_data_type_ref(&self) -> DataTypeRef {
        self.app_context
            .engine_unprivileged_state
            .get_registered_data_type_refs()
            .first()
            .cloned()
            .unwrap_or_else(|| DataTypeRef::new("u32"))
    }

    fn build_resolver_details_struct(resolver_id: &str) -> ValuedStruct {
        ValuedStruct::new_anonymous(vec![
            DataTypeStringUtf8::get_value_from_primitive_string(resolver_id)
                .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_ID.to_string(), false),
        ])
    }

    fn build_struct_viewer_resolver_name_edit_callback(
        app_context: Arc<AppContext>,
        symbol_resolver_editor_view_data: Dependency<SymbolResolverEditorViewData>,
        struct_viewer_view_data: Dependency<StructViewerViewData>,
        resolver_id: String,
    ) -> Arc<dyn Fn(ValuedStructField) + Send + Sync> {
        Arc::new(move |edited_field: ValuedStructField| {
            let Some(project_symbol_catalog) = Self::get_opened_project_symbol_catalog_from_context(&app_context) else {
                return;
            };
            let Ok((original_resolver_id, resolver_descriptor)) =
                Self::build_resolver_descriptor_after_name_details_edit(&project_symbol_catalog, &resolver_id, &edited_field)
            else {
                return;
            };
            let saved_resolver_id = resolver_descriptor.get_resolver_id().to_string();

            SymbolResolverCommandDispatcher::new(app_context.clone()).persist_resolver_descriptor(original_resolver_id, &resolver_descriptor);

            if let Some(mut view_data) = symbol_resolver_editor_view_data.write("SymbolResolverEditor apply resolver name details edit") {
                view_data.select_resolver(Some(saved_resolver_id.clone()));
            }

            let details_struct = Self::build_resolver_details_struct(&saved_resolver_id);
            let selection_key = format!("resolver|{}", saved_resolver_id);
            let edit_callback = Self::build_struct_viewer_resolver_name_edit_callback(
                app_context.clone(),
                symbol_resolver_editor_view_data.clone(),
                struct_viewer_view_data.clone(),
                saved_resolver_id.clone(),
            );

            StructViewerViewData::focus_valued_struct_with_focus_target(
                struct_viewer_view_data.clone(),
                app_context.engine_unprivileged_state.clone(),
                details_struct,
                edit_callback,
                Some(StructViewerFocusTarget::SymbolResolverEditor { selection_key }),
            );
        })
    }

    fn get_opened_project_symbol_catalog_from_context(app_context: &Arc<AppContext>) -> Option<ProjectSymbolCatalog> {
        let opened_project = app_context
            .engine_unprivileged_state
            .get_project_manager()
            .get_opened_project();
        let opened_project = opened_project.read().ok()?;

        opened_project.as_ref().map(|opened_project| {
            opened_project
                .get_project_info()
                .get_project_symbol_catalog()
                .clone()
        })
    }

    fn build_resolver_descriptor_after_name_details_edit(
        project_symbol_catalog: &ProjectSymbolCatalog,
        current_resolver_id: &str,
        edited_field: &ValuedStructField,
    ) -> Result<(Option<String>, SymbolicResolverDescriptor), String> {
        if edited_field.get_name() != StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_ID {
            return Err(String::from("Edited field is not the resolver name."));
        }

        let edited_resolver_id = StructViewerViewData::read_utf8_field_text(edited_field)
            .trim()
            .to_string();
        if edited_resolver_id == current_resolver_id {
            return Err(String::from("Resolver name is unchanged."));
        }

        let resolver_descriptor = project_symbol_catalog
            .find_symbolic_resolver_descriptor(current_resolver_id)
            .ok_or_else(|| String::from("Resolver no longer exists."))?;
        let draft = SymbolResolverEditDraft {
            original_resolver_id: Some(current_resolver_id.to_string()),
            resolver_id: edited_resolver_id,
            resolver_definition: resolver_descriptor.get_resolver_definition().clone(),
        };
        let resolver_descriptor = SymbolResolverEditorViewData::build_resolver_descriptor(project_symbol_catalog, &draft)?;

        Ok((draft.original_resolver_id.clone(), resolver_descriptor))
    }

    fn build_struct_viewer_edit_callback(
        app_context: Arc<AppContext>,
        symbol_resolver_editor_view_data: Dependency<SymbolResolverEditorViewData>,
        struct_viewer_view_data: Dependency<StructViewerViewData>,
        selected_node_path: Option<Vec<usize>>,
        default_data_type_ref: DataTypeRef,
    ) -> Arc<dyn Fn(ValuedStructField) + Send + Sync> {
        Arc::new(move |edited_field: ValuedStructField| {
            let mut should_refocus_details = false;
            let updated_draft = {
                let Some(mut view_data) = symbol_resolver_editor_view_data.write("SymbolResolverEditor apply details edit") else {
                    return;
                };
                let Some(mut draft) = view_data.get_draft().cloned() else {
                    return;
                };

                if let Some(selected_node_path) = selected_node_path.as_deref() {
                    should_refocus_details = Self::apply_node_details_edit(&mut draft, selected_node_path, &edited_field, default_data_type_ref.clone());
                }

                view_data.update_draft(draft.clone());
                draft
            };

            if should_refocus_details {
                let Some(details_struct) = Self::build_details_struct(&updated_draft, selected_node_path.as_deref()) else {
                    return;
                };
                let selection_key = Self::build_struct_viewer_focus_target_key(&updated_draft, selected_node_path.as_deref());
                let edit_callback = Self::build_struct_viewer_edit_callback(
                    app_context.clone(),
                    symbol_resolver_editor_view_data.clone(),
                    struct_viewer_view_data.clone(),
                    selected_node_path.clone(),
                    default_data_type_ref.clone(),
                );

                StructViewerViewData::focus_valued_struct_with_focus_target(
                    struct_viewer_view_data.clone(),
                    app_context.engine_unprivileged_state.clone(),
                    details_struct,
                    edit_callback,
                    Some(StructViewerFocusTarget::SymbolResolverEditor { selection_key }),
                );
            }
        })
    }

    fn apply_node_details_edit(
        draft: &mut SymbolResolverEditDraft,
        selected_node_path: &[usize],
        edited_field: &ValuedStructField,
        default_data_type_ref: DataTypeRef,
    ) -> bool {
        let edited_field_name = edited_field.get_name();
        let edited_text = StructViewerViewData::read_utf8_field_text(edited_field);
        let Some(selected_node) = Self::get_node_mut(draft.resolver_definition.get_root_node_mut(), selected_node_path) else {
            return false;
        };

        match edited_field_name {
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_NODE_KIND => {
                let Some(next_kind) = Self::resolver_node_kind_from_label(&edited_text) else {
                    return false;
                };

                if next_kind != Self::resolver_node_kind(selected_node) {
                    *selected_node = SymbolResolverEditorViewData::default_node_for_kind(next_kind, default_data_type_ref);
                    return true;
                }
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_LITERAL_VALUE => {
                if let (SymbolicResolverNode::Literal(value), Some(parsed_value)) = (selected_node, Self::read_i64_field_value(edited_field)) {
                    *value = i128::from(parsed_value);
                }
            }
            Self::DETAILS_FIELD_LOCAL_FIELD => {
                if let SymbolicResolverNode::LocalField { field_name } = selected_node {
                    *field_name = edited_text;
                }
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_RELATIVE_SYMBOL_PATH => {
                if let SymbolicResolverNode::RelativeSymbolField { symbol_path } = selected_node {
                    *symbol_path = SymbolicResolverRelativeSymbolPath::from_dot_path(&edited_text);
                }
                if let SymbolicResolverNode::RelativePointerChain { pointer_chain } = selected_node {
                    *pointer_chain =
                        SymbolicPointerChain::new_absolute(SymbolicPointerChainLink::parse_text_list(&edited_text), pointer_chain.get_pointer_size());
                }
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_GLOBAL_MODULE => {
                if let SymbolicResolverNode::GlobalSymbolField { module_name, .. } = selected_node {
                    *module_name = edited_text.trim().to_string();
                }
                if let SymbolicResolverNode::GlobalPointerChain { pointer_chain } = selected_node {
                    pointer_chain.set_module_name(edited_text.trim().to_string());
                }
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_GLOBAL_SYMBOL_PATH => {
                if let SymbolicResolverNode::GlobalSymbolField { symbol_path, .. } = selected_node {
                    *symbol_path = SymbolicResolverRelativeSymbolPath::from_dot_path(&edited_text);
                }
                if let SymbolicResolverNode::GlobalPointerChain { pointer_chain } = selected_node {
                    *pointer_chain = SymbolicPointerChain::new(
                        pointer_chain.get_module_name().to_string(),
                        SymbolicPointerChainLink::parse_text_list(&edited_text),
                        pointer_chain.get_pointer_size(),
                    );
                }
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_DATA_TYPE => {
                if let SymbolicResolverNode::TypeSize { data_type_ref } = selected_node {
                    *data_type_ref = DataTypeRef::new(edited_text.trim());
                }
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_OPERATOR => {
                if let (SymbolicResolverNode::Binary { operator, .. }, Some(next_operator)) = (selected_node, Self::resolver_operator_from_label(&edited_text))
                {
                    *operator = next_operator;
                }
            }
            _ => {}
        }

        false
    }

    fn build_details_struct(
        draft: &SymbolResolverEditDraft,
        selected_node_path: Option<&[usize]>,
    ) -> Option<ValuedStruct> {
        let selected_node_path = selected_node_path?;
        let selected_node = Self::get_node(draft.resolver_definition.get_root_node(), selected_node_path)?;
        let mut fields = vec![
            DataTypeStringUtf8::get_value_from_primitive_string(Self::resolver_node_kind_label(Self::resolver_node_kind(selected_node)))
                .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_NODE_KIND.to_string(), false),
        ];

        match selected_node {
            SymbolicResolverNode::Literal(value) => {
                fields.push(
                    DataTypeI64::get_value_from_primitive(Self::clamp_i128_to_i64(*value))
                        .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_LITERAL_VALUE.to_string(), false),
                );
            }
            SymbolicResolverNode::LocalField { field_name } => {
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(field_name)
                        .to_named_valued_struct_field(Self::DETAILS_FIELD_LOCAL_FIELD.to_string(), false),
                );
            }
            SymbolicResolverNode::RelativeSymbolField { symbol_path } => {
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(&symbol_path.to_string())
                        .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_RELATIVE_SYMBOL_PATH.to_string(), false),
                );
            }
            SymbolicResolverNode::GlobalSymbolField { module_name, symbol_path } => {
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(module_name)
                        .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_GLOBAL_MODULE.to_string(), false),
                );
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(&symbol_path.to_string())
                        .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_GLOBAL_SYMBOL_PATH.to_string(), false),
                );
            }
            SymbolicResolverNode::RelativePointerChain { pointer_chain } => {
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(&SymbolicPointerChainLink::display_text_list(pointer_chain.get_links()))
                        .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_RELATIVE_SYMBOL_PATH.to_string(), false),
                );
            }
            SymbolicResolverNode::GlobalPointerChain { pointer_chain } => {
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(pointer_chain.get_module_name())
                        .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_GLOBAL_MODULE.to_string(), false),
                );
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(&SymbolicPointerChainLink::display_text_list(pointer_chain.get_links()))
                        .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_GLOBAL_SYMBOL_PATH.to_string(), false),
                );
            }
            SymbolicResolverNode::TypeSize { data_type_ref } => {
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(data_type_ref.get_data_type_id())
                        .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_DATA_TYPE.to_string(), false),
                );
            }
            SymbolicResolverNode::Binary { operator, .. } => {
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(operator.label())
                        .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_OPERATOR.to_string(), false),
                );
            }
            SymbolicResolverNode::Conditional { .. } => {}
        }

        Some(ValuedStruct::new_anonymous(fields))
    }

    fn build_struct_viewer_focus_target_key(
        draft: &SymbolResolverEditDraft,
        selected_node_path: Option<&[usize]>,
    ) -> String {
        let resolver_key = draft
            .original_resolver_id
            .as_deref()
            .unwrap_or(draft.resolver_id.as_str());
        let node_path_key = selected_node_path
            .map(|node_path| {
                node_path
                    .iter()
                    .map(usize::to_string)
                    .collect::<Vec<_>>()
                    .join(".")
            })
            .unwrap_or_default();

        format!("{}|{}", resolver_key, node_path_key)
    }

    fn resolver_node_kind(resolver_node: &SymbolicResolverNode) -> SymbolResolverNodeKind {
        match resolver_node {
            SymbolicResolverNode::Literal(_) => SymbolResolverNodeKind::Literal,
            SymbolicResolverNode::LocalField { .. } => SymbolResolverNodeKind::LocalField,
            SymbolicResolverNode::RelativeSymbolField { .. } => SymbolResolverNodeKind::RelativeSymbolField,
            SymbolicResolverNode::GlobalSymbolField { .. } => SymbolResolverNodeKind::GlobalSymbolField,
            SymbolicResolverNode::RelativePointerChain { .. } => SymbolResolverNodeKind::RelativePointerChain,
            SymbolicResolverNode::GlobalPointerChain { .. } => SymbolResolverNodeKind::GlobalPointerChain,
            SymbolicResolverNode::TypeSize { .. } => SymbolResolverNodeKind::TypeSize,
            SymbolicResolverNode::Binary { .. } => SymbolResolverNodeKind::Operation,
            SymbolicResolverNode::Conditional { .. } => SymbolResolverNodeKind::Conditional,
        }
    }

    fn resolver_node_kind_label(resolver_node_kind: SymbolResolverNodeKind) -> &'static str {
        match resolver_node_kind {
            SymbolResolverNodeKind::Literal => "Literal",
            SymbolResolverNodeKind::LocalField => "Local Field",
            SymbolResolverNodeKind::RelativeSymbolField => "Relative Symbol Field",
            SymbolResolverNodeKind::GlobalSymbolField => "Global Symbol Field",
            SymbolResolverNodeKind::RelativePointerChain => "Relative Pointer Chain",
            SymbolResolverNodeKind::GlobalPointerChain => "Global Pointer Chain",
            SymbolResolverNodeKind::TypeSize => "Type Size",
            SymbolResolverNodeKind::Operation => "Operation",
            SymbolResolverNodeKind::Conditional => "Conditional",
        }
    }

    fn resolver_node_kind_from_label(label: &str) -> Option<SymbolResolverNodeKind> {
        match label.trim() {
            "Literal" => Some(SymbolResolverNodeKind::Literal),
            "Local Field" => Some(SymbolResolverNodeKind::LocalField),
            "Relative Symbol Field" | "Symbol Field" => Some(SymbolResolverNodeKind::RelativeSymbolField),
            "Global Symbol Field" => Some(SymbolResolverNodeKind::GlobalSymbolField),
            "Relative Pointer Chain" => Some(SymbolResolverNodeKind::RelativePointerChain),
            "Global Pointer Chain" => Some(SymbolResolverNodeKind::GlobalPointerChain),
            "Type Size" => Some(SymbolResolverNodeKind::TypeSize),
            "Operation" => Some(SymbolResolverNodeKind::Operation),
            "Conditional" => Some(SymbolResolverNodeKind::Conditional),
            _ => None,
        }
    }

    fn resolver_operator_from_label(label: &str) -> Option<SymbolicResolverBinaryOperator> {
        SymbolicResolverBinaryOperator::ALL
            .iter()
            .copied()
            .find(|operator| operator.label() == label.trim())
    }

    fn get_node_mut<'resolver>(
        resolver_node: &'resolver mut SymbolicResolverNode,
        node_path: &[usize],
    ) -> Option<&'resolver mut SymbolicResolverNode> {
        if node_path.is_empty() {
            return Some(resolver_node);
        }

        match resolver_node {
            SymbolicResolverNode::Binary { left_node, right_node, .. } => match node_path[0] {
                0 => Self::get_node_mut(left_node, &node_path[1..]),
                1 => Self::get_node_mut(right_node, &node_path[1..]),
                _ => None,
            },
            SymbolicResolverNode::Conditional {
                condition_node,
                true_node,
                false_node,
            } => match node_path[0] {
                0 => Self::get_node_mut(condition_node, &node_path[1..]),
                1 => Self::get_node_mut(true_node, &node_path[1..]),
                2 => Self::get_node_mut(false_node, &node_path[1..]),
                _ => None,
            },
            SymbolicResolverNode::Literal(_)
            | SymbolicResolverNode::LocalField { .. }
            | SymbolicResolverNode::RelativeSymbolField { .. }
            | SymbolicResolverNode::GlobalSymbolField { .. }
            | SymbolicResolverNode::RelativePointerChain { .. }
            | SymbolicResolverNode::GlobalPointerChain { .. }
            | SymbolicResolverNode::TypeSize { .. } => None,
        }
    }

    fn get_node<'resolver>(
        resolver_node: &'resolver SymbolicResolverNode,
        node_path: &[usize],
    ) -> Option<&'resolver SymbolicResolverNode> {
        if node_path.is_empty() {
            return Some(resolver_node);
        }

        match resolver_node {
            SymbolicResolverNode::Binary { left_node, right_node, .. } => match node_path[0] {
                0 => Self::get_node(left_node, &node_path[1..]),
                1 => Self::get_node(right_node, &node_path[1..]),
                _ => None,
            },
            SymbolicResolverNode::Conditional {
                condition_node,
                true_node,
                false_node,
            } => match node_path[0] {
                0 => Self::get_node(condition_node, &node_path[1..]),
                1 => Self::get_node(true_node, &node_path[1..]),
                2 => Self::get_node(false_node, &node_path[1..]),
                _ => None,
            },
            SymbolicResolverNode::Literal(_)
            | SymbolicResolverNode::LocalField { .. }
            | SymbolicResolverNode::RelativeSymbolField { .. }
            | SymbolicResolverNode::GlobalSymbolField { .. }
            | SymbolicResolverNode::RelativePointerChain { .. }
            | SymbolicResolverNode::GlobalPointerChain { .. }
            | SymbolicResolverNode::TypeSize { .. } => None,
        }
    }

    fn clamp_i128_to_i64(value: i128) -> i64 {
        value.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
    }

    fn read_i64_field_value(valued_struct_field: &ValuedStructField) -> Option<i64> {
        let value_bytes = valued_struct_field.get_data_value()?.get_value_bytes();
        let value_bytes: [u8; 8] = value_bytes.as_slice().try_into().ok()?;

        Some(i64::from_le_bytes(value_bytes))
    }
}
