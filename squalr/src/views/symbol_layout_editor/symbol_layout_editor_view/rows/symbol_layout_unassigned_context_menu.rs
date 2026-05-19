use super::super::SymbolLayoutEditorView;
use super::symbol_layout_unassigned_row_action::SymbolLayoutUnassignedRowAction;
use crate::ui::widgets::controls::{context_menu::context_menu::ContextMenu, toolbar_menu::toolbar_menu_item_view::ToolbarMenuItemView};
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::{SymbolLayoutEditorViewData, SymbolLayoutUnassignedContextMenuTarget};
use eframe::egui::Ui;

pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn render_unassigned_context_menu(
    symbol_layout_editor_view: &SymbolLayoutEditorView,
    user_interface: &mut Ui,
    context_menu_target: &SymbolLayoutUnassignedContextMenuTarget,
    can_define_field: bool,
) -> Option<SymbolLayoutUnassignedRowAction> {
    let theme = &symbol_layout_editor_view.app_context.theme;
    let mut open = true;
    let mut pending_unassigned_row_action = None;

    ContextMenu::new(
        symbol_layout_editor_view.app_context.clone(),
        "symbol_layout_unassigned_context_menu",
        context_menu_target.get_position(),
        |user_interface, should_close| {
            if user_interface
                .add(
                    ToolbarMenuItemView::new(
                        symbol_layout_editor_view.app_context.clone(),
                        "Split Range",
                        "symbol_layout_unassigned_ctx_split_range",
                        &None,
                        SymbolLayoutEditorView::FIELD_CONTEXT_MENU_WIDTH,
                    )
                    .icon(theme.icon_library.icon_handle_common_add.clone())
                    .disabled(context_menu_target.get_size_in_bytes() < 2),
                )
                .clicked()
            {
                pending_unassigned_row_action = Some(SymbolLayoutUnassignedRowAction::SplitRange);
                *should_close = true;
            }

            if user_interface
                .add(
                    ToolbarMenuItemView::new(
                        symbol_layout_editor_view.app_context.clone(),
                        "Merge with Above",
                        "symbol_layout_unassigned_ctx_merge_above",
                        &None,
                        SymbolLayoutEditorView::FIELD_CONTEXT_MENU_WIDTH,
                    )
                    .disabled(context_menu_target.get_merge_above_span().is_none()),
                )
                .clicked()
            {
                pending_unassigned_row_action = Some(SymbolLayoutUnassignedRowAction::MergeAbove);
                *should_close = true;
            }

            if user_interface
                .add(
                    ToolbarMenuItemView::new(
                        symbol_layout_editor_view.app_context.clone(),
                        "Merge with Below",
                        "symbol_layout_unassigned_ctx_merge_below",
                        &None,
                        SymbolLayoutEditorView::FIELD_CONTEXT_MENU_WIDTH,
                    )
                    .disabled(context_menu_target.get_merge_below_span().is_none()),
                )
                .clicked()
            {
                pending_unassigned_row_action = Some(SymbolLayoutUnassignedRowAction::MergeBelow);
                *should_close = true;
            }

            if can_define_field {
                user_interface.separator();

                if user_interface
                    .add(
                        ToolbarMenuItemView::new(
                            symbol_layout_editor_view.app_context.clone(),
                            "Define Field...",
                            "symbol_layout_unassigned_ctx_define_field_at",
                            &None,
                            SymbolLayoutEditorView::FIELD_CONTEXT_MENU_WIDTH,
                        )
                        .icon(theme.icon_library.icon_handle_common_add.clone()),
                    )
                    .clicked()
                {
                    pending_unassigned_row_action = Some(SymbolLayoutUnassignedRowAction::DefineField);
                    *should_close = true;
                }
            }
        },
    )
    .width(SymbolLayoutEditorView::FIELD_CONTEXT_MENU_WIDTH)
    .corner_radius(8)
    .show(user_interface, &mut open);

    if !open {
        SymbolLayoutEditorViewData::hide_unassigned_context_menu(symbol_layout_editor_view.symbol_layout_editor_view_data.clone());
    }

    pending_unassigned_row_action
}
