use crate::app_context::AppContext;
use crate::views::symbol_resolver_editor::view_data::symbol_resolver_editor_view_data::{
    SymbolResolverEditDraft, SymbolResolverEditorTakeOverState, SymbolResolverEditorViewData,
};
use eframe::egui::{Align, ComboBox, Direction, Key, Layout, RichText, ScrollArea, Ui, Widget};
use squalr_engine_api::commands::{
    privileged_command_request::PrivilegedCommandRequest, project::save::project_save_request::ProjectSaveRequest,
    registry::set_project_symbols::registry_set_project_symbols_request::RegistrySetProjectSymbolsRequest,
    unprivileged_command_request::UnprivilegedCommandRequest,
};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::{
    data_types::data_type_ref::DataTypeRef,
    projects::project_symbol_catalog::ProjectSymbolCatalog,
    structs::symbolic_resolver_definition::{SymbolicResolverBinaryOperator, SymbolicResolverNode},
};
use std::sync::Arc;

#[derive(Clone)]
pub struct SymbolResolverEditorView {
    app_context: Arc<AppContext>,
    symbol_resolver_editor_view_data: Dependency<SymbolResolverEditorViewData>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResolverNodeKind {
    Literal,
    LocalField,
    TypeSize,
    Binary,
}

impl SymbolResolverEditorView {
    pub const WINDOW_ID: &'static str = "window_symbol_resolver_editor";

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let symbol_resolver_editor_view_data = app_context
            .dependency_container
            .register(SymbolResolverEditorViewData::new());

        Self {
            app_context,
            symbol_resolver_editor_view_data,
        }
    }

    fn get_opened_project_symbol_catalog(&self) -> Option<ProjectSymbolCatalog> {
        let opened_project = self
            .app_context
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

    fn persist_project_symbol_catalog(
        &self,
        updated_project_symbol_catalog: ProjectSymbolCatalog,
    ) {
        let opened_project_lock = self
            .app_context
            .engine_unprivileged_state
            .get_project_manager()
            .get_opened_project();
        let did_update_project = match opened_project_lock.write() {
            Ok(mut opened_project) => {
                if let Some(opened_project) = opened_project.as_mut() {
                    let project_info = opened_project.get_project_info_mut();

                    *project_info.get_project_symbol_catalog_mut() = updated_project_symbol_catalog.clone();
                    project_info.set_has_unsaved_changes(true);
                    true
                } else {
                    false
                }
            }
            Err(error) => {
                log::error!("Failed to acquire opened project while persisting symbol resolver changes: {}.", error);
                false
            }
        };

        if !did_update_project {
            return;
        }

        ProjectSaveRequest {}.send(&self.app_context.engine_unprivileged_state, |project_save_response| {
            if !project_save_response.success {
                log::error!("Failed to save project after applying symbol resolver changes.");
            }
        });

        let registry_set_project_symbols_request = RegistrySetProjectSymbolsRequest {
            project_symbol_catalog: updated_project_symbol_catalog,
        };
        if !registry_set_project_symbols_request.send(&self.app_context.engine_unprivileged_state, |_response| {}) {
            log::error!("Failed to dispatch project symbol registry sync after symbol resolver changes.");
        }
    }

    fn default_data_type_ref(&self) -> DataTypeRef {
        self.app_context
            .engine_unprivileged_state
            .get_registered_data_type_refs()
            .first()
            .cloned()
            .unwrap_or_else(|| DataTypeRef::new("u32"))
    }

    fn render_list_panel(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_resolver_id: Option<&str>,
        filter_text: &str,
        is_take_over_active: bool,
    ) {
        user_interface.horizontal(|user_interface| {
            if user_interface
                .add_enabled(!is_take_over_active, eframe::egui::Button::new("+"))
                .on_hover_text("Create a new resolver.")
                .clicked()
            {
                if let Some(mut view_data) = self
                    .symbol_resolver_editor_view_data
                    .write("SymbolResolverEditor create resolver")
                {
                    view_data.begin_create_resolver(project_symbol_catalog);
                }
            }

            if user_interface
                .add_enabled(!is_take_over_active && selected_resolver_id.is_some(), eframe::egui::Button::new("Edit"))
                .on_hover_text("Edit the selected resolver.")
                .clicked()
            {
                if let Some(selected_resolver_id) = selected_resolver_id {
                    if let Some(mut view_data) = self
                        .symbol_resolver_editor_view_data
                        .write("SymbolResolverEditor edit resolver")
                    {
                        view_data.begin_edit_resolver(project_symbol_catalog, selected_resolver_id);
                    }
                }
            }

            if user_interface
                .add_enabled(!is_take_over_active && selected_resolver_id.is_some(), eframe::egui::Button::new("Delete"))
                .on_hover_text("Delete the selected resolver.")
                .clicked()
            {
                if let Some(selected_resolver_id) = selected_resolver_id {
                    if let Some(mut view_data) = self
                        .symbol_resolver_editor_view_data
                        .write("SymbolResolverEditor delete resolver")
                    {
                        view_data.request_delete_confirmation(selected_resolver_id.to_string());
                    }
                }
            }
        });

        user_interface.add_space(8.0);
        let mut edited_filter_text = filter_text.to_string();
        let filter_response = user_interface.text_edit_singleline(&mut edited_filter_text);
        if filter_response.changed() {
            if let Some(mut view_data) = self
                .symbol_resolver_editor_view_data
                .write("SymbolResolverEditor filter")
            {
                view_data.set_filter_text(edited_filter_text);
            }
        }

        user_interface.add_space(8.0);
        ScrollArea::vertical()
            .id_salt("symbol_resolver_editor_list")
            .show(user_interface, |user_interface| {
                for resolver_descriptor in project_symbol_catalog
                    .get_symbolic_resolver_descriptors()
                    .iter()
                    .filter(|resolver_descriptor| SymbolResolverEditorViewData::layout_matches_filter(resolver_descriptor, filter_text))
                {
                    let resolver_id = resolver_descriptor.get_resolver_id();
                    let selected = selected_resolver_id == Some(resolver_id);
                    let response = user_interface.selectable_label(selected, resolver_id);
                    if response.clicked() {
                        if let Some(mut view_data) = self
                            .symbol_resolver_editor_view_data
                            .write("SymbolResolverEditor select resolver")
                        {
                            view_data.select_resolver(Some(resolver_id.to_string()));
                        }
                    }
                    if response.double_clicked() && !is_take_over_active {
                        if let Some(mut view_data) = self
                            .symbol_resolver_editor_view_data
                            .write("SymbolResolverEditor edit resolver")
                        {
                            view_data.begin_edit_resolver(project_symbol_catalog, resolver_id);
                        }
                    }
                }

                if project_symbol_catalog
                    .get_symbolic_resolver_descriptors()
                    .is_empty()
                {
                    user_interface.label(RichText::new("No resolvers yet.").color(self.app_context.theme.foreground_preview));
                }
            });
    }

    fn render_resolver_take_over(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        title: &str,
        baseline_draft: Option<&SymbolResolverEditDraft>,
        draft: Option<&SymbolResolverEditDraft>,
    ) {
        let Some(draft) = draft else {
            return;
        };
        let baseline_draft = baseline_draft.unwrap_or(draft);
        let mut edited_draft = draft.clone();
        let validation_result = SymbolResolverEditorViewData::build_resolver_descriptor(project_symbol_catalog, &edited_draft);
        let has_unsaved_changes = edited_draft != *baseline_draft;
        let can_save = validation_result.is_ok() && has_unsaved_changes;
        let mut should_save = false;
        let mut should_cancel = false;

        user_interface.horizontal(|user_interface| {
            user_interface.heading(title);
            user_interface.with_layout(Layout::right_to_left(Align::Center), |user_interface| {
                if user_interface
                    .add_enabled(can_save, eframe::egui::Button::new("Save"))
                    .clicked()
                {
                    should_save = true;
                }
                if user_interface.button("Cancel").clicked() {
                    should_cancel = true;
                }
            });
        });
        user_interface.separator();

        user_interface.label("Resolver Id");
        user_interface.text_edit_singleline(&mut edited_draft.resolver_id);
        user_interface.add_space(8.0);
        user_interface.label("Resolver Tree");
        Self::render_resolver_node_editor(
            user_interface,
            edited_draft.resolver_definition.get_root_node_mut(),
            "symbol_resolver_editor_root",
            0,
            self.default_data_type_ref(),
        );
        user_interface.add_space(8.0);
        match validation_result {
            Ok(_) => {
                user_interface.label(RichText::new("Resolver is valid.").color(self.app_context.theme.foreground_preview));
            }
            Err(error) => {
                user_interface.label(RichText::new(error).color(self.app_context.theme.error_red));
            }
        }

        if should_cancel {
            if let Some(mut view_data) = self
                .symbol_resolver_editor_view_data
                .write("SymbolResolverEditor cancel resolver edit")
            {
                view_data.cancel_take_over_state();
            }
            return;
        }

        if should_save {
            match SymbolResolverEditorViewData::apply_draft_to_catalog(project_symbol_catalog, &edited_draft) {
                Ok(updated_project_symbol_catalog) => {
                    let saved_resolver_id = edited_draft.resolver_id.trim().to_string();
                    self.persist_project_symbol_catalog(updated_project_symbol_catalog);
                    if let Some(mut view_data) = self
                        .symbol_resolver_editor_view_data
                        .write("SymbolResolverEditor save resolver")
                    {
                        view_data.select_resolver(Some(saved_resolver_id));
                        view_data.cancel_take_over_state();
                    }
                    return;
                }
                Err(error) => {
                    log::error!("Failed to apply symbol resolver draft: {}.", error);
                }
            }
        }

        if let Some(mut view_data) = self
            .symbol_resolver_editor_view_data
            .write("SymbolResolverEditor update draft")
        {
            view_data.update_draft(edited_draft);
        }
    }

    fn render_delete_confirmation(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        resolver_id: &str,
    ) {
        let mut should_delete = false;
        let mut should_cancel = false;

        user_interface.heading("Delete Resolver");
        user_interface.separator();
        user_interface.label(format!("Delete `{}`?", resolver_id));
        user_interface.horizontal(|user_interface| {
            if user_interface.button("Delete").clicked() {
                should_delete = true;
            }
            if user_interface.button("Cancel").clicked() {
                should_cancel = true;
            }
        });

        if should_cancel {
            if let Some(mut view_data) = self
                .symbol_resolver_editor_view_data
                .write("SymbolResolverEditor cancel delete")
            {
                view_data.cancel_take_over_state();
            }
        }

        if should_delete {
            let updated_project_symbol_catalog = SymbolResolverEditorViewData::remove_resolver_from_catalog(project_symbol_catalog, resolver_id);
            self.persist_project_symbol_catalog(updated_project_symbol_catalog);
            if let Some(mut view_data) = self
                .symbol_resolver_editor_view_data
                .write("SymbolResolverEditor delete resolver")
            {
                view_data.cancel_take_over_state();
            }
        }
    }

    fn render_resolver_node_editor(
        user_interface: &mut Ui,
        resolver_node: &mut SymbolicResolverNode,
        id_salt: &str,
        depth: usize,
        default_data_type_ref: DataTypeRef,
    ) {
        user_interface.indent(id_salt, |user_interface| {
            let current_kind = Self::resolver_node_kind(resolver_node);
            let mut selected_kind = current_kind;
            user_interface.horizontal(|user_interface| {
                ComboBox::from_id_salt(format!("{}_kind", id_salt))
                    .selected_text(Self::resolver_node_kind_label(selected_kind))
                    .show_ui(user_interface, |user_interface| {
                        user_interface.selectable_value(&mut selected_kind, ResolverNodeKind::Literal, "Literal");
                        user_interface.selectable_value(&mut selected_kind, ResolverNodeKind::LocalField, "Local Field");
                        user_interface.selectable_value(&mut selected_kind, ResolverNodeKind::TypeSize, "Type Size");
                        user_interface.selectable_value(&mut selected_kind, ResolverNodeKind::Binary, "Operation");
                    });

                if selected_kind != current_kind {
                    *resolver_node = Self::default_node_for_kind(selected_kind, default_data_type_ref.clone());
                }

                match resolver_node {
                    SymbolicResolverNode::Literal(value) => {
                        let mut value_text = value.to_string();
                        if user_interface.text_edit_singleline(&mut value_text).changed() {
                            if let Ok(parsed_value) = value_text.trim().parse::<i128>() {
                                *value = parsed_value;
                            }
                        }
                    }
                    SymbolicResolverNode::LocalField { field_name } => {
                        user_interface.text_edit_singleline(field_name);
                    }
                    SymbolicResolverNode::TypeSize { data_type_ref } => {
                        let mut type_id = data_type_ref.get_data_type_id().to_string();
                        if user_interface.text_edit_singleline(&mut type_id).changed() {
                            *data_type_ref = DataTypeRef::new(type_id.trim());
                        }
                    }
                    SymbolicResolverNode::Binary { operator, .. } => {
                        ComboBox::from_id_salt(format!("{}_operator", id_salt))
                            .selected_text(operator.label())
                            .show_ui(user_interface, |user_interface| {
                                for candidate_operator in SymbolicResolverBinaryOperator::ALL {
                                    user_interface.selectable_value(operator, candidate_operator, candidate_operator.label());
                                }
                            });
                    }
                }
            });

            if let SymbolicResolverNode::Binary { left_node, right_node, .. } = resolver_node {
                Self::render_resolver_node_editor(
                    user_interface,
                    left_node,
                    &format!("{}_left_{}", id_salt, depth),
                    depth.saturating_add(1),
                    default_data_type_ref.clone(),
                );
                Self::render_resolver_node_editor(
                    user_interface,
                    right_node,
                    &format!("{}_right_{}", id_salt, depth),
                    depth.saturating_add(1),
                    default_data_type_ref,
                );
            }
        });
    }

    fn resolver_node_kind(resolver_node: &SymbolicResolverNode) -> ResolverNodeKind {
        match resolver_node {
            SymbolicResolverNode::Literal(_) => ResolverNodeKind::Literal,
            SymbolicResolverNode::LocalField { .. } => ResolverNodeKind::LocalField,
            SymbolicResolverNode::TypeSize { .. } => ResolverNodeKind::TypeSize,
            SymbolicResolverNode::Binary { .. } => ResolverNodeKind::Binary,
        }
    }

    fn resolver_node_kind_label(resolver_node_kind: ResolverNodeKind) -> &'static str {
        match resolver_node_kind {
            ResolverNodeKind::Literal => "Literal",
            ResolverNodeKind::LocalField => "Local Field",
            ResolverNodeKind::TypeSize => "Type Size",
            ResolverNodeKind::Binary => "Operation",
        }
    }

    fn default_node_for_kind(
        resolver_node_kind: ResolverNodeKind,
        default_data_type_ref: DataTypeRef,
    ) -> SymbolicResolverNode {
        match resolver_node_kind {
            ResolverNodeKind::Literal => SymbolicResolverNode::new_literal(0),
            ResolverNodeKind::LocalField => SymbolicResolverNode::new_local_field(String::from("field")),
            ResolverNodeKind::TypeSize => SymbolicResolverNode::new_type_size(default_data_type_ref),
            ResolverNodeKind::Binary => SymbolicResolverNode::new_binary(
                SymbolicResolverBinaryOperator::Add,
                SymbolicResolverNode::new_literal(0),
                SymbolicResolverNode::new_literal(0),
            ),
        }
    }
}

impl Widget for SymbolResolverEditorView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> eframe::egui::Response {
        let Some(project_symbol_catalog) = self.get_opened_project_symbol_catalog() else {
            return user_interface
                .allocate_ui_with_layout(
                    user_interface.available_size(),
                    Layout::centered_and_justified(Direction::TopDown),
                    |user_interface| {
                        user_interface
                            .label(RichText::new("Open a project to author reusable symbol resolvers.").color(self.app_context.theme.foreground_preview));
                    },
                )
                .response;
        };

        if let Some(mut view_data) = self
            .symbol_resolver_editor_view_data
            .write("SymbolResolverEditor synchronize")
        {
            view_data.synchronize(&project_symbol_catalog);
        }

        let (selected_resolver_id, filter_text, take_over_state, baseline_draft, draft) = self
            .symbol_resolver_editor_view_data
            .read("SymbolResolverEditor view")
            .map(|view_data| {
                (
                    view_data.get_selected_resolver_id().map(str::to_string),
                    view_data.get_filter_text().to_string(),
                    view_data.get_take_over_state().cloned(),
                    view_data.get_baseline_draft().cloned(),
                    view_data.get_draft().cloned(),
                )
            })
            .unwrap_or((None, String::new(), None, None, None));
        let is_take_over_active = take_over_state.is_some();
        let can_handle_window_shortcuts = self
            .app_context
            .window_focus_manager
            .can_window_handle_shortcuts(user_interface.ctx(), Self::WINDOW_ID);

        if can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::Escape)) && is_take_over_active {
            if let Some(mut view_data) = self
                .symbol_resolver_editor_view_data
                .write("SymbolResolverEditor escape")
            {
                view_data.cancel_take_over_state();
            }
        }

        user_interface
            .allocate_ui_with_layout(
                user_interface.available_size(),
                Layout::top_down(Align::Min),
                |user_interface| match take_over_state.as_ref() {
                    Some(SymbolResolverEditorTakeOverState::CreateResolver) => {
                        self.render_resolver_take_over(user_interface, &project_symbol_catalog, "New Resolver", baseline_draft.as_ref(), draft.as_ref());
                    }
                    Some(SymbolResolverEditorTakeOverState::EditResolver { .. }) => {
                        self.render_resolver_take_over(
                            user_interface,
                            &project_symbol_catalog,
                            "Edit Resolver",
                            baseline_draft.as_ref(),
                            draft.as_ref(),
                        );
                    }
                    Some(SymbolResolverEditorTakeOverState::DeleteConfirmation { resolver_id }) => {
                        self.render_delete_confirmation(user_interface, &project_symbol_catalog, resolver_id);
                    }
                    None => {
                        self.render_list_panel(
                            user_interface,
                            &project_symbol_catalog,
                            selected_resolver_id.as_deref(),
                            &filter_text,
                            is_take_over_active,
                        );
                    }
                },
            )
            .response
    }
}
