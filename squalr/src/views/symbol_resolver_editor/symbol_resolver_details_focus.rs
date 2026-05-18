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
    data_types::data_type_ref::DataTypeRef,
    details::DetailsEdit,
    memory::symbolic_pointer_chain::{SymbolicPointerChain, SymbolicPointerChainLink},
    projects::{
        project_symbol_catalog::ProjectSymbolCatalog,
        symbol_resolvers::symbol_resolver_details::{SymbolResolverDetails, SymbolResolverDetailsEditOperation, SymbolResolverDetailsNodeKind},
    },
    structs::symbolic_resolver_definition::{SymbolicResolverNode, SymbolicResolverRelativeSymbolPath},
};
use std::sync::Arc;

#[derive(Clone)]
pub struct SymbolResolverDetailsFocus {
    app_context: Arc<AppContext>,
    symbol_resolver_editor_view_data: Dependency<SymbolResolverEditorViewData>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
}

impl SymbolResolverDetailsFocus {
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

        let details_projection = SymbolResolverDetails::build_resolver_projection(resolver_id);
        let selection_key = format!("resolver|{}", resolver_id);
        let edit_callback = Self::build_struct_viewer_resolver_name_edit_callback(
            self.app_context.clone(),
            self.symbol_resolver_editor_view_data.clone(),
            self.struct_viewer_view_data.clone(),
            resolver_id.to_string(),
        );

        StructViewerViewData::focus_details_projection_with_focus_target(
            self.struct_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            details_projection,
            edit_callback,
            Some(StructViewerFocusTarget::SymbolResolverEditor { selection_key }),
        );
    }

    fn focus_draft_selection(
        &self,
        selected_node_path: Option<Vec<usize>>,
        draft: &SymbolResolverEditDraft,
    ) {
        let Some(details_projection) = Self::build_details_projection(draft, selected_node_path.as_deref()) else {
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

        StructViewerViewData::focus_details_projection_with_focus_target(
            self.struct_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            details_projection,
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

    fn build_struct_viewer_resolver_name_edit_callback(
        app_context: Arc<AppContext>,
        symbol_resolver_editor_view_data: Dependency<SymbolResolverEditorViewData>,
        struct_viewer_view_data: Dependency<StructViewerViewData>,
        resolver_id: String,
    ) -> Arc<dyn Fn(DetailsEdit) + Send + Sync> {
        Arc::new(move |details_edit: DetailsEdit| {
            let Some(project_symbol_catalog) = Self::get_opened_project_symbol_catalog_from_context(&app_context) else {
                return;
            };
            let Ok((original_resolver_id, resolver_descriptor)) =
                Self::build_resolver_descriptor_after_name_details_edit(&project_symbol_catalog, &resolver_id, &details_edit)
            else {
                return;
            };
            let saved_resolver_id = resolver_descriptor.get_resolver_id().to_string();

            SymbolResolverCommandDispatcher::new(app_context.clone()).persist_resolver_descriptor(original_resolver_id, &resolver_descriptor);

            if let Some(mut view_data) = symbol_resolver_editor_view_data.write("SymbolResolverEditor apply resolver name details edit") {
                view_data.select_resolver(Some(saved_resolver_id.clone()));
            }

            let details_projection = SymbolResolverDetails::build_resolver_projection(&saved_resolver_id);
            let selection_key = format!("resolver|{}", saved_resolver_id);
            let edit_callback = Self::build_struct_viewer_resolver_name_edit_callback(
                app_context.clone(),
                symbol_resolver_editor_view_data.clone(),
                struct_viewer_view_data.clone(),
                saved_resolver_id.clone(),
            );

            StructViewerViewData::focus_details_projection_with_focus_target(
                struct_viewer_view_data.clone(),
                app_context.engine_unprivileged_state.clone(),
                details_projection,
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
        details_edit: &DetailsEdit,
    ) -> Result<(Option<String>, SymbolicResolverDescriptor), String> {
        let SymbolResolverDetailsEditOperation::UpdateResolverId(edited_resolver_id) = SymbolResolverDetails::plan_edit(details_edit) else {
            return Err(String::from("Edited details field is not the resolver name."));
        };
        let edited_resolver_id = edited_resolver_id.trim().to_string();
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
    ) -> Arc<dyn Fn(DetailsEdit) + Send + Sync> {
        Arc::new(move |details_edit: DetailsEdit| {
            let mut should_refocus_details = false;
            let updated_draft = {
                let Some(mut view_data) = symbol_resolver_editor_view_data.write("SymbolResolverEditor apply details edit") else {
                    return;
                };
                let Some(mut draft) = view_data.get_draft().cloned() else {
                    return;
                };

                if let Some(selected_node_path) = selected_node_path.as_deref() {
                    should_refocus_details = Self::apply_node_details_edit(&mut draft, selected_node_path, &details_edit, default_data_type_ref.clone());
                }

                view_data.update_draft(draft.clone());
                draft
            };

            if should_refocus_details {
                let Some(details_projection) = Self::build_details_projection(&updated_draft, selected_node_path.as_deref()) else {
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

                StructViewerViewData::focus_details_projection_with_focus_target(
                    struct_viewer_view_data.clone(),
                    app_context.engine_unprivileged_state.clone(),
                    details_projection,
                    edit_callback,
                    Some(StructViewerFocusTarget::SymbolResolverEditor { selection_key }),
                );
            }
        })
    }

    fn apply_node_details_edit(
        draft: &mut SymbolResolverEditDraft,
        selected_node_path: &[usize],
        details_edit: &DetailsEdit,
        default_data_type_ref: DataTypeRef,
    ) -> bool {
        let Some(selected_node) = Self::get_node_mut(draft.resolver_definition.get_root_node_mut(), selected_node_path) else {
            return false;
        };

        match SymbolResolverDetails::plan_edit(details_edit) {
            SymbolResolverDetailsEditOperation::UpdateNodeKind(next_kind) => {
                if next_kind != SymbolResolverDetailsNodeKind::from_node(selected_node) {
                    *selected_node =
                        SymbolResolverEditorViewData::default_node_for_kind(Self::editor_node_kind_from_details_node_kind(next_kind), default_data_type_ref);
                    return true;
                }
            }
            SymbolResolverDetailsEditOperation::UpdateLiteralValue(parsed_value) => {
                if let SymbolicResolverNode::Literal(value) = selected_node {
                    *value = i128::from(parsed_value);
                }
            }
            SymbolResolverDetailsEditOperation::UpdateLocalField(edited_text) => {
                if let SymbolicResolverNode::LocalField { field_name } = selected_node {
                    *field_name = edited_text;
                }
            }
            SymbolResolverDetailsEditOperation::UpdateRelativeSymbolPath(edited_text) => {
                if let SymbolicResolverNode::RelativeSymbolField { symbol_path } = selected_node {
                    *symbol_path = SymbolicResolverRelativeSymbolPath::from_dot_path(&edited_text);
                }
                if let SymbolicResolverNode::RelativePointerChain { pointer_chain } = selected_node {
                    *pointer_chain =
                        SymbolicPointerChain::new_absolute(SymbolicPointerChainLink::parse_text_list(&edited_text), pointer_chain.get_pointer_size());
                }
            }
            SymbolResolverDetailsEditOperation::UpdateGlobalModule(edited_text) => {
                if let SymbolicResolverNode::GlobalSymbolField { module_name, .. } = selected_node {
                    *module_name = edited_text.trim().to_string();
                }
                if let SymbolicResolverNode::GlobalPointerChain { pointer_chain } = selected_node {
                    pointer_chain.set_module_name(edited_text.trim().to_string());
                }
            }
            SymbolResolverDetailsEditOperation::UpdateGlobalSymbolPath(edited_text) => {
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
            SymbolResolverDetailsEditOperation::UpdateDataType(edited_text) => {
                if let SymbolicResolverNode::TypeSize { data_type_ref } = selected_node {
                    *data_type_ref = DataTypeRef::new(edited_text.trim());
                }
            }
            SymbolResolverDetailsEditOperation::UpdateOperator(next_operator) => {
                if let SymbolicResolverNode::Binary { operator, .. } = selected_node {
                    *operator = next_operator;
                }
            }
            SymbolResolverDetailsEditOperation::UpdateResolverId(_)
            | SymbolResolverDetailsEditOperation::NoOp
            | SymbolResolverDetailsEditOperation::Reject(_) => {}
        }

        false
    }

    fn build_details_projection(
        draft: &SymbolResolverEditDraft,
        selected_node_path: Option<&[usize]>,
    ) -> Option<squalr_engine_api::structures::details::DetailsProjection> {
        let selected_node_path = selected_node_path?;
        let selected_node = Self::get_node(draft.resolver_definition.get_root_node(), selected_node_path)?;

        Some(SymbolResolverDetails::build_node_projection(
            draft
                .original_resolver_id
                .as_deref()
                .unwrap_or(&draft.resolver_id),
            selected_node_path,
            selected_node,
        ))
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

    fn editor_node_kind_from_details_node_kind(details_node_kind: SymbolResolverDetailsNodeKind) -> SymbolResolverNodeKind {
        match details_node_kind {
            SymbolResolverDetailsNodeKind::Literal => SymbolResolverNodeKind::Literal,
            SymbolResolverDetailsNodeKind::LocalField => SymbolResolverNodeKind::LocalField,
            SymbolResolverDetailsNodeKind::RelativeSymbolField => SymbolResolverNodeKind::RelativeSymbolField,
            SymbolResolverDetailsNodeKind::GlobalSymbolField => SymbolResolverNodeKind::GlobalSymbolField,
            SymbolResolverDetailsNodeKind::RelativePointerChain => SymbolResolverNodeKind::RelativePointerChain,
            SymbolResolverDetailsNodeKind::GlobalPointerChain => SymbolResolverNodeKind::GlobalPointerChain,
            SymbolResolverDetailsNodeKind::TypeSize => SymbolResolverNodeKind::TypeSize,
            SymbolResolverDetailsNodeKind::Operation => SymbolResolverNodeKind::Operation,
            SymbolResolverDetailsNodeKind::Conditional => SymbolResolverNodeKind::Conditional,
        }
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
}
