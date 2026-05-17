use crate::app_context::AppContext;
use crate::ui::converters::{data_type_to_icon_converter::DataTypeToIconConverter, data_type_to_string_converter::DataTypeToStringConverter};
use crate::ui::widgets::controls::{
    combo_box::{combo_box_item_view::ComboBoxItemView, combo_box_view::ComboBoxView},
    search_box::SearchBoxView,
};
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::SymbolLayoutFieldEditDraft;
use eframe::egui::{Grid, Id, RichText, ScrollArea, Ui, vec2};
use squalr_engine_api::structures::{data_types::data_type_ref::DataTypeRef, projects::project_symbol_catalog::ProjectSymbolCatalog};
use std::sync::Arc;

const BUILT_IN_TYPE_COLUMN_COUNT: usize = 2;
const BUILT_IN_TYPE_ITEM_WIDTH: f32 = 128.0;
const BUILT_IN_TYPE_COLUMN_SPACING: f32 = 4.0;
const BUILT_IN_TYPE_IDS: [&str; 18] = [
    "u8", "i8", "i16", "i16be", "i32", "i32be", "i64", "i64be", "u16", "u16be", "u32", "u32be", "u64", "u64be", "f32", "f32be", "f64", "f64be",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SymbolLayoutFieldTypeOptionKind {
    BuiltIn,
    SymbolLayout,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SymbolLayoutFieldTypeOption {
    data_type_ref: DataTypeRef,
    label: String,
    kind: SymbolLayoutFieldTypeOptionKind,
}

fn build_field_type_options(project_symbol_catalog: &ProjectSymbolCatalog) -> Vec<SymbolLayoutFieldTypeOption> {
    let mut type_options = BUILT_IN_TYPE_IDS
        .iter()
        .map(|data_type_id| SymbolLayoutFieldTypeOption {
            data_type_ref: DataTypeRef::new(data_type_id),
            label: DataTypeToStringConverter::convert_data_type_to_string(data_type_id),
            kind: SymbolLayoutFieldTypeOptionKind::BuiltIn,
        })
        .collect::<Vec<_>>();

    for struct_layout_descriptor in project_symbol_catalog.get_struct_layout_descriptors() {
        let struct_layout_id = struct_layout_descriptor.get_struct_layout_id();
        let struct_data_type_ref = DataTypeRef::new(struct_layout_id);

        if !type_options
            .iter()
            .any(|type_option| type_option.data_type_ref == struct_data_type_ref)
        {
            type_options.push(SymbolLayoutFieldTypeOption {
                data_type_ref: struct_data_type_ref,
                label: struct_layout_id.to_string(),
                kind: SymbolLayoutFieldTypeOptionKind::SymbolLayout,
            });
        }
    }

    type_options
}

fn filter_field_type_options(
    type_options: &[SymbolLayoutFieldTypeOption],
    search_text: &str,
) -> Vec<SymbolLayoutFieldTypeOption> {
    let normalized_search_text = search_text.trim().to_lowercase();

    if normalized_search_text.is_empty() {
        return type_options.to_vec();
    }

    type_options
        .iter()
        .filter(|type_option| {
            type_option
                .label
                .to_lowercase()
                .contains(&normalized_search_text)
                || type_option
                    .data_type_ref
                    .get_data_type_id()
                    .to_lowercase()
                    .contains(&normalized_search_text)
        })
        .cloned()
        .collect()
}

fn type_popup_width(combo_width: f32) -> f32 {
    let built_in_grid_width =
        BUILT_IN_TYPE_ITEM_WIDTH * BUILT_IN_TYPE_COLUMN_COUNT as f32 + BUILT_IN_TYPE_COLUMN_SPACING * (BUILT_IN_TYPE_COLUMN_COUNT.saturating_sub(1) as f32);

    combo_width.max(built_in_grid_width)
}

fn built_in_type_item_width(popup_width: f32) -> f32 {
    let spacing_width = BUILT_IN_TYPE_COLUMN_SPACING * (BUILT_IN_TYPE_COLUMN_COUNT.saturating_sub(1) as f32);

    ((popup_width - spacing_width) / BUILT_IN_TYPE_COLUMN_COUNT as f32).max(1.0)
}

fn type_search_storage_id(menu_id: &str) -> Id {
    Id::new(("symbol_layout_define_field_type_search", menu_id))
}

pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn render_define_field_type_combo(
    app_context: Arc<AppContext>,
    user_interface: &mut Ui,
    project_symbol_catalog: &ProjectSymbolCatalog,
    field_draft: &mut SymbolLayoutFieldEditDraft,
    menu_id: &str,
    width: f32,
    height: f32,
) {
    let type_options = build_field_type_options(project_symbol_catalog);
    let selected_data_type_id = field_draft
        .data_type_selection
        .visible_data_type()
        .get_data_type_id()
        .to_string();
    let selected_type_option = type_options
        .iter()
        .find(|type_option| type_option.data_type_ref.get_data_type_id() == selected_data_type_id.as_str());
    let combo_label = selected_type_option
        .map(|type_option| type_option.label.clone())
        .unwrap_or_else(|| DataTypeToStringConverter::convert_data_type_to_string(&selected_data_type_id));
    let combo_icon = selected_type_option.map(|type_option| {
        DataTypeToIconConverter::convert_data_type_or_symbol_layout_to_icon(
            type_option.data_type_ref.get_data_type_id(),
            type_option.kind == SymbolLayoutFieldTypeOptionKind::SymbolLayout,
            &app_context.theme.icon_library,
        )
    });
    let search_storage_id = type_search_storage_id(menu_id);
    let popup_width = type_popup_width(width);
    let built_in_type_item_width = built_in_type_item_width(popup_width);

    user_interface.add(
        ComboBoxView::new(
            app_context.clone(),
            combo_label,
            menu_id,
            combo_icon,
            |popup_user_interface: &mut Ui, should_close: &mut bool| {
                let mut search_text = popup_user_interface
                    .ctx()
                    .data_mut(|data| data.get_temp::<String>(search_storage_id).unwrap_or_default());

                popup_user_interface.add_space(4.0);
                let search_box_id = format!("symbol_layout_define_field_type_search_{}", menu_id);
                popup_user_interface.add(
                    SearchBoxView::new(app_context.clone(), &mut search_text, "Search types", &search_box_id)
                        .width((popup_width - 8.0).max(1.0))
                        .height(height),
                );
                popup_user_interface.add_space(4.0);
                popup_user_interface
                    .ctx()
                    .data_mut(|data| data.insert_temp(search_storage_id, search_text.clone()));

                let filtered_type_options = filter_field_type_options(&type_options, &search_text);

                if filtered_type_options.is_empty() {
                    popup_user_interface.label(RichText::new("No matching types").color(app_context.theme.foreground_preview));
                    return;
                }

                let (built_in_type_options, symbol_layout_type_options): (Vec<_>, Vec<_>) = filtered_type_options
                    .into_iter()
                    .partition(|type_option| type_option.kind == SymbolLayoutFieldTypeOptionKind::BuiltIn);

                ScrollArea::vertical()
                    .max_height(240.0)
                    .auto_shrink([false, false])
                    .show(popup_user_interface, |scroll_user_interface| {
                        if !built_in_type_options.is_empty() {
                            Grid::new(Id::new(("symbol_layout_define_field_builtin_type_grid", menu_id)))
                                .spacing(vec2(BUILT_IN_TYPE_COLUMN_SPACING, 0.0))
                                .min_col_width(BUILT_IN_TYPE_ITEM_WIDTH)
                                .show(scroll_user_interface, |grid_user_interface| {
                                    for (type_option_position, type_option) in built_in_type_options.iter().enumerate() {
                                        let data_type_id = type_option.data_type_ref.get_data_type_id();
                                        let row_icon = Some(DataTypeToIconConverter::convert_data_type_to_icon(
                                            data_type_id,
                                            &app_context.theme.icon_library,
                                        ));
                                        let item_response = grid_user_interface.add(ComboBoxItemView::new(
                                            app_context.clone(),
                                            &type_option.label,
                                            row_icon,
                                            built_in_type_item_width,
                                        ));

                                        if item_response.clicked() {
                                            field_draft
                                                .data_type_selection
                                                .select_single_data_type(type_option.data_type_ref.clone());
                                            grid_user_interface
                                                .ctx()
                                                .data_mut(|data| data.insert_temp(search_storage_id, String::new()));
                                            *should_close = true;
                                        }

                                        if (type_option_position + 1) % BUILT_IN_TYPE_COLUMN_COUNT == 0 {
                                            grid_user_interface.end_row();
                                        }
                                    }

                                    if built_in_type_options.len() % BUILT_IN_TYPE_COLUMN_COUNT != 0 {
                                        grid_user_interface.end_row();
                                    }
                                });
                        }

                        if !built_in_type_options.is_empty() && !symbol_layout_type_options.is_empty() {
                            scroll_user_interface.separator();
                        }

                        for type_option in symbol_layout_type_options {
                            let item_response = scroll_user_interface.add(ComboBoxItemView::new(
                                app_context.clone(),
                                &type_option.label,
                                Some(DataTypeToIconConverter::convert_symbol_layout_to_icon(&app_context.theme.icon_library)),
                                popup_width,
                            ));

                            if item_response.clicked() {
                                field_draft
                                    .data_type_selection
                                    .select_single_data_type(type_option.data_type_ref);
                                scroll_user_interface
                                    .ctx()
                                    .data_mut(|data| data.insert_temp(search_storage_id, String::new()));
                                *should_close = true;
                            }
                        }
                    });
            },
        )
        .width(width)
        .popup_width(popup_width)
        .height(height),
    );
}
