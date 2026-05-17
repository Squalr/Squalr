use super::super::SymbolLayoutEditorView;
use super::super::details::symbol_layout_details_focus::focus_selected_layout_in_struct_viewer;
use super::super::rows::symbol_layout_row_view::{SymbolLayoutRowAction, SymbolLayoutRowView};
use super::super::toolbars::symbol_layout_list_toolbar_view::SymbolLayoutListToolbarView;
use crate::app_context::AppContext;
use crate::ui::widgets::controls::{list_header::ListHeaderView, search_box::SearchBoxView};
use crate::views::struct_viewer::view_data::struct_viewer_view_data::StructViewerViewData;
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::SymbolLayoutEditorViewData;
use eframe::egui::{RichText, ScrollArea, Ui};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::{data_types::data_type_ref::DataTypeRef, projects::project_symbol_catalog::ProjectSymbolCatalog};
use std::sync::Arc;

pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) struct SymbolLayoutListPanelView<'view> {
    app_context: Arc<AppContext>,
    symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
    project_symbol_catalog: &'view ProjectSymbolCatalog,
    selected_layout_id: Option<&'view str>,
    filter_text: &'view str,
    default_data_type_ref: DataTypeRef,
    is_take_over_active: bool,
}

impl<'view> SymbolLayoutListPanelView<'view> {
    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn new(
        app_context: Arc<AppContext>,
        symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
        struct_viewer_view_data: Dependency<StructViewerViewData>,
        project_symbol_catalog: &'view ProjectSymbolCatalog,
        selected_layout_id: Option<&'view str>,
        filter_text: &'view str,
        default_data_type_ref: DataTypeRef,
        is_take_over_active: bool,
    ) -> Self {
        Self {
            app_context,
            symbol_layout_editor_view_data,
            struct_viewer_view_data,
            project_symbol_catalog,
            selected_layout_id,
            filter_text,
            default_data_type_ref,
            is_take_over_active,
        }
    }

    fn render_filter_text_box(
        &self,
        user_interface: &mut Ui,
    ) {
        let mut edited_filter_text = self.filter_text.to_string();
        user_interface.add(
            SearchBoxView::new(
                self.app_context.clone(),
                &mut edited_filter_text,
                "Filter symbol layouts...",
                "symbol_layout_editor_filter_text",
            )
            .width(user_interface.available_width())
            .height(SymbolLayoutEditorView::FIELD_ROW_HEIGHT),
        );
        if edited_filter_text != self.filter_text {
            SymbolLayoutEditorViewData::set_filter_text(self.symbol_layout_editor_view_data.clone(), edited_filter_text);
        }
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn show(
        self,
        user_interface: &mut Ui,
    ) {
        user_interface.add(
            SymbolLayoutListToolbarView::new(
                self.app_context.clone(),
                self.symbol_layout_editor_view_data.clone(),
                self.project_symbol_catalog,
                self.default_data_type_ref.clone(),
                self.is_take_over_active,
            )
            .height(SymbolLayoutEditorView::TOOLBAR_HEIGHT)
            .icon_button_size(SymbolLayoutEditorView::ICON_BUTTON_WIDTH, SymbolLayoutEditorView::FIELD_ROW_HEIGHT),
        );

        self.render_filter_text_box(user_interface);

        user_interface.add(
            ListHeaderView::new(self.app_context.clone(), "Symbol Layout", "Kind | Entries | Uses")
                .height(SymbolLayoutEditorView::LIST_ROW_HEIGHT)
                .horizontal_padding(8.0),
        );
        ScrollArea::vertical()
            .id_salt("symbol_layout_editor_layout_list")
            .auto_shrink([false, false])
            .show(user_interface, |user_interface| {
                for struct_layout_descriptor in self
                    .project_symbol_catalog
                    .get_struct_layout_descriptors()
                    .iter()
                    .filter(|struct_layout_descriptor| SymbolLayoutEditorViewData::layout_matches_filter(struct_layout_descriptor, self.filter_text))
                {
                    let struct_layout_id = struct_layout_descriptor.get_struct_layout_id();
                    let usage_count = SymbolLayoutEditorViewData::count_symbol_claim_usages(self.project_symbol_catalog, struct_layout_id);
                    let field_count = struct_layout_descriptor
                        .get_struct_layout_definition()
                        .get_fields()
                        .len();
                    let row_action = SymbolLayoutRowView::new(
                        self.app_context.clone(),
                        struct_layout_id,
                        struct_layout_descriptor
                            .get_struct_layout_definition()
                            .get_layout_kind(),
                        field_count,
                        usage_count,
                        self.selected_layout_id == Some(struct_layout_id),
                    )
                    .show(user_interface);
                    match row_action {
                        Some(SymbolLayoutRowAction::Select) => {
                            SymbolLayoutEditorViewData::select_symbol_layout(self.symbol_layout_editor_view_data.clone(), Some(struct_layout_id.to_string()));
                            focus_selected_layout_in_struct_viewer(
                                self.app_context.clone(),
                                self.struct_viewer_view_data.clone(),
                                self.project_symbol_catalog,
                                Some(struct_layout_id),
                            );
                        }
                        Some(SymbolLayoutRowAction::Open) if !self.is_take_over_active => {
                            SymbolLayoutEditorViewData::begin_open_symbol_layout(
                                self.symbol_layout_editor_view_data.clone(),
                                self.project_symbol_catalog,
                                struct_layout_id,
                            );
                        }
                        Some(SymbolLayoutRowAction::Rename) if !self.is_take_over_active => {
                            SymbolLayoutEditorViewData::begin_rename_symbol_layout(
                                self.symbol_layout_editor_view_data.clone(),
                                self.project_symbol_catalog,
                                struct_layout_id,
                            );
                        }
                        _ => {}
                    }
                }

                if self
                    .project_symbol_catalog
                    .get_struct_layout_descriptors()
                    .is_empty()
                {
                    user_interface.label(RichText::new("No symbol layouts yet.").color(self.app_context.theme.foreground_preview));
                }
            });
    }
}
