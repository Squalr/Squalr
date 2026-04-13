use crate::app_context::AppContext;
use crate::ui::{
    draw::icon_draw::IconDraw,
    widgets::controls::{button::Button as ThemeButton, data_value_box::data_value_box_view::DataValueBoxView, groupbox::GroupBox},
};
use crate::views::{
    code_viewer::{code_viewer_view::CodeViewerView, view_data::code_viewer_view_data::CodeViewerViewData},
    memory_viewer::{memory_viewer_view::MemoryViewerView, view_data::memory_viewer_view_data::MemoryViewerViewData},
    symbol_explorer::symbol_tree_entry_view::SymbolTreeEntryView,
    symbol_explorer::view_data::{
        symbol_explorer_view_data::{RootedSymbolDraftLocatorMode, SymbolExplorerSelection, SymbolExplorerTakeOverState, SymbolExplorerViewData},
        symbol_tree_entry::{ResolvedPointerTarget, SymbolTreeEntry, SymbolTreeEntryKind, build_symbol_tree_entries},
    },
};
use eframe::egui::{Align, Color32, Direction, Layout, Response, RichText, ScrollArea, Sense, TextEdit, Ui, UiBuilder, Widget, vec2};
use epaint::{CornerRadius, Rect, Stroke, TextureHandle, pos2};
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
use squalr_engine_api::structures::data_values::{
    anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType,
};
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
    const TOOLBAR_HEIGHT: f32 = 28.0;
    const TOOLBAR_ICON_BUTTON_SIZE: f32 = 36.0;
    const DISPLAY_NAME_DATA_VALUE_BOX_ID: &'static str = "symbol_explorer_display_name";
    const CREATE_DISPLAY_NAME_DATA_VALUE_BOX_ID: &'static str = "symbol_explorer_create_display_name";
    const STRING_DATA_TYPE_ID: &'static str = "string_utf8";

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
        SymbolExplorerViewData::cancel_take_over_state(self.symbol_explorer_view_data.clone());
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

    fn draw_text_button(
        &self,
        user_interface: &mut Ui,
        label: &str,
        fill_color: Color32,
        enabled: bool,
        minimum_width: f32,
    ) -> Response {
        let theme = &self.app_context.theme;
        let text_color = if enabled { theme.foreground } else { theme.foreground_preview };
        let label_galley = user_interface
            .painter()
            .layout_no_wrap(label.to_string(), theme.font_library.font_noto_sans.font_normal.clone(), text_color);
        let desired_width = (label_galley.size().x + 18.0).max(minimum_width);
        let button_response = user_interface.add_sized(
            [desired_width, 28.0],
            ThemeButton::new_from_theme(theme)
                .disabled(!enabled)
                .background_color(fill_color),
        );
        let text_position = pos2(
            button_response.rect.center().x - label_galley.size().x * 0.5,
            button_response.rect.center().y - label_galley.size().y * 0.5,
        );

        user_interface
            .painter()
            .galley(text_position, label_galley, text_color);

        button_response
    }

    fn draw_toggle_button(
        &self,
        user_interface: &mut Ui,
        label: &str,
        is_selected: bool,
    ) -> Response {
        let theme = &self.app_context.theme;

        self.draw_text_button(
            user_interface,
            label,
            if is_selected {
                theme.background_control_primary
            } else {
                theme.background_control_secondary
            },
            true,
            116.0,
        )
    }

    fn draw_icon_button(
        &self,
        user_interface: &mut Ui,
        icon_handle: &TextureHandle,
        tooltip_text: &str,
        enabled: bool,
        background_color: Color32,
    ) -> Response {
        let theme = &self.app_context.theme;
        let button_response = user_interface.add_sized(
            vec2(Self::TOOLBAR_ICON_BUTTON_SIZE, Self::TOOLBAR_HEIGHT),
            ThemeButton::new_from_theme(theme)
                .disabled(!enabled)
                .background_color(background_color)
                .with_tooltip_text(tooltip_text),
        );

        IconDraw::draw_tinted(
            user_interface,
            button_response.rect,
            icon_handle,
            if enabled { theme.foreground } else { theme.foreground_preview },
        );

        button_response
    }

    fn render_string_data_value_box(
        &self,
        user_interface: &mut Ui,
        value_text: &mut String,
        preview_text: &str,
        id: &str,
        width: f32,
    ) {
        let mut anonymous_value_string = AnonymousValueString::new(value_text.clone(), AnonymousValueStringFormat::String, ContainerType::None);
        let string_data_type = DataTypeRef::new(Self::STRING_DATA_TYPE_ID);

        user_interface.add(
            DataValueBoxView::new(
                self.app_context.clone(),
                &mut anonymous_value_string,
                &string_data_type,
                false,
                true,
                preview_text,
                id,
            )
            .width(width.max(1.0))
            .height(Self::TOOLBAR_HEIGHT)
            .show_format_button(false)
            .use_format_text_colors(false),
        );

        let next_value_text = anonymous_value_string.get_anonymous_value_string().to_string();
        if *value_text != next_value_text {
            *value_text = next_value_text;
        }
    }

    fn render_delete_confirmation_details(
        &self,
        user_interface: &mut Ui,
        display_name: &str,
    ) {
        let details_width = user_interface.available_width().max(1.0);

        user_interface.add(
            GroupBox::new_from_theme(&self.app_context.theme, "Delete Rooted Symbol", |user_interface| {
                user_interface.label(
                    RichText::new("Confirm deletion of this rooted symbol.")
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
                user_interface.add_space(8.0);
                user_interface.label(
                    RichText::new(display_name)
                        .font(
                            self.app_context
                                .theme
                                .font_library
                                .font_ubuntu_mono_bold
                                .font_normal
                                .clone(),
                        )
                        .color(self.app_context.theme.foreground),
                );
                user_interface.add_space(6.0);
                user_interface.label(RichText::new("Use the toolbar actions above to confirm or cancel.").color(self.app_context.theme.foreground_preview));
            })
            .desired_width(details_width),
        );
    }

    #[allow(dead_code)]
    fn render_symbol_tree_list_legacy(
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

                    if self
                        .draw_text_button(user_interface, expansion_label, self.app_context.theme.background_control_secondary, true, 24.0)
                        .clicked()
                    {
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
                let response = user_interface.selectable_label(is_selected, RichText::new(row_label).color(self.app_context.theme.foreground));

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
            let symbol_tree_entry_view_response = SymbolTreeEntryView::new(self.app_context.clone(), symbol_tree_entry, is_selected).show(user_interface);

            if symbol_tree_entry_view_response.did_click_expand_arrow {
                SymbolExplorerViewData::toggle_tree_node_expansion(self.symbol_explorer_view_data.clone(), symbol_tree_entry.get_node_key());
            }

            if symbol_tree_entry_view_response.did_click_row {
                let selection = match symbol_tree_entry.get_kind() {
                    SymbolTreeEntryKind::RootedSymbol { symbol_key } => SymbolExplorerSelection::RootedSymbol(symbol_key.to_string()),
                    SymbolTreeEntryKind::StructField | SymbolTreeEntryKind::ArrayElement | SymbolTreeEntryKind::PointerTarget => {
                        SymbolExplorerSelection::DerivedNode(symbol_tree_entry.get_node_key().to_string())
                    }
                };

                SymbolExplorerViewData::set_selected_entry(self.symbol_explorer_view_data.clone(), Some(selection));
            }
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
            let response = user_interface.selectable_label(
                is_selected,
                RichText::new(struct_layout_descriptor.get_struct_layout_id()).color(self.app_context.theme.foreground),
            );

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
        let details_width = user_interface.available_width().max(1.0);

        user_interface.add(
            GroupBox::new_from_theme(&self.app_context.theme, "Rooted Symbol", |user_interface| {
                user_interface.label(RichText::new("Display Name").color(self.app_context.theme.foreground));
                user_interface.horizontal(|user_interface| {
                    let check_button_width = Self::TOOLBAR_ICON_BUTTON_SIZE;
                    let value_box_width = (user_interface.available_width() - check_button_width - 6.0).max(1.0);

                    self.render_string_data_value_box(
                        user_interface,
                        &mut edited_display_name_draft,
                        "Symbol name",
                        Self::DISPLAY_NAME_DATA_VALUE_BOX_ID,
                        value_box_width,
                    );
                    let trimmed_display_name_draft = edited_display_name_draft.trim().to_string();
                    let can_apply_rename = !trimmed_display_name_draft.is_empty() && trimmed_display_name_draft != rooted_symbol.get_display_name();

                    if self
                        .draw_icon_button(
                            user_interface,
                            &self
                                .app_context
                                .theme
                                .icon_library
                                .icon_handle_common_check_mark,
                            "Apply rooted-symbol display name.",
                            can_apply_rename,
                            self.app_context.theme.background_control_secondary,
                        )
                        .clicked()
                    {
                        self.rename_rooted_symbol(rooted_symbol.get_symbol_key(), trimmed_display_name_draft.clone());
                    }
                });

                if edited_display_name_draft != current_display_name_draft {
                    SymbolExplorerViewData::set_rooted_symbol_display_name_draft(self.symbol_explorer_view_data.clone(), edited_display_name_draft.clone());
                }

                user_interface.add_space(8.0);
                user_interface.label(
                    RichText::new(format!("key: {}", rooted_symbol.get_symbol_key()))
                        .monospace()
                        .color(self.app_context.theme.foreground_preview),
                );
                user_interface.label(
                    RichText::new(format!("type: {}", rooted_symbol.get_struct_layout_id()))
                        .monospace()
                        .color(self.app_context.theme.foreground_preview),
                );
                user_interface.label(
                    RichText::new(format!("locator: {}", rooted_symbol.get_root_locator()))
                        .monospace()
                        .color(self.app_context.theme.foreground_preview),
                );
            })
            .desired_width(details_width),
        );

        user_interface.add_space(12.0);
        user_interface.add(
            GroupBox::new_from_theme(&self.app_context.theme, "Metadata", |user_interface| {
                if rooted_symbol.get_metadata().is_empty() {
                    user_interface.label(RichText::new("No metadata.").color(self.app_context.theme.foreground_preview));
                    return;
                }

                for (metadata_key, metadata_value) in rooted_symbol.get_metadata() {
                    user_interface.label(
                        RichText::new(format!("{} = {}", metadata_key, metadata_value))
                            .monospace()
                            .color(self.app_context.theme.foreground_preview),
                    );
                }
            })
            .desired_width(details_width),
        );
    }

    fn render_derived_symbol_details(
        &self,
        user_interface: &mut Ui,
        symbol_tree_entry: &SymbolTreeEntry,
        resolved_pointer_target: Option<ResolvedPointerTarget>,
    ) {
        let details_width = user_interface.available_width().max(1.0);

        user_interface.add(
            GroupBox::new_from_theme(&self.app_context.theme, "Derived Symbol", |user_interface| {
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
                user_interface.label(
                    RichText::new(format!("path: {}", symbol_tree_entry.get_full_path()))
                        .monospace()
                        .color(self.app_context.theme.foreground_preview),
                );
                user_interface.label(
                    RichText::new(format!(
                        "type: {}{}",
                        symbol_tree_entry.get_symbol_type_id(),
                        symbol_tree_entry.get_container_type()
                    ))
                    .monospace()
                    .color(self.app_context.theme.foreground_preview),
                );
                user_interface.label(
                    RichText::new(format!("locator: {}", symbol_tree_entry.get_locator()))
                        .monospace()
                        .color(self.app_context.theme.foreground_preview),
                );

                if let Some(resolved_pointer_target) = resolved_pointer_target {
                    user_interface.label(
                        RichText::new(format!("resolved target: {}", resolved_pointer_target.get_target_locator()))
                            .monospace()
                            .color(self.app_context.theme.foreground_preview),
                    );

                    if !resolved_pointer_target.get_evaluated_pointer_path().is_empty() {
                        user_interface.label(
                            RichText::new(format!("pointer path: {}", resolved_pointer_target.get_evaluated_pointer_path()))
                                .monospace()
                                .color(self.app_context.theme.foreground_preview),
                        );
                    }
                }
            })
            .desired_width(details_width),
        );
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
        let details_width = user_interface.available_width().max(1.0);

        user_interface.add(
            GroupBox::new_from_theme(&self.app_context.theme, "Symbol Type", |user_interface| {
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
                user_interface.label(
                    RichText::new(format!(
                        "{} field(s)",
                        struct_layout_descriptor
                            .get_struct_layout_definition()
                            .get_fields()
                            .len()
                    ))
                    .monospace()
                    .color(self.app_context.theme.foreground_preview),
                );
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
            })
            .desired_width(details_width),
        );
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
        let details_width = user_interface.available_width().max(1.0);

        user_interface.add(
            GroupBox::new_from_theme(&self.app_context.theme, "New Rooted Symbol", |user_interface| {
                user_interface.label(RichText::new("Display Name").color(self.app_context.theme.foreground));
                self.render_string_data_value_box(
                    user_interface,
                    &mut edited_draft.display_name,
                    "Symbol name",
                    Self::CREATE_DISPLAY_NAME_DATA_VALUE_BOX_ID,
                    user_interface.available_width(),
                );
                user_interface.add_space(6.0);

                user_interface.label(RichText::new("Type Id").color(self.app_context.theme.foreground));
                user_interface.add(TextEdit::singleline(&mut edited_draft.struct_layout_id));
                user_interface.add_space(6.0);

                user_interface.label(RichText::new("Locator").color(self.app_context.theme.foreground));
                user_interface.horizontal_wrapped(|user_interface| {
                    let is_absolute_address = matches!(edited_draft.locator_mode, RootedSymbolDraftLocatorMode::AbsoluteAddress);

                    if self
                        .draw_toggle_button(user_interface, "Absolute Address", is_absolute_address)
                        .clicked()
                    {
                        edited_draft.locator_mode = RootedSymbolDraftLocatorMode::AbsoluteAddress;
                    }

                    if self
                        .draw_toggle_button(user_interface, "Module + Offset", !is_absolute_address)
                        .clicked()
                    {
                        edited_draft.locator_mode = RootedSymbolDraftLocatorMode::ModuleOffset;
                    }
                });
                user_interface.add_space(6.0);

                match edited_draft.locator_mode {
                    RootedSymbolDraftLocatorMode::AbsoluteAddress => {
                        user_interface.label(RichText::new("Address").color(self.app_context.theme.foreground));
                        user_interface.add(TextEdit::singleline(&mut edited_draft.address_text).hint_text("0x12345678 or 305419896"));
                    }
                    RootedSymbolDraftLocatorMode::ModuleOffset => {
                        user_interface.label(RichText::new("Module").color(self.app_context.theme.foreground));
                        user_interface.add(TextEdit::singleline(&mut edited_draft.module_name));
                        user_interface.add_space(6.0);
                        user_interface.label(RichText::new("Offset").color(self.app_context.theme.foreground));
                        user_interface.add(TextEdit::singleline(&mut edited_draft.offset_text).hint_text("0x1234 or 4660"));
                    }
                }
            })
            .desired_width(details_width),
        );

        if edited_draft != original_draft {
            SymbolExplorerViewData::set_rooted_symbol_create_draft(self.symbol_explorer_view_data.clone(), edited_draft.clone());
        }

        if !project_symbol_catalog
            .get_struct_layout_descriptors()
            .is_empty()
        {
            user_interface.add_space(12.0);
            user_interface.add(
                GroupBox::new_from_theme(&self.app_context.theme, "Available Types", |user_interface| {
                    for struct_layout_descriptor in project_symbol_catalog.get_struct_layout_descriptors() {
                        user_interface.label(
                            RichText::new(struct_layout_descriptor.get_struct_layout_id())
                                .monospace()
                                .color(self.app_context.theme.foreground_preview),
                        );
                    }
                })
                .desired_width(details_width),
            );
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

    fn build_rooted_symbol_create_request_from_draft(
        edited_draft: &crate::views::symbol_explorer::view_data::symbol_explorer_view_data::RootedSymbolCreateDraft,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) -> Option<ProjectSymbolsCreateRequest> {
        let parsed_address = Self::parse_u64_draft(&edited_draft.address_text);
        let parsed_offset = Self::parse_u64_draft(&edited_draft.offset_text);
        let has_valid_type_id = !edited_draft.struct_layout_id.trim().is_empty()
            && project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == edited_draft.struct_layout_id.trim());
        let has_valid_locator = match edited_draft.locator_mode {
            RootedSymbolDraftLocatorMode::AbsoluteAddress => parsed_address.is_some(),
            RootedSymbolDraftLocatorMode::ModuleOffset => !edited_draft.module_name.trim().is_empty() && parsed_offset.is_some(),
        };

        if edited_draft.display_name.trim().is_empty() || !has_valid_type_id || !has_valid_locator {
            return None;
        }

        Some(ProjectSymbolsCreateRequest {
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
        })
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
                        user_interface
                            .label(RichText::new("Open a project to browse symbol types and rooted symbols.").color(self.app_context.theme.foreground_preview));
                    },
                )
                .response;
        };

        SymbolExplorerViewData::synchronize_selection(self.symbol_explorer_view_data.clone(), &project_symbol_catalog);
        SymbolExplorerViewData::synchronize_rooted_symbol_display_name_draft(self.symbol_explorer_view_data.clone(), &project_symbol_catalog);
        SymbolExplorerViewData::synchronize_rooted_symbol_create_draft(self.symbol_explorer_view_data.clone(), &project_symbol_catalog);
        SymbolExplorerViewData::synchronize_take_over_state(self.symbol_explorer_view_data.clone(), &project_symbol_catalog);
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
        let (selected_entry, take_over_state, current_create_rooted_symbol_draft) = self
            .symbol_explorer_view_data
            .read("Symbol explorer view")
            .map(|symbol_explorer_view_data| {
                (
                    symbol_explorer_view_data.get_selected_entry().cloned(),
                    symbol_explorer_view_data.get_take_over_state().cloned(),
                    symbol_explorer_view_data
                        .get_rooted_symbol_create_draft()
                        .clone(),
                )
            })
            .unwrap_or_default();
        let selected_rooted_symbol = match selected_entry.as_ref() {
            Some(SymbolExplorerSelection::RootedSymbol(selected_symbol_key)) => project_symbol_catalog
                .get_rooted_symbols()
                .iter()
                .find(|rooted_symbol| rooted_symbol.get_symbol_key() == selected_symbol_key),
            _ => None,
        };
        let create_rooted_symbol_request = match selected_entry.as_ref() {
            Some(SymbolExplorerSelection::CreateRootedSymbol) => {
                Self::build_rooted_symbol_create_request_from_draft(&current_create_rooted_symbol_draft, &project_symbol_catalog)
            }
            _ => None,
        };
        let theme = self.app_context.theme.clone();

        user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
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
                let (list_toolbar_rect, _list_toolbar_response) =
                    list_user_interface.allocate_exact_size(vec2(list_user_interface.available_width().max(1.0), Self::TOOLBAR_HEIGHT), Sense::empty());
                list_user_interface
                    .painter()
                    .rect_filled(list_toolbar_rect, CornerRadius::ZERO, theme.background_primary);

                let mut toolbar_user_interface = list_user_interface.new_child(
                    UiBuilder::new()
                        .max_rect(list_toolbar_rect)
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
                toolbar_user_interface.with_layout(Layout::right_to_left(Align::Center), |toolbar_user_interface| {
                    if let Some(SymbolExplorerTakeOverState::DeleteConfirmation { symbol_key, .. }) = take_over_state.as_ref() {
                        if self
                            .draw_icon_button(
                                toolbar_user_interface,
                                &theme.icon_library.icon_handle_common_delete,
                                "Delete rooted symbol.",
                                true,
                                theme.background_control_secondary,
                            )
                            .clicked()
                        {
                            self.delete_rooted_symbol(symbol_key);
                        }

                        if self
                            .draw_icon_button(
                                toolbar_user_interface,
                                &theme.icon_library.icon_handle_navigation_cancel,
                                "Cancel deletion.",
                                true,
                                theme.background_control_secondary,
                            )
                            .clicked()
                        {
                            SymbolExplorerViewData::cancel_take_over_state(self.symbol_explorer_view_data.clone());
                        }

                        return;
                    }

                    if self
                        .draw_icon_button(
                            toolbar_user_interface,
                            &theme.icon_library.icon_handle_common_add,
                            "Create a new rooted symbol.",
                            true,
                            theme.background_control_secondary,
                        )
                        .clicked()
                    {
                        SymbolExplorerViewData::begin_create_rooted_symbol(self.symbol_explorer_view_data.clone(), &project_symbol_catalog);
                    }

                    match selected_entry.as_ref() {
                        Some(SymbolExplorerSelection::RootedSymbol(_)) => {
                            if self
                                .draw_icon_button(
                                    toolbar_user_interface,
                                    &theme.icon_library.icon_handle_common_delete,
                                    "Delete selected rooted symbol.",
                                    selected_rooted_symbol.is_some(),
                                    theme.background_control_secondary,
                                )
                                .clicked()
                            {
                                if let Some(rooted_symbol) = selected_rooted_symbol {
                                    SymbolExplorerViewData::request_delete_confirmation(
                                        self.symbol_explorer_view_data.clone(),
                                        rooted_symbol.get_symbol_key().to_string(),
                                        rooted_symbol.get_display_name().to_string(),
                                    );
                                }
                            }

                            if self
                                .draw_icon_button(
                                    toolbar_user_interface,
                                    &theme.icon_library.icon_handle_project_cpu_instruction,
                                    "Open selected rooted symbol in Code Viewer.",
                                    selected_rooted_symbol.is_some(),
                                    theme.background_control_secondary,
                                )
                                .clicked()
                            {
                                if let Some(rooted_symbol) = selected_rooted_symbol {
                                    self.focus_code_viewer_for_locator(rooted_symbol.get_root_locator());
                                }
                            }

                            if self
                                .draw_icon_button(
                                    toolbar_user_interface,
                                    &theme.icon_library.icon_handle_scan_collect_values,
                                    "Open selected rooted symbol in Memory Viewer.",
                                    selected_rooted_symbol.is_some(),
                                    theme.background_control_secondary,
                                )
                                .clicked()
                            {
                                if let Some(rooted_symbol) = selected_rooted_symbol {
                                    self.focus_memory_viewer_for_locator(rooted_symbol.get_root_locator());
                                }
                            }
                        }
                        Some(SymbolExplorerSelection::DerivedNode(selected_node_key)) => {
                            if let Some(symbol_tree_entry) = symbol_tree_entries
                                .iter()
                                .find(|symbol_tree_entry| symbol_tree_entry.get_node_key() == selected_node_key)
                            {
                                if self
                                    .draw_icon_button(
                                        toolbar_user_interface,
                                        &theme.icon_library.icon_handle_project_cpu_instruction,
                                        "Open selected derived symbol in Code Viewer.",
                                        true,
                                        theme.background_control_secondary,
                                    )
                                    .clicked()
                                {
                                    self.focus_code_viewer_for_locator(symbol_tree_entry.get_locator());
                                }

                                if self
                                    .draw_icon_button(
                                        toolbar_user_interface,
                                        &theme.icon_library.icon_handle_scan_collect_values,
                                        "Open selected derived symbol in Memory Viewer.",
                                        true,
                                        theme.background_control_secondary,
                                    )
                                    .clicked()
                                {
                                    self.focus_memory_viewer_for_locator(symbol_tree_entry.get_locator());
                                }

                                if self
                                    .draw_icon_button(
                                        toolbar_user_interface,
                                        &theme.icon_library.icon_handle_common_add,
                                        "Promote selected derived symbol to a rooted symbol.",
                                        true,
                                        theme.background_control_secondary,
                                    )
                                    .clicked()
                                {
                                    self.create_rooted_symbol_from_locator(
                                        symbol_tree_entry.get_promotion_display_name().to_string(),
                                        symbol_tree_entry.get_promoted_symbol_type_id(),
                                        symbol_tree_entry.get_locator().clone(),
                                    );
                                }
                            }
                        }
                        Some(SymbolExplorerSelection::CreateRootedSymbol) => {
                            if self
                                .draw_icon_button(
                                    toolbar_user_interface,
                                    &theme.icon_library.icon_handle_navigation_cancel,
                                    "Cancel rooted-symbol creation.",
                                    true,
                                    theme.background_control_secondary,
                                )
                                .clicked()
                            {
                                SymbolExplorerViewData::set_selected_entry(self.symbol_explorer_view_data.clone(), None);
                            }

                            if self
                                .draw_icon_button(
                                    toolbar_user_interface,
                                    &theme.icon_library.icon_handle_common_check_mark,
                                    "Create rooted symbol.",
                                    create_rooted_symbol_request.is_some(),
                                    theme.background_control_secondary,
                                )
                                .clicked()
                            {
                                if let Some(project_symbols_create_request) = create_rooted_symbol_request.clone() {
                                    self.create_rooted_symbol(project_symbols_create_request);
                                }
                            }
                        }
                        _ => {}
                    }
                });

                list_user_interface.add_space(8.0);
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

                ScrollArea::vertical()
                    .id_salt("symbol_explorer_details")
                    .auto_shrink([false, false])
                    .show(&mut details_user_interface, |details_user_interface| {
                        if let Some(SymbolExplorerTakeOverState::DeleteConfirmation { display_name, .. }) = take_over_state.as_ref() {
                            self.render_delete_confirmation_details(details_user_interface, display_name);
                            return;
                        }

                        match selected_entry.as_ref() {
                            Some(SymbolExplorerSelection::RootedSymbol(selected_symbol_key)) => {
                                if let Some(rooted_symbol) = project_symbol_catalog
                                    .get_rooted_symbols()
                                    .iter()
                                    .find(|rooted_symbol| rooted_symbol.get_symbol_key() == selected_symbol_key)
                                {
                                    self.render_rooted_symbol_details(details_user_interface, rooted_symbol);
                                } else {
                                    details_user_interface.label(RichText::new("Selected rooted symbol no longer exists.").color(theme.foreground_preview));
                                }
                            }
                            Some(SymbolExplorerSelection::DerivedNode(selected_node_key)) => {
                                if let Some(symbol_tree_entry) = symbol_tree_entries
                                    .iter()
                                    .find(|symbol_tree_entry| symbol_tree_entry.get_node_key() == selected_node_key)
                                {
                                    self.render_derived_symbol_details(
                                        details_user_interface,
                                        symbol_tree_entry,
                                        Self::get_resolved_pointer_target_for_node_key(&resolved_pointer_targets_by_node_key, symbol_tree_entry.get_node_key()),
                                    );
                                } else {
                                    details_user_interface
                                        .label(RichText::new("Selected derived symbol node no longer exists.").color(theme.foreground_preview));
                                }
                            }
                            Some(SymbolExplorerSelection::StructLayout(struct_layout_id)) => {
                                self.render_struct_layout_details(details_user_interface, &project_symbol_catalog, struct_layout_id);
                            }
                            Some(SymbolExplorerSelection::CreateRootedSymbol) => {
                                self.render_create_rooted_symbol_details(details_user_interface, &project_symbol_catalog);
                            }
                            None => {
                                details_user_interface.label(RichText::new("Select a rooted symbol or symbol type.").color(theme.foreground_preview));
                            }
                        }
                    });
            })
            .response
    }
}
