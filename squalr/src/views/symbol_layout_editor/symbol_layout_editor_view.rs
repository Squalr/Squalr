mod authoring;
mod controls;
mod details;
mod list;
mod rows;
mod takeovers;
mod toolbars;

use crate::app_context::AppContext;
use crate::ui::list_navigation::ListNavigationDirection;
use crate::views::struct_viewer::view_data::struct_viewer_view_data::StructViewerViewData;
use crate::views::symbol_layout_editor::symbol_layout_command_dispatcher::SymbolLayoutCommandDispatcher;
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::{
    SymbolLayoutEditDraft, SymbolLayoutEditorTakeOverState, SymbolLayoutEditorViewData,
};
use authoring::symbol_layout_field_draft_factory::SymbolLayoutFieldDraftFactory;
use details::symbol_layout_details_focus::{clear_struct_viewer_if_symbol_layout_focused, focus_selected_layout_in_struct_viewer};
use eframe::egui::{Align, Direction, Key, Layout, RichText, Ui, Widget};
use list::symbol_layout_list_panel_view::SymbolLayoutListPanelView;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use std::{collections::BTreeSet, sync::Arc};

#[derive(Clone)]
pub struct SymbolLayoutEditorView {
    app_context: Arc<AppContext>,
    symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
}

impl SymbolLayoutEditorView {
    pub const WINDOW_ID: &'static str = "window_symbol_layout_editor";
    const TOOLBAR_HEIGHT: f32 = 28.0;
    const FIELD_ROW_HEIGHT: f32 = 28.0;
    const LIST_ROW_HEIGHT: f32 = 28.0;
    const ICON_BUTTON_WIDTH: f32 = 36.0;
    const FIELD_INPUT_SPACING: f32 = 8.0;
    const TAKE_OVER_HEADER_HEIGHT: f32 = 32.0;
    const TAKE_OVER_PADDING_X: f32 = 0.0;
    const TAKE_OVER_PADDING_Y: f32 = 0.0;
    const TAKE_OVER_CONTENT_PADDING_X: f32 = 12.0;
    const TAKE_OVER_HEADER_TITLE_PADDING_X: f32 = 8.0;
    const TAKE_OVER_SECTION_SPACING: f32 = 12.0;
    const TAKE_OVER_GROUPBOX_SPACING: f32 = 8.0;
    const TAKE_OVER_GROUPBOX_SIDE_PADDING: f32 = 8.0;
    const TAKE_OVER_BOTTOM_PADDING: f32 = 8.0;
    const TAKE_OVER_ACTION_BUTTON_WIDTH: f32 = 120.0;
    const TAKE_OVER_ACTION_BUTTON_SPACING: f32 = 12.0;
    const FIELD_ADD_BUTTON_CORNER_RADIUS: u8 = 8;
    const FIELD_ROW_LEFT_PADDING: f32 = 8.0;
    const FIELD_ROW_ICON_SIZE: f32 = 16.0;
    const FIELD_ROW_ICON_GAP: f32 = 4.0;
    const FIELD_ROW_PREVIEW_GAP: f32 = 12.0;
    const FIELD_CONTEXT_MENU_WIDTH: f32 = 184.0;
    const UNION_VARIANT_CHILD_INDENT: f32 = 20.0;
    const DEFINE_FIELD_CONTAINER_SELECTOR_WIDTH: f32 = 118.0;
    const DEFINE_FIELD_GROUPBOX_SIDE_PADDING: f32 = 8.0;
    const LAYOUT_KIND_COMBO_WIDTH: f32 = 128.0;

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let symbol_layout_editor_view_data = app_context
            .dependency_container
            .register(SymbolLayoutEditorViewData::new());
        let struct_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<StructViewerViewData>();

        Self {
            app_context,
            symbol_layout_editor_view_data,
            struct_viewer_view_data,
        }
    }

    fn get_opened_project_symbol_catalog(&self) -> Option<ProjectSymbolCatalog> {
        Self::get_opened_project_symbol_catalog_from_context(&self.app_context)
    }

    fn get_opened_project_symbol_catalog_from_context(app_context: &AppContext) -> Option<ProjectSymbolCatalog> {
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

    fn command_dispatcher(&self) -> SymbolLayoutCommandDispatcher {
        SymbolLayoutCommandDispatcher::new(self.app_context.clone())
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn resolve_data_type_size_in_bytes(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> Option<u64> {
        let size_in_bytes = self
            .app_context
            .engine_unprivileged_state
            .get_unit_size_in_bytes(data_type_ref);

        (size_in_bytes > 0).then_some(size_in_bytes)
    }

    fn symbol_layout_take_over_has_unsaved_changes(
        baseline_project_symbol_catalog: Option<&ProjectSymbolCatalog>,
        baseline_draft: &SymbolLayoutEditDraft,
        edited_draft: &SymbolLayoutEditDraft,
        edited_struct_layout_descriptor: Option<&squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor>,
        unassigned_split_offsets: &BTreeSet<u64>,
    ) -> bool {
        if let (Some(baseline_project_symbol_catalog), Some(original_layout_id), Some(edited_struct_layout_descriptor)) = (
            baseline_project_symbol_catalog,
            edited_draft.original_layout_id.as_deref(),
            edited_struct_layout_descriptor,
        ) && let Some(baseline_struct_layout_descriptor) = baseline_project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == original_layout_id)
        {
            return edited_struct_layout_descriptor.get_struct_layout_id() != baseline_struct_layout_descriptor.get_struct_layout_id()
                || edited_struct_layout_descriptor.get_struct_layout_definition() != baseline_struct_layout_descriptor.get_struct_layout_definition();
        }

        edited_draft != baseline_draft || !unassigned_split_offsets.is_empty()
    }
}

impl Widget for SymbolLayoutEditorView {
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
                            .label(RichText::new("Open a project to author reusable symbol layouts.").color(self.app_context.theme.foreground_preview));
                    },
                )
                .response;
        };

        SymbolLayoutEditorViewData::synchronize(self.symbol_layout_editor_view_data.clone(), &project_symbol_catalog);
        let (
            selected_layout_id,
            filter_text,
            take_over_state,
            baseline_project_symbol_catalog,
            baseline_draft,
            draft,
            unassigned_split_offsets,
            selected_field_index,
            selected_field_layout_id,
            selected_unassigned_span,
            define_field_draft,
        ) = self
            .symbol_layout_editor_view_data
            .read("SymbolLayoutEditor view")
            .map(|symbol_layout_editor_view_data| {
                (
                    symbol_layout_editor_view_data
                        .get_selected_layout_id()
                        .map(str::to_string),
                    symbol_layout_editor_view_data.get_filter_text().to_string(),
                    symbol_layout_editor_view_data.get_take_over_state().cloned(),
                    symbol_layout_editor_view_data
                        .get_baseline_project_symbol_catalog()
                        .cloned(),
                    symbol_layout_editor_view_data.get_baseline_draft().cloned(),
                    symbol_layout_editor_view_data.get_draft().cloned(),
                    symbol_layout_editor_view_data
                        .get_unassigned_split_offsets()
                        .clone(),
                    symbol_layout_editor_view_data.get_selected_field_index(),
                    symbol_layout_editor_view_data
                        .get_selected_field_layout_id()
                        .map(str::to_string),
                    symbol_layout_editor_view_data
                        .get_selected_unassigned_span()
                        .cloned(),
                    symbol_layout_editor_view_data.get_define_field_draft().cloned(),
                )
            })
            .unwrap_or((None, String::new(), None, None, None, None, BTreeSet::new(), None, None, None, None));
        let is_take_over_active = take_over_state.is_some();
        let is_window_focused = self
            .app_context
            .window_focus_manager
            .is_window_focused(Self::WINDOW_ID);
        let can_handle_window_shortcuts = self
            .app_context
            .window_focus_manager
            .can_window_handle_shortcuts(user_interface.ctx(), Self::WINDOW_ID);

        if is_window_focused && user_interface.input(|input_state| input_state.key_pressed(Key::Escape)) && is_take_over_active {
            if let Some(SymbolLayoutEditorTakeOverState::UnassignFieldConfirmation { return_state, .. }) = take_over_state.as_ref() {
                SymbolLayoutEditorViewData::return_to_define_field_source(self.symbol_layout_editor_view_data.clone(), return_state.clone());
            } else if let Some(SymbolLayoutEditorTakeOverState::DefineFieldFromUnassignedSpan { return_state, .. }) = take_over_state.as_ref() {
                SymbolLayoutEditorViewData::return_to_define_field_source(self.symbol_layout_editor_view_data.clone(), return_state.clone());
            } else {
                SymbolLayoutEditorViewData::cancel_take_over_state(self.symbol_layout_editor_view_data.clone());
                clear_struct_viewer_if_symbol_layout_focused(self.struct_viewer_view_data.clone());
            }
        }

        if !is_take_over_active && can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::Enter)) {
            if let Some(selected_layout_id) = selected_layout_id.as_deref() {
                SymbolLayoutEditorViewData::begin_open_symbol_layout(
                    self.symbol_layout_editor_view_data.clone(),
                    &project_symbol_catalog,
                    selected_layout_id,
                    |data_type_ref| self.resolve_data_type_size_in_bytes(data_type_ref),
                );
            }
        }

        if !is_take_over_active && can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::ArrowUp)) {
            let next_layout_id = SymbolLayoutEditorViewData::navigate_symbol_layout_selection(
                self.symbol_layout_editor_view_data.clone(),
                &project_symbol_catalog,
                ListNavigationDirection::Up,
            );
            focus_selected_layout_in_struct_viewer(
                self.app_context.clone(),
                self.struct_viewer_view_data.clone(),
                &project_symbol_catalog,
                next_layout_id.as_deref(),
            );
        }

        if !is_take_over_active && can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::ArrowDown)) {
            let next_layout_id = SymbolLayoutEditorViewData::navigate_symbol_layout_selection(
                self.symbol_layout_editor_view_data.clone(),
                &project_symbol_catalog,
                ListNavigationDirection::Down,
            );
            focus_selected_layout_in_struct_viewer(
                self.app_context.clone(),
                self.struct_viewer_view_data.clone(),
                &project_symbol_catalog,
                next_layout_id.as_deref(),
            );
        }

        if !is_take_over_active && can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::Delete)) {
            if let Some(selected_layout_id) = selected_layout_id.as_deref() {
                SymbolLayoutEditorViewData::request_delete_confirmation(self.symbol_layout_editor_view_data.clone(), selected_layout_id.to_string());
            }
        }

        user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                let content_rect = user_interface.available_rect_before_wrap();
                let mut content_user_interface = user_interface.new_child(
                    eframe::egui::UiBuilder::new()
                        .max_rect(content_rect)
                        .layout(Layout::top_down(Align::Min)),
                );
                match take_over_state.as_ref() {
                    Some(SymbolLayoutEditorTakeOverState::CreateSymbolLayout) => {
                        self.render_symbol_layout_take_over(
                            &mut content_user_interface,
                            &project_symbol_catalog,
                            "",
                            baseline_project_symbol_catalog.as_ref(),
                            baseline_draft.as_ref(),
                            draft.as_ref(),
                            &unassigned_split_offsets,
                            selected_field_index,
                            selected_field_layout_id.as_deref(),
                            selected_unassigned_span.as_ref(),
                            true,
                        );
                    }
                    Some(SymbolLayoutEditorTakeOverState::RenameSymbolLayout { .. }) => {
                        self.render_symbol_layout_take_over(
                            &mut content_user_interface,
                            &project_symbol_catalog,
                            "Rename Symbol Layout",
                            baseline_project_symbol_catalog.as_ref(),
                            baseline_draft.as_ref(),
                            draft.as_ref(),
                            &unassigned_split_offsets,
                            selected_field_index,
                            selected_field_layout_id.as_deref(),
                            selected_unassigned_span.as_ref(),
                            true,
                        );
                    }
                    Some(SymbolLayoutEditorTakeOverState::OpenSymbolLayout { .. }) => {
                        self.render_symbol_layout_take_over(
                            &mut content_user_interface,
                            &project_symbol_catalog,
                            "Edit Symbol Layout",
                            baseline_project_symbol_catalog.as_ref(),
                            baseline_draft.as_ref(),
                            draft.as_ref(),
                            &unassigned_split_offsets,
                            selected_field_index,
                            selected_field_layout_id.as_deref(),
                            selected_unassigned_span.as_ref(),
                            false,
                        );
                    }
                    Some(SymbolLayoutEditorTakeOverState::DefineFieldFromUnassignedSpan {
                        layout_id,
                        offset,
                        size,
                        return_state,
                    }) => {
                        self.render_define_field_from_unassigned_take_over(
                            &mut content_user_interface,
                            &project_symbol_catalog,
                            layout_id,
                            *offset,
                            *size,
                            return_state,
                            draft.as_ref(),
                            define_field_draft.as_ref(),
                        );
                    }
                    Some(SymbolLayoutEditorTakeOverState::DeleteConfirmation { layout_id }) => {
                        self.render_delete_confirmation_take_over(&mut content_user_interface, &project_symbol_catalog, layout_id);
                    }
                    Some(SymbolLayoutEditorTakeOverState::UnassignFieldConfirmation {
                        layout_id: _,
                        field_index,
                        return_state,
                    }) => {
                        self.render_field_unassign_confirmation_take_over(
                            &mut content_user_interface,
                            &project_symbol_catalog,
                            *field_index,
                            return_state,
                            draft.as_ref(),
                        );
                    }
                    None => {
                        SymbolLayoutListPanelView::new(
                            self.app_context.clone(),
                            self.symbol_layout_editor_view_data.clone(),
                            self.struct_viewer_view_data.clone(),
                            &project_symbol_catalog,
                            selected_layout_id.as_deref(),
                            &filter_text,
                            SymbolLayoutFieldDraftFactory::default_data_type_ref(&self.app_context),
                            false,
                        )
                        .show(&mut content_user_interface);
                    }
                }
            })
            .response
    }
}
