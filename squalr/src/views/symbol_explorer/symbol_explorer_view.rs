use crate::app_context::AppContext;
use crate::views::{
    code_viewer::{code_viewer_view::CodeViewerView, view_data::code_viewer_view_data::CodeViewerViewData},
    memory_viewer::{memory_viewer_view::MemoryViewerView, view_data::memory_viewer_view_data::MemoryViewerViewData},
    symbol_explorer::view_data::symbol_explorer_view_data::{SymbolExplorerSelection, SymbolExplorerViewData},
};
use eframe::egui::{Align, Direction, Layout, Response, RichText, ScrollArea, Sense, Ui, UiBuilder, Widget, vec2};
use epaint::{CornerRadius, Rect, Stroke, pos2};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::{
    project_root_symbol::ProjectRootSymbol, project_root_symbol_locator::ProjectRootSymbolLocator, project_symbol_catalog::ProjectSymbolCatalog,
};
use std::sync::Arc;

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

    fn render_rooted_symbol_list(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_entry: Option<&SymbolExplorerSelection>,
    ) {
        user_interface.label(
            RichText::new(format!("Rooted Symbols ({})", project_symbol_catalog.get_rooted_symbols().len()))
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

        for rooted_symbol in project_symbol_catalog.get_rooted_symbols() {
            let is_selected = matches!(
                selected_entry,
                Some(SymbolExplorerSelection::RootedSymbol(selected_symbol_key)) if selected_symbol_key == rooted_symbol.get_symbol_key()
            );
            let response = user_interface.selectable_label(
                is_selected,
                format!("{}  [{}]", rooted_symbol.get_display_name(), rooted_symbol.get_struct_layout_id()),
            );

            if response.clicked() {
                SymbolExplorerViewData::set_selected_entry(
                    self.symbol_explorer_view_data.clone(),
                    Some(SymbolExplorerSelection::RootedSymbol(rooted_symbol.get_symbol_key().to_string())),
                );
            }

            user_interface.label(
                RichText::new(rooted_symbol.get_root_locator().to_string())
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
        user_interface.label(
            RichText::new(rooted_symbol.get_display_name())
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
        user_interface.monospace(format!("key: {}", rooted_symbol.get_symbol_key()));
        user_interface.monospace(format!("type: {}", rooted_symbol.get_struct_layout_id()));
        user_interface.monospace(format!("locator: {}", rooted_symbol.get_root_locator()));
        user_interface.add_space(10.0);

        user_interface.horizontal(|user_interface| {
            if user_interface.button("Open In Memory").clicked() {
                self.focus_memory_viewer_for_locator(rooted_symbol.get_root_locator());
            }

            if user_interface.button("Open In Code").clicked() {
                self.focus_code_viewer_for_locator(rooted_symbol.get_root_locator());
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
}

impl Widget for SymbolExplorerView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let Some(project_symbol_catalog) = self.get_opened_project_symbol_catalog() else {
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
                        self.render_rooted_symbol_list(user_interface, &project_symbol_catalog, selected_entry.as_ref());
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
                    Some(SymbolExplorerSelection::StructLayout(struct_layout_id)) => {
                        self.render_struct_layout_details(&mut details_user_interface, &project_symbol_catalog, struct_layout_id);
                    }
                    None => {
                        details_user_interface.label("Select a rooted symbol or symbol type.");
                    }
                }
            })
            .response
    }
}
