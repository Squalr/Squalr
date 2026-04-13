use crate::app_context::AppContext;
use crate::views::{
    code_viewer::{code_viewer_view::CodeViewerView, view_data::code_viewer_view_data::CodeViewerViewData},
    memory_viewer::{memory_viewer_view::MemoryViewerView, view_data::memory_viewer_view_data::MemoryViewerViewData},
    symbol_explorer::view_data::{
        symbol_explorer_view_data::{RootedSymbolDraftLocatorMode, SymbolExplorerSelection, SymbolExplorerViewData},
        symbol_tree_entry::{ResolvedPointerTarget, SymbolTreeEntry, SymbolTreeEntryKind, build_symbol_tree_entries},
    },
};
use eframe::egui::{Align, Button, Direction, Layout, Response, RichText, ScrollArea, Sense, TextEdit, Ui, UiBuilder, Widget, vec2};
use epaint::{CornerRadius, Rect, Stroke, pos2};
use squalr_engine_api::commands::{
    project_symbols::{
        create::project_symbols_create_request::ProjectSymbolsCreateRequest, delete::project_symbols_delete_request::ProjectSymbolsDeleteRequest,
        rename::project_symbols_rename_request::ProjectSymbolsRenameRequest,
    },
    unprivileged_command_request::UnprivilegedCommandRequest,
};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::memory::pointer::Pointer;
use squalr_engine_api::structures::projects::{
    project_root_symbol::ProjectRootSymbol, project_root_symbol_locator::ProjectRootSymbolLocator, project_symbol_catalog::ProjectSymbolCatalog,
};
use squalr_engine_api::structures::structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition};
use squalr_engine_session::virtual_snapshots::virtual_snapshot_query::VirtualSnapshotQuery;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct SymbolExplorerView {
    app_context: Arc<AppContext>,
    symbol_explorer_view_data: Dependency<SymbolExplorerViewData>,
    memory_viewer_view_data: Dependency<MemoryViewerViewData>,
    code_viewer_view_data: Dependency<CodeViewerViewData>,
}

impl SymbolExplorerView {
    pub const WINDOW_ID: &'static str = "window_symbol_explorer";
    const DETAILS_PANEL_WIDTH_RATIO: f32 = 0.42;
    const POINTER_CHILDREN_VIRTUAL_SNAPSHOT_ID: &'static str = "symbol_explorer_pointer_children";
    const POINTER_CHILDREN_REFRESH_INTERVAL: Duration = Duration::from_millis(250);

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let symbol_explorer_view_data = app_context
            .dependency_container
            .register(SymbolExplorerViewData::new());
        let memory_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<MemoryViewerViewData>();
        let code_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<CodeViewerViewData>();

        Self {
            app_context,
            symbol_explorer_view_data,
            memory_viewer_view_data,
            code_viewer_view_data,
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

    fn focus_memory_viewer_for_locator(
        &self,
        root_locator: &ProjectRootSymbolLocator,
    ) {
        MemoryViewerViewData::request_focus_address(
            self.memory_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            root_locator.get_focus_address(),
            root_locator.get_focus_module_name().to_string(),
        );

        match self.app_context.docking_manager.write() {
            Ok(mut docking_manager) => {
                docking_manager.set_window_visibility(MemoryViewerView::WINDOW_ID, true);
                docking_manager.select_tab_by_window_id(MemoryViewerView::WINDOW_ID);
            }
            Err(error) => {
                log::error!("Failed to acquire docking manager while opening memory viewer from Symbol Explorer: {}", error);
            }
        }
    }

    fn focus_code_viewer_for_locator(
        &self,
        root_locator: &ProjectRootSymbolLocator,
    ) {
        CodeViewerViewData::request_focus_address(
            self.code_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            root_locator.get_focus_address(),
            root_locator.get_focus_module_name().to_string(),
        );

        match self.app_context.docking_manager.write() {
            Ok(mut docking_manager) => {
                docking_manager.set_window_visibility(CodeViewerView::WINDOW_ID, true);
                docking_manager.select_tab_by_window_id(CodeViewerView::WINDOW_ID);
            }
            Err(error) => {
                log::error!("Failed to acquire docking manager while opening code viewer from Symbol Explorer: {}", error);
            }
        }
    }

    fn rename_rooted_symbol(
        &self,
        symbol_key: &str,
        display_name: String,
    ) {
        let project_symbols_rename_request = ProjectSymbolsRenameRequest {
            symbol_key: symbol_key.to_string(),
            display_name,
        };

        project_symbols_rename_request.send(&self.app_context.engine_unprivileged_state, |_project_symbols_rename_response| {});
    }

    fn delete_rooted_symbol(
        &self,
        symbol_key: &str,
    ) {
        let project_symbols_delete_request = ProjectSymbolsDeleteRequest {
            symbol_keys: vec![symbol_key.to_string()],
        };

        project_symbols_delete_request.send(&self.app_context.engine_unprivileged_state, |_project_symbols_delete_response| {});
    }

    fn create_rooted_symbol(
        &self,
        project_symbols_create_request: ProjectSymbolsCreateRequest,
    ) {
        let symbol_explorer_view_data = self.symbol_explorer_view_data.clone();

        project_symbols_create_request.send(&self.app_context.engine_unprivileged_state, move |project_symbols_create_response| {
            if project_symbols_create_response.success && !project_symbols_create_response.created_symbol_key.is_empty() {
                SymbolExplorerViewData::set_selected_entry(
                    symbol_explorer_view_data,
                    Some(SymbolExplorerSelection::RootedSymbol(project_symbols_create_response.created_symbol_key)),
                );
            }
        });
    }

    fn create_rooted_symbol_from_locator(
        &self,
        display_name: String,
        symbol_type_id: String,
        project_root_symbol_locator: ProjectRootSymbolLocator,
    ) {
        let (address, module_name, offset) = match project_root_symbol_locator {
            ProjectRootSymbolLocator::AbsoluteAddress { address } => (Some(address), None, None),
            ProjectRootSymbolLocator::ModuleOffset { module_name, offset } => (None, Some(module_name), Some(offset)),
        };

        self.create_rooted_symbol(ProjectSymbolsCreateRequest {
            display_name,
            struct_layout_id: symbol_type_id,
            address,
            module_name,
            offset,
            metadata: Default::default(),
        });
    }

    fn sync_pointer_child_virtual_snapshot(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entries: &[SymbolTreeEntry],
    ) {
        let pointer_snapshot_queries = self.build_pointer_snapshot_queries(project_symbol_catalog, symbol_tree_entries);

        self.app_context
            .engine_unprivileged_state
            .set_virtual_snapshot_queries(
                Self::POINTER_CHILDREN_VIRTUAL_SNAPSHOT_ID,
                Self::POINTER_CHILDREN_REFRESH_INTERVAL,
                pointer_snapshot_queries,
            );
        self.app_context
            .engine_unprivileged_state
            .request_virtual_snapshot_refresh(Self::POINTER_CHILDREN_VIRTUAL_SNAPSHOT_ID);
    }

    fn build_pointer_snapshot_queries(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entries: &[SymbolTreeEntry],
    ) -> Vec<VirtualSnapshotQuery> {
        symbol_tree_entries
            .iter()
            .filter(|symbol_tree_entry| {
                symbol_tree_entry.is_expanded()
                    && !matches!(symbol_tree_entry.get_kind(), SymbolTreeEntryKind::PointerTarget)
                    && symbol_tree_entry
                        .get_container_type()
                        .get_pointer_size()
                        .is_some()
            })
            .filter_map(|symbol_tree_entry| self.build_pointer_virtual_snapshot_query(project_symbol_catalog, symbol_tree_entry))
            .collect()
    }

    fn build_pointer_virtual_snapshot_query(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entry: &SymbolTreeEntry,
    ) -> Option<VirtualSnapshotQuery> {
        let pointer_size = symbol_tree_entry.get_container_type().get_pointer_size()?;
        let symbolic_struct_definition =
            self.build_symbolic_struct_definition_for_symbol_type(project_symbol_catalog, symbol_tree_entry.get_symbol_type_id())?;

        Some(VirtualSnapshotQuery::Pointer {
            query_id: symbol_tree_entry.get_node_key().to_string(),
            pointer: Pointer::new_with_size(
                symbol_tree_entry.get_locator().get_focus_address(),
                vec![0],
                symbol_tree_entry
                    .get_locator()
                    .get_focus_module_name()
                    .to_string(),
                pointer_size,
            ),
            symbolic_struct_definition,
        })
    }

    fn build_symbolic_struct_definition_for_symbol_type(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_type_id: &str,
    ) -> Option<SymbolicStructDefinition> {
        if let Some(project_struct_layout_descriptor) = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == symbol_type_id)
        {
            return Some(
                project_struct_layout_descriptor
                    .get_struct_layout_definition()
                    .clone(),
            );
        }

        if let Ok(symbolic_struct_definition) = SymbolicStructDefinition::from_str(symbol_type_id) {
            return Some(symbolic_struct_definition);
        }

        if let Some(symbolic_struct_definition) = self
            .app_context
            .engine_unprivileged_state
            .resolve_struct_layout_definition(symbol_type_id)
        {
            return Some(symbolic_struct_definition);
        }

        if let Ok(symbolic_field_definition) = SymbolicFieldDefinition::from_str(symbol_type_id) {
            return Some(SymbolicStructDefinition::new_anonymous(vec![symbolic_field_definition]));
        }

        Some(SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
            DataTypeRef::new(symbol_type_id),
            Default::default(),
        )]))
    }

    fn collect_resolved_pointer_targets_by_node_key(&self) -> HashMap<String, ResolvedPointerTarget> {
        let Some(virtual_snapshot) = self
            .app_context
            .engine_unprivileged_state
            .get_virtual_snapshot(Self::POINTER_CHILDREN_VIRTUAL_SNAPSHOT_ID)
        else {
            return HashMap::new();
        };

        virtual_snapshot
            .get_query_results()
            .iter()
            .filter_map(|(query_id, virtual_snapshot_query_result)| {
                let resolved_address = virtual_snapshot_query_result.resolved_address?;
                let target_locator = if virtual_snapshot_query_result.resolved_module_name.is_empty() {
                    ProjectRootSymbolLocator::new_absolute_address(resolved_address)
                } else {
                    ProjectRootSymbolLocator::new_module_offset(virtual_snapshot_query_result.resolved_module_name.clone(), resolved_address)
                };

                Some((
                    query_id.clone(),
                    ResolvedPointerTarget::new(target_locator, virtual_snapshot_query_result.evaluated_pointer_path.clone()),
                ))
            })
            .collect()
    }

    fn get_resolved_pointer_target_for_node_key(
        resolved_pointer_targets_by_node_key: &HashMap<String, ResolvedPointerTarget>,
        node_key: &str,
    ) -> Option<ResolvedPointerTarget> {
        resolved_pointer_targets_by_node_key.get(node_key).cloned()
    }

    fn render_symbol_tree_list(
        &self,
        user_interface: &mut Ui,
        symbol_tree_entries: &[SymbolTreeEntry],
        selected_entry: Option<&SymbolExplorerSelection>,
    ) {
        user_interface.label(
            RichText::new(format!(
                "Rooted Symbols ({})",
                symbol_tree_entries
                    .iter()
                    .filter(|symbol_tree_entry| matches!(symbol_tree_entry.get_kind(), SymbolTreeEntryKind::RootedSymbol { .. }))
                    .count()
            ))
            .font(
                self.app_context
                    .theme
                    .font_library
                    .font_noto_sans
                    .font_header
                    .clone(),
            )
            .color(self.app_context.theme.foreground),
        );
        user_interface.add_space(6.0);

        for symbol_tree_entry in symbol_tree_entries {
            let is_selected = matches!(
                selected_entry,
                Some(SymbolExplorerSelection::RootedSymbol(selected_symbol_key))
                    if matches!(symbol_tree_entry.get_kind(), SymbolTreeEntryKind::RootedSymbol { symbol_key } if selected_symbol_key == symbol_key)
            ) || matches!(
                selected_entry,
                Some(SymbolExplorerSelection::DerivedNode(selected_node_key)) if selected_node_key == symbol_tree_entry.get_node_key()
            );

            user_interface.horizontal(|user_interface| {
                user_interface.add_space(symbol_tree_entry.get_depth() as f32 * 16.0);

                if symbol_tree_entry.can_expand() {
                    let expansion_label = if symbol_tree_entry.is_expanded() { "▾" } else { "▸" };

                    if user_interface.button(expansion_label).clicked() {
                        SymbolExplorerViewData::toggle_tree_node_expansion(self.symbol_explorer_view_data.clone(), symbol_tree_entry.get_node_key());
                    }
                } else {
                    user_interface.add_space(24.0);
                }

                let row_label = format!(
                    "{}  [{}{}]",
                    symbol_tree_entry.get_display_name(),
                    symbol_tree_entry.get_symbol_type_id(),
                    symbol_tree_entry.get_container_type()
                );
                let response = user_interface.selectable_label(is_selected, row_label);

                if response.clicked() {
                    let selection = match symbol_tree_entry.get_kind() {
                        SymbolTreeEntryKind::RootedSymbol { symbol_key } => SymbolExplorerSelection::RootedSymbol(symbol_key.to_string()),
                        SymbolTreeEntryKind::StructField | SymbolTreeEntryKind::ArrayElement | SymbolTreeEntryKind::PointerTarget => {
                            SymbolExplorerSelection::DerivedNode(symbol_tree_entry.get_node_key().to_string())
                        }
                    };

                    SymbolExplorerViewData::set_selected_entry(self.symbol_explorer_view_data.clone(), Some(selection));
                }
            });

            user_interface.label(
                RichText::new(symbol_tree_entry.get_locator().to_string())
                    .font(
                        self.app_context
                            .theme
                            .font_library
                            .font_noto_sans
                            .font_small
                            .clone(),
                    )
                    .color(self.app_context.theme.foreground_preview),
            );
            user_interface.add_space(6.0);
        }
    }

    fn render_struct_layout_list(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_entry: Option<&SymbolExplorerSelection>,
    ) {
        user_interface.add_space(8.0);
        user_interface.label(
            RichText::new(format!("Symbol Types ({})", project_symbol_catalog.get_struct_layout_descriptors().len()))
                .font(
                    self.app_context
                        .theme
                        .font_library
                        .font_noto_sans
                        .font_header
                        .clone(),
                )
                .color(self.app_context.theme.foreground),
        );
        user_interface.add_space(6.0);

        for struct_layout_descriptor in project_symbol_catalog.get_struct_layout_descriptors() {
            let is_selected = matches!(
                selected_entry,
                Some(SymbolExplorerSelection::StructLayout(selected_struct_layout_id))
                    if selected_struct_layout_id == struct_layout_descriptor.get_struct_layout_id()
            );
            let response = user_interface.selectable_label(is_selected, struct_layout_descriptor.get_struct_layout_id());

            if response.clicked() {
                SymbolExplorerViewData::set_selected_entry(
                    self.symbol_explorer_view_data.clone(),
                    Some(SymbolExplorerSelection::StructLayout(
                        struct_layout_descriptor.get_struct_layout_id().to_string(),
                    )),
                );
            }

            user_interface.label(
                RichText::new(format!(
                    "{} field(s)",
                    struct_layout_descriptor
                        .get_struct_layout_definition()
                        .get_fields()
                        .len()
                ))
                .font(
                    self.app_context
                        .theme
                        .font_library
                        .font_noto_sans
                        .font_small
                        .clone(),
                )
                .color(self.app_context.theme.foreground_preview),
            );
            user_interface.add_space(6.0);
        }
    }

    fn render_rooted_symbol_details(
        &self,
        user_interface: &mut Ui,
        rooted_symbol: &ProjectRootSymbol,
    ) {
        let current_display_name_draft = self
            .symbol_explorer_view_data
            .read("Symbol explorer rooted symbol details")
            .map(|symbol_explorer_view_data| {
                symbol_explorer_view_data
                    .get_rooted_symbol_display_name_draft()
                    .to_string()
            })
            .unwrap_or_else(|| rooted_symbol.get_display_name().to_string());
        let mut edited_display_name_draft = current_display_name_draft.clone();

        user_interface.label(
            RichText::new("Display Name")
                .font(
                    self.app_context
                        .theme
                        .font_library
                        .font_noto_sans
                        .font_header
                        .clone(),
                )
                .color(self.app_context.theme.foreground),
        );
        let rename_response = user_interface.add(TextEdit::singleline(&mut edited_display_name_draft));

        if rename_response.changed() {
            SymbolExplorerViewData::set_rooted_symbol_display_name_draft(self.symbol_explorer_view_data.clone(), edited_display_name_draft.clone());
        }

        user_interface.add_space(6.0);
        user_interface.monospace(format!("key: {}", rooted_symbol.get_symbol_key()));
        user_interface.monospace(format!("type: {}", rooted_symbol.get_struct_layout_id()));
        user_interface.monospace(format!("locator: {}", rooted_symbol.get_root_locator()));
        user_interface.add_space(10.0);

        user_interface.horizontal(|user_interface| {
            let trimmed_display_name_draft = edited_display_name_draft.trim().to_string();
            let can_apply_rename = !trimmed_display_name_draft.is_empty() && trimmed_display_name_draft != rooted_symbol.get_display_name();

            if user_interface
                .add_enabled(can_apply_rename, Button::new("Apply Name"))
                .clicked()
            {
                self.rename_rooted_symbol(rooted_symbol.get_symbol_key(), trimmed_display_name_draft);
            }

            if user_interface.button("Open In Memory").clicked() {
                self.focus_memory_viewer_for_locator(rooted_symbol.get_root_locator());
            }

            if user_interface.button("Open In Code").clicked() {
                self.focus_code_viewer_for_locator(rooted_symbol.get_root_locator());
            }

            if user_interface.button("Delete Symbol").clicked() {
                self.delete_rooted_symbol(rooted_symbol.get_symbol_key());
            }
        });

        if !rooted_symbol.get_metadata().is_empty() {
            user_interface.add_space(12.0);
            user_interface.label(
                RichText::new("Metadata")
                    .font(
                        self.app_context
                            .theme
                            .font_library
                            .font_noto_sans
                            .font_header
                            .clone(),
                    )
                    .color(self.app_context.theme.foreground),
            );

            for (metadata_key, metadata_value) in rooted_symbol.get_metadata() {
                user_interface.monospace(format!("{} = {}", metadata_key, metadata_value));
            }
        }
    }

    fn render_derived_symbol_details(
        &self,
        user_interface: &mut Ui,
        symbol_tree_entry: &SymbolTreeEntry,
        resolved_pointer_target: Option<ResolvedPointerTarget>,
    ) {
        user_interface.label(
            RichText::new(symbol_tree_entry.get_display_name())
                .font(
                    self.app_context
                        .theme
                        .font_library
                        .font_noto_sans
                        .font_header
                        .clone(),
                )
                .color(self.app_context.theme.foreground),
        );
        user_interface.add_space(6.0);
        user_interface.monospace(format!("path: {}", symbol_tree_entry.get_full_path()));
        user_interface.monospace(format!(
            "type: {}{}",
            symbol_tree_entry.get_symbol_type_id(),
            symbol_tree_entry.get_container_type()
        ));
        user_interface.monospace(format!("locator: {}", symbol_tree_entry.get_locator()));

        if let Some(resolved_pointer_target) = resolved_pointer_target {
            user_interface.monospace(format!("resolved target: {}", resolved_pointer_target.get_target_locator()));

            if !resolved_pointer_target.get_evaluated_pointer_path().is_empty() {
                user_interface.monospace(format!("pointer path: {}", resolved_pointer_target.get_evaluated_pointer_path()));
            }
        }

        user_interface.add_space(10.0);

        user_interface.horizontal(|user_interface| {
            if user_interface.button("Promote to Rooted Symbol").clicked() {
                self.create_rooted_symbol_from_locator(
                    symbol_tree_entry.get_promotion_display_name().to_string(),
                    symbol_tree_entry.get_promoted_symbol_type_id(),
                    symbol_tree_entry.get_locator().clone(),
                );
            }

            if user_interface.button("Open In Memory").clicked() {
                self.focus_memory_viewer_for_locator(symbol_tree_entry.get_locator());
            }

            if user_interface.button("Open In Code").clicked() {
                self.focus_code_viewer_for_locator(symbol_tree_entry.get_locator());
            }
        });
    }

    fn render_struct_layout_details(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        struct_layout_id: &str,
    ) {
        let Some(struct_layout_descriptor) = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == struct_layout_id)
        else {
            user_interface.label("Selected symbol type no longer exists.");
            return;
        };

        user_interface.label(
            RichText::new(struct_layout_descriptor.get_struct_layout_id())
                .font(
                    self.app_context
                        .theme
                        .font_library
                        .font_noto_sans
                        .font_header
                        .clone(),
                )
                .color(self.app_context.theme.foreground),
        );
        user_interface.add_space(6.0);
        user_interface.monospace(format!(
            "{} field(s)",
            struct_layout_descriptor
                .get_struct_layout_definition()
                .get_fields()
                .len()
        ));
        user_interface.add_space(10.0);

        for field_definition in struct_layout_descriptor
            .get_struct_layout_definition()
            .get_fields()
        {
            let unit_size_in_bytes = self
                .app_context
                .engine_unprivileged_state
                .get_default_value(field_definition.get_data_type_ref())
                .map(|default_value| default_value.get_size_in_bytes())
                .unwrap_or(1);
            let field_name = if field_definition.get_field_name().is_empty() {
                "(anonymous)"
            } else {
                field_definition.get_field_name()
            };

            user_interface.label(
                RichText::new(format!(
                    "{}: {}{}",
                    field_name,
                    field_definition.get_data_type_ref(),
                    field_definition.get_container_type()
                ))
                .font(
                    self.app_context
                        .theme
                        .font_library
                        .font_noto_sans
                        .font_normal
                        .clone(),
                )
                .color(self.app_context.theme.foreground),
            );
            user_interface.label(
                RichText::new(format!(
                    "{} byte(s)",
                    field_definition
                        .get_container_type()
                        .get_total_size_in_bytes(unit_size_in_bytes)
                ))
                .font(
                    self.app_context
                        .theme
                        .font_library
                        .font_noto_sans
                        .font_small
                        .clone(),
                )
                .color(self.app_context.theme.foreground_preview),
            );
            user_interface.add_space(4.0);
        }
    }

    fn render_create_rooted_symbol_details(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) {
        let original_draft = self
            .symbol_explorer_view_data
            .read("Symbol explorer rooted symbol create details")
            .map(|symbol_explorer_view_data| {
                symbol_explorer_view_data
                    .get_rooted_symbol_create_draft()
                    .clone()
            })
            .unwrap_or_default();
        let mut edited_draft = original_draft.clone();

        user_interface.label(
            RichText::new("New Rooted Symbol")
                .font(
                    self.app_context
                        .theme
                        .font_library
                        .font_noto_sans
                        .font_header
                        .clone(),
                )
                .color(self.app_context.theme.foreground),
        );
        user_interface.add_space(6.0);

        user_interface.label("Display Name");
        user_interface.add(TextEdit::singleline(&mut edited_draft.display_name));
        user_interface.add_space(6.0);

        user_interface.label("Type Id");
        user_interface.add(TextEdit::singleline(&mut edited_draft.struct_layout_id));
        user_interface.add_space(6.0);

        user_interface.label("Locator");
        user_interface.horizontal(|user_interface| {
            let is_absolute_address = matches!(edited_draft.locator_mode, RootedSymbolDraftLocatorMode::AbsoluteAddress);

            if user_interface
                .selectable_label(is_absolute_address, "Absolute Address")
                .clicked()
            {
                edited_draft.locator_mode = RootedSymbolDraftLocatorMode::AbsoluteAddress;
            }

            if user_interface
                .selectable_label(!is_absolute_address, "Module + Offset")
                .clicked()
            {
                edited_draft.locator_mode = RootedSymbolDraftLocatorMode::ModuleOffset;
            }
        });
        user_interface.add_space(6.0);

        match edited_draft.locator_mode {
            RootedSymbolDraftLocatorMode::AbsoluteAddress => {
                user_interface.label("Address");
                user_interface.add(TextEdit::singleline(&mut edited_draft.address_text).hint_text("0x12345678 or 305419896"));
            }
            RootedSymbolDraftLocatorMode::ModuleOffset => {
                user_interface.label("Module");
                user_interface.add(TextEdit::singleline(&mut edited_draft.module_name));
                user_interface.add_space(6.0);
                user_interface.label("Offset");
                user_interface.add(TextEdit::singleline(&mut edited_draft.offset_text).hint_text("0x1234 or 4660"));
            }
        }

        if edited_draft != original_draft {
            SymbolExplorerViewData::set_rooted_symbol_create_draft(self.symbol_explorer_view_data.clone(), edited_draft.clone());
        }

        let parsed_address = Self::parse_u64_draft(&edited_draft.address_text);
        let parsed_offset = Self::parse_u64_draft(&edited_draft.offset_text);
        let can_create_rooted_symbol = !edited_draft.display_name.trim().is_empty()
            && !edited_draft.struct_layout_id.trim().is_empty()
            && project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == edited_draft.struct_layout_id.trim())
            && match edited_draft.locator_mode {
                RootedSymbolDraftLocatorMode::AbsoluteAddress => parsed_address.is_some(),
                RootedSymbolDraftLocatorMode::ModuleOffset => !edited_draft.module_name.trim().is_empty() && parsed_offset.is_some(),
            };

        user_interface.add_space(10.0);
        if user_interface
            .add_enabled(can_create_rooted_symbol, Button::new("Create Rooted Symbol"))
            .clicked()
        {
            self.create_rooted_symbol(ProjectSymbolsCreateRequest {
                display_name: edited_draft.display_name.trim().to_string(),
                struct_layout_id: edited_draft.struct_layout_id.trim().to_string(),
                address: match edited_draft.locator_mode {
                    RootedSymbolDraftLocatorMode::AbsoluteAddress => parsed_address,
                    RootedSymbolDraftLocatorMode::ModuleOffset => None,
                },
                module_name: match edited_draft.locator_mode {
                    RootedSymbolDraftLocatorMode::AbsoluteAddress => None,
                    RootedSymbolDraftLocatorMode::ModuleOffset => Some(edited_draft.module_name.trim().to_string()),
                },
                offset: match edited_draft.locator_mode {
                    RootedSymbolDraftLocatorMode::AbsoluteAddress => None,
                    RootedSymbolDraftLocatorMode::ModuleOffset => parsed_offset,
                },
                metadata: Default::default(),
            });
        }

        if !project_symbol_catalog
            .get_struct_layout_descriptors()
            .is_empty()
        {
            user_interface.add_space(12.0);
            user_interface.label(
                RichText::new("Available Types")
                    .font(
                        self.app_context
                            .theme
                            .font_library
                            .font_noto_sans
                            .font_header
                            .clone(),
                    )
                    .color(self.app_context.theme.foreground),
            );

            for struct_layout_descriptor in project_symbol_catalog.get_struct_layout_descriptors() {
                user_interface.monospace(struct_layout_descriptor.get_struct_layout_id());
            }
        }
    }

    fn parse_u64_draft(numeric_draft: &str) -> Option<u64> {
        let trimmed_numeric_draft = numeric_draft.trim();

        if trimmed_numeric_draft.is_empty() {
            return None;
        }

        if let Some(hex_numeric_draft) = trimmed_numeric_draft
            .strip_prefix("0x")
            .or_else(|| trimmed_numeric_draft.strip_prefix("0X"))
        {
            u64::from_str_radix(hex_numeric_draft, 16).ok()
        } else {
            trimmed_numeric_draft.parse::<u64>().ok()
        }
    }
}

impl Widget for SymbolExplorerView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let Some(project_symbol_catalog) = self.get_opened_project_symbol_catalog() else {
            self.app_context
                .engine_unprivileged_state
                .set_virtual_snapshot_queries(Self::POINTER_CHILDREN_VIRTUAL_SNAPSHOT_ID, Self::POINTER_CHILDREN_REFRESH_INTERVAL, Vec::new());

            return user_interface
                .allocate_ui_with_layout(
                    user_interface.available_size(),
                    Layout::centered_and_justified(Direction::TopDown),
                    |user_interface| {
                        user_interface.label("Open a project to browse symbol types and rooted symbols.");
                    },
                )
                .response;
        };

        SymbolExplorerViewData::synchronize_selection(self.symbol_explorer_view_data.clone(), &project_symbol_catalog);
        SymbolExplorerViewData::synchronize_rooted_symbol_display_name_draft(self.symbol_explorer_view_data.clone(), &project_symbol_catalog);
        SymbolExplorerViewData::synchronize_rooted_symbol_create_draft(self.symbol_explorer_view_data.clone(), &project_symbol_catalog);
        let expanded_tree_node_keys = self
            .symbol_explorer_view_data
            .read("Symbol explorer expanded tree nodes")
            .map(|symbol_explorer_view_data| symbol_explorer_view_data.get_expanded_tree_node_keys().clone())
            .unwrap_or_default();
        let structural_symbol_tree_entries = build_symbol_tree_entries(&project_symbol_catalog, &expanded_tree_node_keys, &HashMap::new(), |data_type_ref| {
            self.app_context
                .engine_unprivileged_state
                .get_default_value(data_type_ref)
                .map(|default_value| default_value.get_size_in_bytes())
        });
        self.sync_pointer_child_virtual_snapshot(&project_symbol_catalog, &structural_symbol_tree_entries);
        let resolved_pointer_targets_by_node_key = self.collect_resolved_pointer_targets_by_node_key();
        let symbol_tree_entries = build_symbol_tree_entries(
            &project_symbol_catalog,
            &expanded_tree_node_keys,
            &resolved_pointer_targets_by_node_key,
            |data_type_ref| {
                self.app_context
                    .engine_unprivileged_state
                    .get_default_value(data_type_ref)
                    .map(|default_value| default_value.get_size_in_bytes())
            },
        );
        SymbolExplorerViewData::synchronize_selection_to_tree_entries(self.symbol_explorer_view_data.clone(), &symbol_tree_entries);
        let selected_entry = self
            .symbol_explorer_view_data
            .read("Symbol explorer view")
            .and_then(|symbol_explorer_view_data| symbol_explorer_view_data.get_selected_entry().cloned());
        let theme = self.app_context.theme.clone();

        user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                let toolbar_height = 28.0;
                let (toolbar_rect, _toolbar_response) =
                    user_interface.allocate_exact_size(vec2(user_interface.available_width().max(1.0), toolbar_height), Sense::empty());
                user_interface
                    .painter()
                    .rect_filled(toolbar_rect, CornerRadius::ZERO, theme.background_primary);

                let mut toolbar_user_interface = user_interface.new_child(
                    UiBuilder::new()
                        .max_rect(toolbar_rect)
                        .layout(Layout::left_to_right(Align::Center)),
                );
                toolbar_user_interface.label(
                    RichText::new(format!(
                        "{} rooted symbol(s), {} symbol type(s)",
                        project_symbol_catalog.get_rooted_symbols().len(),
                        project_symbol_catalog.get_struct_layout_descriptors().len()
                    ))
                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                    .color(theme.foreground),
                );
                toolbar_user_interface.add_space(10.0);
                if toolbar_user_interface.button("New Rooted Symbol").clicked() {
                    SymbolExplorerViewData::begin_create_rooted_symbol(self.symbol_explorer_view_data.clone(), &project_symbol_catalog);
                }

                let content_rect = user_interface.available_rect_before_wrap();
                let details_panel_width = (content_rect.width() * Self::DETAILS_PANEL_WIDTH_RATIO).clamp(220.0, content_rect.width() - 140.0);
                let list_rect = Rect::from_min_max(content_rect.min, pos2(content_rect.max.x - details_panel_width, content_rect.max.y));
                let details_rect = Rect::from_min_max(pos2(list_rect.max.x, content_rect.min.y), content_rect.max);

                user_interface.painter().line_segment(
                    [
                        pos2(list_rect.max.x, list_rect.min.y),
                        pos2(list_rect.max.x, list_rect.max.y),
                    ],
                    Stroke::new(1.0, theme.submenu_border),
                );

                let mut list_user_interface = user_interface.new_child(
                    UiBuilder::new()
                        .max_rect(list_rect.shrink2(vec2(10.0, 8.0)))
                        .layout(Layout::top_down(Align::Min)),
                );
                ScrollArea::vertical()
                    .id_salt("symbol_explorer_list")
                    .auto_shrink([false, false])
                    .show(&mut list_user_interface, |user_interface| {
                        self.render_symbol_tree_list(user_interface, &symbol_tree_entries, selected_entry.as_ref());
                        self.render_struct_layout_list(user_interface, &project_symbol_catalog, selected_entry.as_ref());

                        if project_symbol_catalog.is_empty() {
                            user_interface.add_space(12.0);
                            user_interface.label(
                                RichText::new("This project has no authored symbols yet.")
                                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                                    .color(theme.foreground_preview),
                            );
                        }
                    });

                let mut details_user_interface = user_interface.new_child(
                    UiBuilder::new()
                        .max_rect(details_rect.shrink2(vec2(12.0, 8.0)))
                        .layout(Layout::top_down(Align::Min)),
                );

                match selected_entry.as_ref() {
                    Some(SymbolExplorerSelection::RootedSymbol(selected_symbol_key)) => {
                        if let Some(rooted_symbol) = project_symbol_catalog
                            .get_rooted_symbols()
                            .iter()
                            .find(|rooted_symbol| rooted_symbol.get_symbol_key() == selected_symbol_key)
                        {
                            self.render_rooted_symbol_details(&mut details_user_interface, rooted_symbol);
                        } else {
                            details_user_interface.label("Selected rooted symbol no longer exists.");
                        }
                    }
                    Some(SymbolExplorerSelection::DerivedNode(selected_node_key)) => {
                        if let Some(symbol_tree_entry) = symbol_tree_entries
                            .iter()
                            .find(|symbol_tree_entry| symbol_tree_entry.get_node_key() == selected_node_key)
                        {
                            self.render_derived_symbol_details(
                                &mut details_user_interface,
                                symbol_tree_entry,
                                Self::get_resolved_pointer_target_for_node_key(&resolved_pointer_targets_by_node_key, symbol_tree_entry.get_node_key()),
                            );
                        } else {
                            details_user_interface.label("Selected derived symbol node no longer exists.");
                        }
                    }
                    Some(SymbolExplorerSelection::StructLayout(struct_layout_id)) => {
                        self.render_struct_layout_details(&mut details_user_interface, &project_symbol_catalog, struct_layout_id);
                    }
                    Some(SymbolExplorerSelection::CreateRootedSymbol) => {
                        self.render_create_rooted_symbol_details(&mut details_user_interface, &project_symbol_catalog);
                    }
                    None => {
                        details_user_interface.label("Select a rooted symbol or symbol type.");
                    }
                }
            })
            .response
    }
}
