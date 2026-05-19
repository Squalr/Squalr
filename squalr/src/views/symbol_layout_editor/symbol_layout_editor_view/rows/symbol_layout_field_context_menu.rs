use super::super::SymbolLayoutEditorView;
use super::symbol_layout_field_row_action::SymbolLayoutFieldRowAction;
use crate::ui::widgets::controls::{context_menu::context_menu::ContextMenu, toolbar_menu::toolbar_menu_item_view::ToolbarMenuItemView};
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::{SymbolLayoutEditorViewData, SymbolLayoutFieldContextMenuTarget};
use eframe::egui::Ui;
use squalr_engine_api::structures::structs::symbolic_struct_definition::SymbolicLayoutKind;

pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn render_field_context_menu(
    symbol_layout_editor_view: &SymbolLayoutEditorView,
    user_interface: &mut Ui,
    layout_kind: SymbolicLayoutKind,
    context_menu_target: &SymbolLayoutFieldContextMenuTarget,
    field_count: usize,
    can_delete_final_field: bool,
) -> Option<SymbolLayoutFieldRowAction> {
    let theme = &symbol_layout_editor_view.app_context.theme;
    let field_index = context_menu_target.get_field_index();
    let can_remove_field = can_delete_final_field || field_count > 1;
    let can_move_up = field_index > 0;
    let can_move_down = field_index + 1 < field_count;
    let mut open = true;
    let mut pending_field_row_action = None;
    let entry_name = if layout_kind.is_union() { "variant" } else { "field" };
    let context_menu_id = context_menu_target
        .get_layout_id()
        .map(|layout_id| format!("symbol_layout_field_context_menu_{}", layout_id))
        .unwrap_or_else(|| String::from("symbol_layout_field_context_menu"));

    ContextMenu::new(
        symbol_layout_editor_view.app_context.clone(),
        &context_menu_id,
        context_menu_target.get_position(),
        |user_interface, should_close| {
            if user_interface
                .add(
                    ToolbarMenuItemView::new(
                        symbol_layout_editor_view.app_context.clone(),
                        &format!("Move {} up", entry_name),
                        "symbol_layout_field_ctx_move_up",
                        &None,
                        SymbolLayoutEditorView::FIELD_CONTEXT_MENU_WIDTH,
                    )
                    .icon(theme.icon_library.icon_handle_navigation_up_arrow_small.clone())
                    .disabled(!can_move_up),
                )
                .clicked()
            {
                pending_field_row_action = Some(SymbolLayoutFieldRowAction::MoveUp);
                *should_close = true;
            }

            if user_interface
                .add(
                    ToolbarMenuItemView::new(
                        symbol_layout_editor_view.app_context.clone(),
                        &format!("Move {} down", entry_name),
                        "symbol_layout_field_ctx_move_down",
                        &None,
                        SymbolLayoutEditorView::FIELD_CONTEXT_MENU_WIDTH,
                    )
                    .icon(
                        theme
                            .icon_library
                            .icon_handle_navigation_down_arrow_small
                            .clone(),
                    )
                    .disabled(!can_move_down),
                )
                .clicked()
            {
                pending_field_row_action = Some(SymbolLayoutFieldRowAction::MoveDown);
                *should_close = true;
            }

            if user_interface
                .add(
                    ToolbarMenuItemView::new(
                        symbol_layout_editor_view.app_context.clone(),
                        &format!("Insert new {} below", entry_name),
                        "symbol_layout_field_ctx_insert_below",
                        &None,
                        SymbolLayoutEditorView::FIELD_CONTEXT_MENU_WIDTH,
                    )
                    .icon(theme.icon_library.icon_handle_common_add.clone()),
                )
                .clicked()
            {
                pending_field_row_action = Some(SymbolLayoutFieldRowAction::InsertAfter);
                *should_close = true;
            }

            user_interface.separator();

            if user_interface
                .add(
                    ToolbarMenuItemView::new(
                        symbol_layout_editor_view.app_context.clone(),
                        "Delete",
                        "symbol_layout_field_ctx_delete",
                        &None,
                        SymbolLayoutEditorView::FIELD_CONTEXT_MENU_WIDTH,
                    )
                    .icon(theme.icon_library.icon_handle_common_delete.clone())
                    .icon_background(theme.background_control_danger, theme.background_control_danger_dark)
                    .disabled(!can_remove_field),
                )
                .clicked()
            {
                pending_field_row_action = Some(SymbolLayoutFieldRowAction::RequestRemoveFieldConfirmation);
                *should_close = true;
            }
        },
    )
    .width(SymbolLayoutEditorView::FIELD_CONTEXT_MENU_WIDTH)
    .corner_radius(8)
    .show(user_interface, &mut open);

    if !open {
        SymbolLayoutEditorViewData::hide_field_context_menu(symbol_layout_editor_view.symbol_layout_editor_view_data.clone());
    }

    pending_field_row_action
}
