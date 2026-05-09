use crate::ui::list_navigation::{ListNavigationDirection, resolve_next_index};
use epaint::Pos2;
use squalr_engine_api::registries::symbols::symbolic_resolver_descriptor::SymbolicResolverDescriptor;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use squalr_engine_api::structures::structs::symbolic_resolver_definition::{SymbolicResolverBinaryOperator, SymbolicResolverDefinition, SymbolicResolverNode};

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
            SymbolResolverNodeKind::TypeSize => SymbolicResolverNode::new_type_size(default_data_type_ref),
            SymbolResolverNodeKind::Operation => SymbolicResolverNode::new_binary(
                SymbolicResolverBinaryOperator::Add,
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

    pub fn apply_draft_to_catalog(
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &SymbolResolverEditDraft,
    ) -> Result<ProjectSymbolCatalog, String> {
        let resolver_descriptor = Self::build_resolver_descriptor(project_symbol_catalog, draft)?;
        let mut updated_project_symbol_catalog = project_symbol_catalog.clone();
        let mut resolver_descriptors = updated_project_symbol_catalog
            .get_symbolic_resolver_descriptors()
            .iter()
            .filter(|existing_resolver_descriptor| draft.original_resolver_id.as_deref() != Some(existing_resolver_descriptor.get_resolver_id()))
            .cloned()
            .collect::<Vec<_>>();

        resolver_descriptors.push(resolver_descriptor);
        resolver_descriptors.sort_by(|left_resolver, right_resolver| {
            left_resolver
                .get_resolver_id()
                .to_ascii_lowercase()
                .cmp(&right_resolver.get_resolver_id().to_ascii_lowercase())
        });
        updated_project_symbol_catalog.set_symbolic_resolver_descriptors(resolver_descriptors);

        Ok(updated_project_symbol_catalog)
    }

    pub fn remove_resolver_from_catalog(
        project_symbol_catalog: &ProjectSymbolCatalog,
        resolver_id: &str,
    ) -> ProjectSymbolCatalog {
        let mut updated_project_symbol_catalog = project_symbol_catalog.clone();
        updated_project_symbol_catalog.set_symbolic_resolver_descriptors(
            updated_project_symbol_catalog
                .get_symbolic_resolver_descriptors()
                .iter()
                .filter(|resolver_descriptor| resolver_descriptor.get_resolver_id() != resolver_id)
                .cloned()
                .collect(),
        );

        updated_project_symbol_catalog
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolResolverNodeKind {
    Literal,
    LocalField,
    TypeSize,
    Operation,
}

#[cfg(test)]
mod tests {
    use super::{SymbolResolverEditDraft, SymbolResolverEditorTakeOverState, SymbolResolverEditorViewData};
    use squalr_engine_api::registries::symbols::{struct_layout_descriptor::StructLayoutDescriptor, symbolic_resolver_descriptor::SymbolicResolverDescriptor};
    use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
    use squalr_engine_api::structures::structs::{
        symbolic_field_definition::SymbolicFieldDefinition,
        symbolic_resolver_definition::{SymbolicResolverDefinition, SymbolicResolverNode},
        symbolic_struct_definition::SymbolicStructDefinition,
    };
    use std::str::FromStr;

    #[test]
    fn create_default_new_draft_picks_unique_resolver_id() {
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_resolvers_and_symbol_claims(
            Vec::new(),
            Vec::new(),
            vec![SymbolicResolverDescriptor::new(
                String::from("new.resolver"),
                SymbolicResolverDefinition::new(SymbolicResolverNode::new_literal(1)),
            )],
            Vec::new(),
        );

        let draft = SymbolResolverEditorViewData::create_default_new_draft(&project_symbol_catalog);

        assert_eq!(draft.resolver_id, "new.resolver2");
    }

    #[test]
    fn begin_delete_confirmation_selects_resolver_without_draft() {
        let mut view_data = SymbolResolverEditorViewData::new();

        view_data.begin_delete_confirmation("health.count");

        assert_eq!(view_data.get_selected_resolver_id(), Some("health.count"));
        assert_eq!(
            view_data.get_take_over_state(),
            Some(&SymbolResolverEditorTakeOverState::DeleteConfirmation {
                resolver_id: String::from("health.count"),
            })
        );
        assert!(view_data.get_draft().is_none());
    }

    #[test]
    fn apply_draft_to_catalog_adds_resolver_descriptor() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let draft = SymbolResolverEditDraft {
            original_resolver_id: None,
            resolver_id: String::from("inventory.count"),
            resolver_definition: SymbolicResolverDefinition::new(SymbolicResolverNode::new_local_field(String::from("count"))),
        };

        let updated_project_symbol_catalog =
            SymbolResolverEditorViewData::apply_draft_to_catalog(&project_symbol_catalog, &draft).expect("Expected resolver draft to apply.");

        assert!(
            updated_project_symbol_catalog
                .find_symbolic_resolver_descriptor("inventory.count")
                .is_some()
        );
    }

    #[test]
    fn build_resolver_descriptor_rejects_duplicate_ids() {
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_resolvers_and_symbol_claims(
            Vec::new(),
            Vec::new(),
            vec![SymbolicResolverDescriptor::new(
                String::from("inventory.count"),
                SymbolicResolverDefinition::new(SymbolicResolverNode::new_literal(1)),
            )],
            Vec::new(),
        );
        let draft = SymbolResolverEditDraft {
            original_resolver_id: None,
            resolver_id: String::from("inventory.count"),
            resolver_definition: SymbolicResolverDefinition::new(SymbolicResolverNode::new_literal(2)),
        };

        let result = SymbolResolverEditorViewData::build_resolver_descriptor(&project_symbol_catalog, &draft);

        assert!(result.is_err());
    }

    #[test]
    fn count_resolver_usages_includes_dynamic_counts_and_offsets() {
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_resolvers_and_symbol_claims(
            Vec::new(),
            vec![StructLayoutDescriptor::new(
                String::from("inventory"),
                SymbolicStructDefinition::new(
                    String::from("inventory"),
                    vec![
                        SymbolicFieldDefinition::from_str("count:u32").expect("Expected count field to parse."),
                        SymbolicFieldDefinition::from_str("items:u16[resolver(inventory.item_count)] @ resolver(inventory.items_offset)")
                            .expect("Expected items field to parse."),
                        SymbolicFieldDefinition::from_str("padding:u8[resolver(inventory.item_count)]").expect("Expected padding field to parse."),
                    ],
                ),
            )],
            Vec::new(),
            Vec::new(),
        );

        assert_eq!(
            SymbolResolverEditorViewData::count_resolver_usages(&project_symbol_catalog, "inventory.item_count"),
            2
        );
        assert_eq!(
            SymbolResolverEditorViewData::count_resolver_usages(&project_symbol_catalog, "inventory.items_offset"),
            1
        );
        assert_eq!(
            SymbolResolverEditorViewData::count_resolver_usages(&project_symbol_catalog, "inventory.unused"),
            0
        );
    }
}
