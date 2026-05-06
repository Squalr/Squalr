use squalr_engine_api::registries::symbols::symbolic_resolver_descriptor::SymbolicResolverDescriptor;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use squalr_engine_api::structures::structs::symbolic_resolver_definition::{SymbolicResolverDefinition, SymbolicResolverNode};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolResolverEditorTakeOverState {
    CreateResolver,
    EditResolver { resolver_id: String },
    DeleteConfirmation { resolver_id: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolResolverEditDraft {
    pub original_resolver_id: Option<String>,
    pub resolver_id: String,
    pub resolver_definition: SymbolicResolverDefinition,
}

#[derive(Clone, Default)]
pub struct SymbolResolverEditorViewData {
    selected_resolver_id: Option<String>,
    filter_text: String,
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

    pub fn get_filter_text(&self) -> &str {
        &self.filter_text
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

    pub fn set_filter_text(
        &mut self,
        filter_text: String,
    ) {
        self.filter_text = filter_text;
    }

    pub fn select_resolver(
        &mut self,
        resolver_id: Option<String>,
    ) {
        self.selected_resolver_id = resolver_id;
    }

    pub fn begin_create_resolver(
        &mut self,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) {
        let baseline_draft = Self::create_default_new_draft(project_symbol_catalog);

        self.selected_resolver_id = None;
        self.take_over_state = Some(SymbolResolverEditorTakeOverState::CreateResolver);
        self.baseline_draft = Some(baseline_draft.clone());
        self.draft = Some(baseline_draft);
    }

    pub fn begin_edit_resolver(
        &mut self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        resolver_id: &str,
    ) {
        self.selected_resolver_id = Some(resolver_id.to_string());
        self.take_over_state = Some(SymbolResolverEditorTakeOverState::EditResolver {
            resolver_id: resolver_id.to_string(),
        });
        self.baseline_draft = project_symbol_catalog
            .find_symbolic_resolver_descriptor(resolver_id)
            .map(Self::create_draft_from_descriptor);
        self.draft = self.baseline_draft.clone();
    }

    pub fn request_delete_confirmation(
        &mut self,
        resolver_id: String,
    ) {
        self.take_over_state = Some(SymbolResolverEditorTakeOverState::DeleteConfirmation { resolver_id });
        self.baseline_draft = None;
        self.draft = None;
    }

    pub fn cancel_take_over_state(&mut self) {
        self.take_over_state = None;
        self.baseline_draft = None;
        self.draft = None;
    }

    pub fn update_draft(
        &mut self,
        draft: SymbolResolverEditDraft,
    ) {
        self.draft = Some(draft);
    }

    pub fn synchronize(
        &mut self,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) {
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

        let should_clear_take_over_state = match self.take_over_state.as_ref() {
            Some(SymbolResolverEditorTakeOverState::CreateResolver) => false,
            Some(SymbolResolverEditorTakeOverState::EditResolver { resolver_id })
            | Some(SymbolResolverEditorTakeOverState::DeleteConfirmation { resolver_id }) => project_symbol_catalog
                .find_symbolic_resolver_descriptor(resolver_id)
                .is_none(),
            None => false,
        };

        if should_clear_take_over_state {
            self.cancel_take_over_state();
        }
    }

    pub fn layout_matches_filter(
        resolver_descriptor: &SymbolicResolverDescriptor,
        filter_text: &str,
    ) -> bool {
        let trimmed_filter_text = filter_text.trim();
        if trimmed_filter_text.is_empty() {
            return true;
        }

        resolver_descriptor
            .get_resolver_id()
            .to_ascii_lowercase()
            .contains(&trimmed_filter_text.to_ascii_lowercase())
    }

    pub fn create_default_new_draft(project_symbol_catalog: &ProjectSymbolCatalog) -> SymbolResolverEditDraft {
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
            resolver_definition: SymbolicResolverDefinition::new(SymbolicResolverNode::new_literal(0)),
        }
    }

    pub fn create_draft_from_descriptor(resolver_descriptor: &SymbolicResolverDescriptor) -> SymbolResolverEditDraft {
        SymbolResolverEditDraft {
            original_resolver_id: Some(resolver_descriptor.get_resolver_id().to_string()),
            resolver_id: resolver_descriptor.get_resolver_id().to_string(),
            resolver_definition: resolver_descriptor.get_resolver_definition().clone(),
        }
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

#[cfg(test)]
mod tests {
    use super::{SymbolResolverEditDraft, SymbolResolverEditorViewData};
    use squalr_engine_api::registries::symbols::symbolic_resolver_descriptor::SymbolicResolverDescriptor;
    use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
    use squalr_engine_api::structures::structs::symbolic_resolver_definition::{SymbolicResolverDefinition, SymbolicResolverNode};

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
}
