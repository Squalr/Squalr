use crate::app_context::AppContext;
use crate::ui::widgets::controls::combo_box::{combo_box_item_view::ComboBoxItemView, combo_box_view::ComboBoxView};
use crate::views::symbol_layout_editor::view_data::symbol_layout_field_container_edit::{SymbolLayoutFieldContainerEdit, SymbolLayoutFieldContainerKind};
use eframe::egui::Ui;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use std::sync::Arc;

fn container_label(container_edit: &SymbolLayoutFieldContainerEdit) -> String {
    match container_edit.kind {
        SymbolLayoutFieldContainerKind::Element => String::from("Value"),
        SymbolLayoutFieldContainerKind::Pointer => format!("Ptr {}", container_edit.pointer_size),
        _ => container_edit.kind.label().to_string(),
    }
}

pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn render_define_field_container_selector(
    app_context: Arc<AppContext>,
    user_interface: &mut Ui,
    container_edit: &mut SymbolLayoutFieldContainerEdit,
    menu_id: &str,
    width: f32,
    height: f32,
) {
    let mut selected_container_edit = None;
    let current_label = container_label(container_edit);

    user_interface.add(
        ComboBoxView::new(
            app_context.clone(),
            current_label,
            menu_id,
            None,
            |popup_user_interface: &mut Ui, should_close: &mut bool| {
                let value_response = popup_user_interface.add(ComboBoxItemView::new(app_context.clone(), "Value", None, width));

                if value_response.clicked() {
                    selected_container_edit = Some(SymbolLayoutFieldContainerEdit::default());
                    *should_close = true;
                }

                popup_user_interface.separator();

                for pointer_size in PointerScanPointerSize::ALL {
                    let pointer_label = format!("Ptr {}", pointer_size);
                    let pointer_response = popup_user_interface.add(ComboBoxItemView::new(app_context.clone(), &pointer_label, None, width));

                    if pointer_response.clicked() {
                        selected_container_edit = Some(SymbolLayoutFieldContainerEdit {
                            kind: SymbolLayoutFieldContainerKind::Pointer,
                            pointer_size,
                            ..SymbolLayoutFieldContainerEdit::default()
                        });
                        *should_close = true;
                    }
                }
            },
        )
        .width(width)
        .height(height),
    );

    if let Some(selected_container_edit) = selected_container_edit {
        *container_edit = selected_container_edit;
    }
}
