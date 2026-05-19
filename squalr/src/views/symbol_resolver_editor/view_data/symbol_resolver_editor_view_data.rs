use crate::ui::list_navigation::{ListNavigationDirection, resolve_next_index};
use epaint::Pos2;
use squalr_engine_api::registries::symbols::symbolic_resolver_descriptor::SymbolicResolverDescriptor;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::memory::symbolic_pointer_chain::{SymbolicPointerChain, SymbolicPointerChainLink};
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use squalr_engine_api::structures::structs::symbolic_resolver_definition::{
    SymbolicResolverBinaryOperator, SymbolicResolverDefinition, SymbolicResolverNode, SymbolicResolverRelativeSymbolPath,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolResolverEditorTakeOverState {
    CreateResolver,
    RenameResolver { resolver_id: String },
    OpenResolver { resolver_id: String },
    DeleteConfirmation { resolver_id: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolResolverEditDraft {
    pub original_resolver_id: Option<String>,
    pub resolver_id: String,
    pub resolver_definition: SymbolicResolverDefinition,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SymbolResolverContextMenuTarget {
    resolver_id: String,
    position: Pos2,
}

impl SymbolResolverContextMenuTarget {
    pub fn new(
        resolver_id: String,
        position: Pos2,
    ) -> Self {
        Self { resolver_id, position }
    }

    pub fn get_resolver_id(&self) -> &str {
        &self.resolver_id
    }

    pub fn get_position(&self) -> Pos2 {
        self.position
    }
}

#[derive(Clone, Default)]
pub struct SymbolResolverEditorViewData {
    selected_resolver_id: Option<String>,
    selected_node_path: Option<Vec<usize>>,
    resolver_context_menu_target: Option<SymbolResolverContextMenuTarget>,
    take_over_state: Option<SymbolResolverEditorTakeOverState>,
    baseline_draft: Option<SymbolResolverEditDraft>,
    draft: Option<SymbolResolverEditDraft>,
}

impl SymbolResolverEditorViewData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_selected_resolver_id(&self) -> Option<&str> {
        self.selected_resolver_id.as_deref()
    }

    pub fn get_selected_node_path(&self) -> Option<&[usize]> {
        self.selected_node_path.as_deref()
    }

    pub fn get_resolver_context_menu_target(&self) -> Option<&SymbolResolverContextMenuTarget> {
        self.resolver_context_menu_target.as_ref()
    }

    pub fn get_take_over_state(&self) -> Option<&SymbolResolverEditorTakeOverState> {
        self.take_over_state.as_ref()
    }

    pub fn get_baseline_draft(&self) -> Option<&SymbolResolverEditDraft> {
        self.baseline_draft.as_ref()
    }

    pub fn get_draft(&self) -> Option<&SymbolResolverEditDraft> {
        self.draft.as_ref()
    }

    pub fn select_resolver(
        &mut self,
        resolver_id: Option<String>,
    ) {
        self.selected_resolver_id = resolver_id;
        self.selected_node_path = None;
        self.resolver_context_menu_target = None;
    }

    pub fn navigate_resolver_selection(
        &mut self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        direction: ListNavigationDirection,
    ) -> Option<String> {
        let resolver_ids = project_symbol_catalog
            .get_symbolic_resolver_descriptors()
            .iter()
            .map(|resolver_descriptor| resolver_descriptor.get_resolver_id().to_string())
            .collect::<Vec<String>>();
        let selected_resolver_index = self
            .selected_resolver_id
            .as_ref()
            .and_then(|selected_resolver_id| {
                resolver_ids
                    .iter()
                    .position(|resolver_id| resolver_id == selected_resolver_id)
            });
        let next_selection_index = resolve_next_index(selected_resolver_index, resolver_ids.len(), direction)?;
        let next_resolver_id = resolver_ids.get(next_selection_index)?.clone();

        self.select_resolver(Some(next_resolver_id.clone()));

        Some(next_resolver_id)
    }

    pub fn select_node(
        &mut self,
        resolver_id: String,
        node_path: Vec<usize>,
    ) {
        self.selected_resolver_id = Some(resolver_id);
        self.selected_node_path = Some(node_path);
        self.resolver_context_menu_target = None;
    }

    pub fn begin_create_resolver(
        &mut self,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) {
        self.begin_create_resolver_with_root(project_symbol_catalog, SymbolicResolverNode::new_literal(0));
    }

    pub fn begin_create_resolver_with_root(
        &mut self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        root_node: SymbolicResolverNode,
    ) {
        let baseline_draft = Self::create_default_new_draft_with_root(project_symbol_catalog, root_node);

        self.selected_resolver_id = None;
        self.selected_node_path = None;
        self.resolver_context_menu_target = None;
        self.take_over_state = Some(SymbolResolverEditorTakeOverState::CreateResolver);
        self.baseline_draft = Some(baseline_draft.clone());
        self.draft = Some(baseline_draft);
    }

    pub fn begin_rename_resolver(
        &mut self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        resolver_id: &str,
    ) {
        self.selected_resolver_id = Some(resolver_id.to_string());
        self.selected_node_path = None;
        self.resolver_context_menu_target = None;
        self.take_over_state = Some(SymbolResolverEditorTakeOverState::RenameResolver {
            resolver_id: resolver_id.to_string(),
        });
        self.baseline_draft = project_symbol_catalog
            .find_symbolic_resolver_descriptor(resolver_id)
            .map(Self::create_draft_from_descriptor);
        self.draft = self.baseline_draft.clone();
    }

    pub fn begin_open_resolver(
        &mut self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        resolver_id: &str,
    ) {
        self.selected_resolver_id = Some(resolver_id.to_string());
        self.selected_node_path = Some(Vec::new());
        self.resolver_context_menu_target = None;
        self.take_over_state = Some(SymbolResolverEditorTakeOverState::OpenResolver {
            resolver_id: resolver_id.to_string(),
        });
        self.baseline_draft = project_symbol_catalog
            .find_symbolic_resolver_descriptor(resolver_id)
            .map(Self::create_draft_from_descriptor);
        self.draft = self.baseline_draft.clone();
    }

    pub fn begin_delete_confirmation(
        &mut self,
        resolver_id: &str,
    ) {
        self.selected_resolver_id = Some(resolver_id.to_string());
        self.selected_node_path = None;
        self.resolver_context_menu_target = None;
        self.take_over_state = Some(SymbolResolverEditorTakeOverState::DeleteConfirmation {
            resolver_id: resolver_id.to_string(),
        });
        self.baseline_draft = None;
        self.draft = None;
    }

    pub fn cancel_take_over_state(&mut self) {
        self.take_over_state = None;
        self.selected_node_path = None;
        self.resolver_context_menu_target = None;
        self.baseline_draft = None;
        self.draft = None;
    }

    pub fn show_resolver_context_menu(
        &mut self,
        resolver_id: String,
        position: Pos2,
    ) {
        self.selected_resolver_id = Some(resolver_id.clone());
        self.selected_node_path = None;
        self.resolver_context_menu_target = Some(SymbolResolverContextMenuTarget::new(resolver_id, position));
    }

    pub fn hide_resolver_context_menu(&mut self) {
        self.resolver_context_menu_target = None;
    }

    pub fn update_draft(
        &mut self,
        draft: SymbolResolverEditDraft,
    ) {
        self.draft = Some(draft);
    }

    pub fn update_draft_resolver_id(
        &mut self,
        resolver_id: String,
    ) {
        if let Some(draft) = self.draft.as_mut() {
            draft.resolver_id = resolver_id;
        }
    }

    pub fn synchronize(
        &mut self,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) {
        if matches!(self.take_over_state, Some(SymbolResolverEditorTakeOverState::CreateResolver)) {
            return;
        }

        self.selected_resolver_id = self
            .selected_resolver_id
            .as_ref()
            .filter(|selected_resolver_id| {
                project_symbol_catalog
                    .find_symbolic_resolver_descriptor(selected_resolver_id)
                    .is_some()
            })
            .cloned()
            .or_else(|| {
                project_symbol_catalog
                    .get_symbolic_resolver_descriptors()
                    .first()
                    .map(|resolver_descriptor| resolver_descriptor.get_resolver_id().to_string())
            });
        if self.selected_resolver_id.is_none() {
            self.selected_node_path = None;
        }

        self.resolver_context_menu_target = self
            .resolver_context_menu_target
            .as_ref()
            .filter(|context_menu_target| {
                project_symbol_catalog
                    .find_symbolic_resolver_descriptor(context_menu_target.get_resolver_id())
                    .is_some()
            })
            .cloned();

        let should_clear_take_over_state = match self.take_over_state.as_ref() {
            Some(SymbolResolverEditorTakeOverState::CreateResolver) => false,
            Some(
                SymbolResolverEditorTakeOverState::RenameResolver { resolver_id }
                | SymbolResolverEditorTakeOverState::OpenResolver { resolver_id }
                | SymbolResolverEditorTakeOverState::DeleteConfirmation { resolver_id },
            ) => project_symbol_catalog
                .find_symbolic_resolver_descriptor(resolver_id)
                .is_none(),
            None => false,
        };

        if should_clear_take_over_state {
            self.cancel_take_over_state();
        } else if matches!(self.take_over_state, Some(SymbolResolverEditorTakeOverState::OpenResolver { .. })) && self.selected_node_path.is_none() {
            self.selected_node_path = Some(Vec::new());
        }
    }

    pub fn create_default_new_draft(project_symbol_catalog: &ProjectSymbolCatalog) -> SymbolResolverEditDraft {
        Self::create_default_new_draft_with_root(project_symbol_catalog, SymbolicResolverNode::new_literal(0))
    }

    pub fn create_default_new_draft_with_root(
        project_symbol_catalog: &ProjectSymbolCatalog,
        root_node: SymbolicResolverNode,
    ) -> SymbolResolverEditDraft {
        let mut suffix_index = 1_u64;
        let mut proposed_resolver_id = String::from("new.resolver");
        while project_symbol_catalog
            .find_symbolic_resolver_descriptor(&proposed_resolver_id)
            .is_some()
        {
            suffix_index = suffix_index.saturating_add(1);
            proposed_resolver_id = format!("new.resolver{}", suffix_index);
        }

        SymbolResolverEditDraft {
            original_resolver_id: None,
            resolver_id: proposed_resolver_id,
            resolver_definition: SymbolicResolverDefinition::new(root_node),
        }
    }

    pub fn default_node_for_kind(
        resolver_node_kind: SymbolResolverNodeKind,
        default_data_type_ref: DataTypeRef,
    ) -> SymbolicResolverNode {
        match resolver_node_kind {
            SymbolResolverNodeKind::Literal => SymbolicResolverNode::new_literal(0),
            SymbolResolverNodeKind::LocalField => SymbolicResolverNode::new_local_field(String::from("field")),
            SymbolResolverNodeKind::RelativeSymbolField => {
                SymbolicResolverNode::new_relative_symbol_field(SymbolicResolverRelativeSymbolPath::from_dot_path("Symbol.Field"))
            }
            SymbolResolverNodeKind::GlobalSymbolField => {
                SymbolicResolverNode::new_global_symbol_field(String::from("module"), SymbolicResolverRelativeSymbolPath::from_dot_path("Symbol.Field"))
            }
            SymbolResolverNodeKind::RelativePointerChain => SymbolicResolverNode::new_relative_pointer_chain(SymbolicPointerChain::new_absolute(
                vec![SymbolicPointerChainLink::Offset(0)],
                PointerScanPointerSize::Pointer64,
            )),
            SymbolResolverNodeKind::GlobalPointerChain => SymbolicResolverNode::new_global_pointer_chain(SymbolicPointerChain::new(
                String::from("module"),
                vec![SymbolicPointerChainLink::Offset(0)],
                PointerScanPointerSize::Pointer64,
            )),
            SymbolResolverNodeKind::TypeSize => SymbolicResolverNode::new_type_size(default_data_type_ref),
            SymbolResolverNodeKind::Operation => SymbolicResolverNode::new_binary(
                SymbolicResolverBinaryOperator::Add,
                SymbolicResolverNode::new_literal(0),
                SymbolicResolverNode::new_literal(0),
            ),
            SymbolResolverNodeKind::Conditional => SymbolicResolverNode::new_conditional(
                SymbolicResolverNode::new_literal(1),
                SymbolicResolverNode::new_literal(0),
                SymbolicResolverNode::new_literal(0),
            ),
        }
    }

    pub fn create_draft_from_descriptor(resolver_descriptor: &SymbolicResolverDescriptor) -> SymbolResolverEditDraft {
        SymbolResolverEditDraft {
            original_resolver_id: Some(resolver_descriptor.get_resolver_id().to_string()),
            resolver_id: resolver_descriptor.get_resolver_id().to_string(),
            resolver_definition: resolver_descriptor.get_resolver_definition().clone(),
        }
    }

    pub fn count_resolver_usages(
        project_symbol_catalog: &ProjectSymbolCatalog,
        resolver_id: &str,
    ) -> usize {
        project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .flat_map(|struct_layout_descriptor| {
                struct_layout_descriptor
                    .get_struct_layout_definition()
                    .get_fields()
            })
            .map(|symbolic_field_definition| {
                let count_usage = usize::from(
                    symbolic_field_definition
                        .get_count_resolution()
                        .as_resolver_id()
                        == Some(resolver_id),
                );
                let offset_usage = usize::from(
                    symbolic_field_definition
                        .get_offset_resolution()
                        .as_resolver_id()
                        == Some(resolver_id),
                );

                count_usage.saturating_add(offset_usage)
            })
            .sum()
    }

    pub fn build_resolver_descriptor(
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &SymbolResolverEditDraft,
    ) -> Result<SymbolicResolverDescriptor, String> {
        let trimmed_resolver_id = draft.resolver_id.trim();
        if trimmed_resolver_id.is_empty() {
            return Err(String::from("Resolver id is required."));
        }

        let conflicts_with_existing_resolver = project_symbol_catalog
            .get_symbolic_resolver_descriptors()
            .iter()
            .any(|resolver_descriptor| {
                resolver_descriptor.get_resolver_id() == trimmed_resolver_id && draft.original_resolver_id.as_deref() != Some(trimmed_resolver_id)
            });
        if conflicts_with_existing_resolver {
            return Err(String::from("Resolver id must be unique."));
        }

        Ok(SymbolicResolverDescriptor::new(
            trimmed_resolver_id.to_string(),
            draft.resolver_definition.clone(),
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolResolverNodeKind {
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
