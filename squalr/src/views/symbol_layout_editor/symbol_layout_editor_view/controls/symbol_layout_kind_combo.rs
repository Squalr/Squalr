use crate::app_context::AppContext;
use crate::ui::widgets::controls::combo_box::{combo_box_item_view::ComboBoxItemView, combo_box_view::ComboBoxView};
use eframe::egui::Ui;
use squalr_engine_api::structures::structs::symbolic_struct_definition::SymbolicLayoutKind;
use std::sync::Arc;

pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn render_symbol_layout_kind_combo(
    app_context: Arc<AppContext>,
    user_interface: &mut Ui,
    layout_kind: &mut SymbolicLayoutKind,
    menu_id: &str,
    combo_width: f32,
    row_height: f32,
) {
    let mut selected_layout_kind = None;

    user_interface.add(
        ComboBoxView::new(app_context.clone(), layout_kind.label(), menu_id, None, |popup_user_interface, should_close| {
            for candidate_layout_kind in SymbolicLayoutKind::ALL {
                let item_response = popup_user_interface.add(ComboBoxItemView::new(app_context.clone(), candidate_layout_kind.label(), None, combo_width));

                if item_response.clicked() {
                    selected_layout_kind = Some(candidate_layout_kind);
                    *should_close = true;
                }
            }
        })
        .width(combo_width)
        .popup_width(combo_width)
        .height(row_height),
    );

    if let Some(selected_layout_kind) = selected_layout_kind {
        *layout_kind = selected_layout_kind;
    }
}
