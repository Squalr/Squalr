use crate::app_context::AppContext;
use crate::ui::converters::{data_type_to_icon_converter::DataTypeToIconConverter, data_type_to_string_converter::DataTypeToStringConverter};
use crate::ui::widgets::controls::{
    combo_box::combo_box_item_view::ComboBoxItemView, combo_box::combo_box_view::ComboBoxView, groupbox::GroupBox, search_box::SearchBoxView,
};
use crate::views::symbol_tree::symbol_tree_take_over_view_helpers::{
    SYMBOL_TREE_TAKE_OVER_ACTION_BUTTON_SPACING, SYMBOL_TREE_TAKE_OVER_ACTION_BUTTON_WIDTH, SYMBOL_TREE_TAKE_OVER_ROW_HEIGHT, draw_sized_action_button,
    render_offset_data_value_box, render_string_data_value_box,
};
use crate::views::symbol_tree::view_data::symbol_tree_view_data::DefineFieldDraft;
use eframe::egui::{Direction, Grid, Id, Layout, RichText, ScrollArea, Ui, vec2};
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::container_type::ContainerType;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use squalr_engine_api::structures::projects::symbol_tree::operations::define_field::{
    DefineFieldPlan, DefineFieldPlanRequest, build_define_field_plan, build_define_field_symbol_layout_id, filter_registered_pointer_sizes,
    parse_define_field_relative_offset,
};
use squalr_engine_api::structures::structs::symbolic_field_definition::SymbolicFieldDefinition;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;

const DEFINE_FIELD_BUILT_IN_TYPE_COLUMN_COUNT: usize = 2;
const DEFINE_FIELD_BUILT_IN_TYPE_ITEM_WIDTH: f32 = 128.0;
const DEFINE_FIELD_BUILT_IN_TYPE_COLUMN_SPACING: f32 = 4.0;
const DEFINE_FIELD_CONTAINER_SELECTOR_WIDTH: f32 = 118.0;
const MODULE_FIELD_BUILT_IN_TYPE_IDS: [&str; 18] = [
    "u8", "i8", "i16", "i16be", "i32", "i32be", "i64", "i64be", "u16", "u16be", "u32", "u32be", "u64", "u64be", "f32", "f32be", "f64", "f64be",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModuleFieldTypeOptionKind {
    BuiltIn,
    SymbolLayout,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleFieldTypeOption {
    pub data_type_ref: DataTypeRef,
    pub label: String,
    pub kind: ModuleFieldTypeOptionKind,
}

#[derive(Clone, Debug)]
pub enum SymbolTreeDefineFieldAction {
    None,
    Cancel,
    Create(DefineFieldPlan),
    DraftChanged(DefineFieldDraft),
}

pub struct SymbolTreeDefineFieldView<'a> {
    app_context: Arc<AppContext>,
    project_symbol_catalog: &'a ProjectSymbolCatalog,
    module_name: &'a str,
    segment_offset: u64,
    segment_length: u64,
    define_field_draft: &'a DefineFieldDraft,
}

impl<'a> SymbolTreeDefineFieldView<'a> {
    pub fn new(
        app_context: Arc<AppContext>,
        project_symbol_catalog: &'a ProjectSymbolCatalog,
        module_name: &'a str,
        segment_offset: u64,
        segment_length: u64,
        define_field_draft: &'a DefineFieldDraft,
    ) -> Self {
        Self {
            app_context,
            project_symbol_catalog,
            module_name,
            segment_offset,
            segment_length,
            define_field_draft,
        }
    }

    pub fn build_module_field_type_options(project_symbol_catalog: &ProjectSymbolCatalog) -> Vec<ModuleFieldTypeOption> {
        let mut type_options = MODULE_FIELD_BUILT_IN_TYPE_IDS
            .iter()
            .map(|data_type_id| ModuleFieldTypeOption {
                data_type_ref: DataTypeRef::new(data_type_id),
                label: DataTypeToStringConverter::convert_data_type_to_string(data_type_id),
                kind: ModuleFieldTypeOptionKind::BuiltIn,
            })
            .collect::<Vec<_>>();

        for struct_layout_descriptor in project_symbol_catalog.get_struct_layout_descriptors() {
            let struct_layout_id = struct_layout_descriptor.get_struct_layout_id();
            let struct_data_type_ref = DataTypeRef::new(struct_layout_id);

            if !type_options
                .iter()
                .any(|type_option| type_option.data_type_ref == struct_data_type_ref)
            {
                type_options.push(ModuleFieldTypeOption {
                    data_type_ref: struct_data_type_ref,
                    label: struct_layout_id.to_string(),
                    kind: ModuleFieldTypeOptionKind::SymbolLayout,
                });
            }
        }

        type_options
    }

    pub fn filter_module_field_type_options(
        type_options: &[ModuleFieldTypeOption],
        search_text: &str,
    ) -> Vec<ModuleFieldTypeOption> {
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

    pub fn module_field_type_option_uses_icon(type_option_kind: ModuleFieldTypeOptionKind) -> bool {
        matches!(type_option_kind, ModuleFieldTypeOptionKind::BuiltIn)
    }

    pub fn define_field_type_popup_width(combo_width: f32) -> f32 {
        let built_in_grid_width = DEFINE_FIELD_BUILT_IN_TYPE_ITEM_WIDTH * DEFINE_FIELD_BUILT_IN_TYPE_COLUMN_COUNT as f32
            + DEFINE_FIELD_BUILT_IN_TYPE_COLUMN_SPACING * (DEFINE_FIELD_BUILT_IN_TYPE_COLUMN_COUNT.saturating_sub(1) as f32);

        combo_width.max(built_in_grid_width)
    }

    pub fn define_field_builtin_type_item_width(popup_width: f32) -> f32 {
        let spacing_width = DEFINE_FIELD_BUILT_IN_TYPE_COLUMN_SPACING * (DEFINE_FIELD_BUILT_IN_TYPE_COLUMN_COUNT.saturating_sub(1) as f32);

        ((popup_width - spacing_width) / DEFINE_FIELD_BUILT_IN_TYPE_COLUMN_COUNT as f32).max(1.0)
    }

    pub fn build_define_field_plan(
        define_field_draft: &DefineFieldDraft,
        module_name: &str,
        segment_offset: u64,
        segment_length: u64,
        resolve_type_size: impl Fn(&str) -> Option<u64>,
    ) -> Result<DefineFieldPlan, String> {
        let define_field_plan_request = DefineFieldPlanRequest {
            display_name: define_field_draft.display_name.clone(),
            relative_offset_text: define_field_draft.relative_offset_text.clone(),
            relative_offset_format: define_field_draft.relative_offset_format,
            container_type: define_field_draft.container_type,
            data_type_ref: define_field_draft
                .data_type_selection
                .visible_data_type()
                .clone(),
        };

        build_define_field_plan(&define_field_plan_request, module_name, segment_offset, segment_length, resolve_type_size)
    }

    pub fn build_define_field_offset_warning(
        define_field_draft: &DefineFieldDraft,
        segment_length: u64,
        resolve_type_size: impl Fn(&str) -> Option<u64>,
    ) -> Option<String> {
        let relative_offset = match parse_define_field_relative_offset(&define_field_draft.relative_offset_text, define_field_draft.relative_offset_format) {
            Ok(relative_offset) => relative_offset,
            Err(parse_error) => return Some(parse_error),
        };
        let struct_layout_id =
            build_define_field_symbol_layout_id(define_field_draft.data_type_selection.visible_data_type(), define_field_draft.container_type);
        let Some(field_size) = resolve_type_size(&struct_layout_id) else {
            return Some(format!("Cannot resolve byte size for `{}`.", struct_layout_id));
        };

        if field_size == 0 {
            return Some(format!("`{}` has no byte size.", struct_layout_id));
        }

        let relative_field_end = match relative_offset.checked_add(field_size) {
            Some(relative_field_end) => relative_field_end,
            None => return Some(String::from("Field range is too large.")),
        };

        if relative_field_end > segment_length {
            if field_size > segment_length {
                return Some(format!(
                    "`{}` uses {} byte(s); selected span has {}.",
                    struct_layout_id, field_size, segment_length
                ));
            }

            return Some(format!(
                "`{}` uses {} byte(s); choose 0 to {}.",
                struct_layout_id,
                field_size,
                segment_length.saturating_sub(field_size)
            ));
        }

        None
    }

    pub fn show(
        self,
        user_interface: &mut Ui,
    ) -> SymbolTreeDefineFieldAction {
        let theme = &self.app_context.theme;
        let original_define_field_draft = self.define_field_draft.clone();
        let mut edited_define_field_draft = original_define_field_draft.clone();
        let mut define_field_plan_result = Err(String::from("Field is not ready."));
        let mut action = SymbolTreeDefineFieldAction::None;

        user_interface.allocate_ui_with_layout(
            user_interface.available_size(),
            Layout::centered_and_justified(Direction::TopDown),
            |user_interface| {
                let panel_width = user_interface.available_width();

                user_interface.add(
                    GroupBox::new_from_theme(theme, "Define Field", |user_interface| {
                        user_interface.label(RichText::new(format!("{} + 0x{:X}", self.module_name, self.segment_offset)).color(theme.foreground_preview));
                        user_interface.add_space(8.0);

                        user_interface.label(RichText::new("Name").color(theme.foreground));
                        user_interface.add_space(2.0);
                        render_string_data_value_box(
                            self.app_context.clone(),
                            user_interface,
                            &mut edited_define_field_draft.display_name,
                            "field_name",
                            "symbol_tree_define_field_name",
                            user_interface.available_width(),
                        );
                        user_interface.add_space(8.0);

                        let max_relative_offset = self.segment_length.saturating_sub(1);
                        user_interface.label(RichText::new(format!("Offset in UNASSIGNED (0 to {})", max_relative_offset)).color(theme.foreground));
                        user_interface.add_space(2.0);
                        render_offset_data_value_box(
                            self.app_context.clone(),
                            user_interface,
                            &mut edited_define_field_draft.relative_offset_text,
                            &mut edited_define_field_draft.relative_offset_format,
                            "0",
                            "symbol_tree_define_field_offset",
                            user_interface.available_width(),
                        );

                        if let Some(offset_warning) =
                            Self::build_define_field_offset_warning(&edited_define_field_draft, self.segment_length, |struct_layout_id| {
                                self.resolve_define_field_symbol_layout_id_size(struct_layout_id)
                            })
                        {
                            user_interface.add_space(4.0);
                            user_interface.label(RichText::new(offset_warning).color(theme.warning));
                        }
                        user_interface.add_space(8.0);

                        user_interface.horizontal(|user_interface| {
                            user_interface.spacing_mut().item_spacing.x = 4.0;
                            let pointer_sizes = filter_registered_pointer_sizes(
                                &self
                                    .app_context
                                    .engine_unprivileged_state
                                    .get_registered_data_type_refs(),
                            );
                            let selector_width = DEFINE_FIELD_CONTAINER_SELECTOR_WIDTH.min(user_interface.available_width());
                            self.render_define_field_container_selector(
                                user_interface,
                                &mut edited_define_field_draft.container_type,
                                &pointer_sizes,
                                &format!("symbol_tree_define_field_container_{}_{}", self.module_name, self.segment_offset),
                                selector_width,
                            );

                            let type_selector_width = user_interface.available_width();
                            self.render_module_field_type_combo(
                                user_interface,
                                &mut edited_define_field_draft.data_type_selection,
                                &format!("symbol_tree_define_field_type_{}_{}", self.module_name, self.segment_offset),
                                type_selector_width,
                            );
                        });

                        define_field_plan_result = Self::build_define_field_plan(
                            &edited_define_field_draft,
                            self.module_name,
                            self.segment_offset,
                            self.segment_length,
                            |struct_layout_id| self.resolve_define_field_symbol_layout_id_size(struct_layout_id),
                        );

                        if let Err(validation_error) = define_field_plan_result.as_ref() {
                            if validation_error == "Field name is required." {
                                user_interface.add_space(6.0);
                                user_interface.label(RichText::new(validation_error).color(theme.error_red));
                            }
                        }

                        user_interface.add_space(12.0);
                        user_interface.allocate_ui(vec2(user_interface.available_width(), 32.0), |user_interface| {
                            let button_size = vec2(SYMBOL_TREE_TAKE_OVER_ACTION_BUTTON_WIDTH, SYMBOL_TREE_TAKE_OVER_ROW_HEIGHT);
                            let total_button_row_width = button_size.x * 2.0 + SYMBOL_TREE_TAKE_OVER_ACTION_BUTTON_SPACING;
                            let side_spacing = ((user_interface.available_width() - total_button_row_width) * 0.5).max(0.0);

                            user_interface.horizontal(|user_interface| {
                                user_interface.add_space(side_spacing);
                                user_interface.spacing_mut().item_spacing.x = SYMBOL_TREE_TAKE_OVER_ACTION_BUTTON_SPACING;

                                let button_cancel = draw_sized_action_button(
                                    &self.app_context,
                                    user_interface,
                                    "Cancel",
                                    button_size,
                                    theme.background_control_secondary,
                                    theme.background_control_secondary_dark,
                                    true,
                                );

                                if button_cancel.clicked() {
                                    action = SymbolTreeDefineFieldAction::Cancel;
                                }

                                let can_create_field = define_field_plan_result.is_ok();
                                let create_fill = if can_create_field {
                                    theme.background_control_primary
                                } else {
                                    theme.background_control_secondary
                                };
                                let create_stroke = if can_create_field {
                                    theme.background_control_primary_dark
                                } else {
                                    theme.background_control_secondary_dark
                                };
                                let button_create = draw_sized_action_button(
                                    &self.app_context,
                                    user_interface,
                                    "Create",
                                    button_size,
                                    create_fill,
                                    create_stroke,
                                    can_create_field,
                                );

                                if button_create.clicked() {
                                    if let Ok(define_field_plan) = define_field_plan_result.clone() {
                                        action = SymbolTreeDefineFieldAction::Create(define_field_plan);
                                    }
                                }
                            });
                        });
                    })
                    .desired_width(panel_width),
                );
            },
        );

        if matches!(action, SymbolTreeDefineFieldAction::None) && edited_define_field_draft != original_define_field_draft {
            return SymbolTreeDefineFieldAction::DraftChanged(edited_define_field_draft);
        }

        action
    }

    fn resolve_define_field_symbol_layout_id_size(
        &self,
        struct_layout_id: &str,
    ) -> Option<u64> {
        let symbolic_field_definition = SymbolicFieldDefinition::from_str(struct_layout_id).ok()?;

        self.resolve_define_field_symbolic_size(&symbolic_field_definition, &mut HashSet::new())
    }

    fn resolve_define_field_symbolic_size(
        &self,
        symbolic_field_definition: &SymbolicFieldDefinition,
        visited_type_ids: &mut HashSet<String>,
    ) -> Option<u64> {
        if let Some(pointer_size) = symbolic_field_definition
            .get_container_type()
            .get_pointer_size()
        {
            return Some(pointer_size.get_size_in_bytes());
        }

        let data_type_id = symbolic_field_definition
            .get_data_type_ref()
            .get_data_type_id()
            .to_string();
        let unit_size_in_bytes = if let Some(symbolic_struct_definition) = self
            .project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == data_type_id)
            .map(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_definition().clone())
        {
            if !visited_type_ids.insert(data_type_id.clone()) {
                return None;
            }

            let struct_size_in_bytes = symbolic_struct_definition
                .get_fields()
                .iter()
                .try_fold(0_u64, |accumulated_size, field_definition| {
                    accumulated_size.checked_add(self.resolve_define_field_symbolic_size(field_definition, visited_type_ids)?)
                })?;

            visited_type_ids.remove(&data_type_id);
            struct_size_in_bytes
        } else if let Some(default_value) = self
            .app_context
            .engine_unprivileged_state
            .get_default_value(symbolic_field_definition.get_data_type_ref())
        {
            default_value.get_size_in_bytes()
        } else {
            return None;
        };

        Some(
            symbolic_field_definition
                .get_container_type()
                .get_total_size_in_bytes(unit_size_in_bytes),
        )
    }

    fn define_field_container_label(container_type: ContainerType) -> String {
        match container_type {
            ContainerType::None => String::from("Value"),
            _ => container_type
                .get_pointer_size()
                .map(|pointer_size| format!("Ptr {}", pointer_size))
                .unwrap_or_else(|| String::from("Value")),
        }
    }

    fn module_field_type_search_storage_id(menu_id: &str) -> Id {
        Id::new(("symbol_tree_module_field_type_search", menu_id))
    }

    fn render_define_field_container_selector(
        &self,
        user_interface: &mut Ui,
        container_type: &mut ContainerType,
        pointer_sizes: &[PointerScanPointerSize],
        menu_id: &str,
        width: f32,
    ) {
        let mut selected_container_type = None;
        if let Some(selected_pointer_size) = container_type.get_pointer_size() {
            if !pointer_sizes.contains(&selected_pointer_size) {
                *container_type = ContainerType::None;
            }
        }
        let current_label = Self::define_field_container_label(*container_type);

        user_interface.add(
            ComboBoxView::new(
                self.app_context.clone(),
                current_label,
                menu_id,
                None,
                |popup_user_interface: &mut Ui, should_close: &mut bool| {
                    let value_response = popup_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), "Value", None, width));

                    if value_response.clicked() {
                        selected_container_type = Some(ContainerType::None);
                        *should_close = true;
                    }

                    popup_user_interface.separator();

                    for pointer_size in pointer_sizes {
                        let pointer_label = format!("Ptr {}", pointer_size);
                        let pointer_response = popup_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), &pointer_label, None, width));

                        if pointer_response.clicked() {
                            selected_container_type = Some(ContainerType::Pointer(*pointer_size));
                            *should_close = true;
                        }
                    }
                },
            )
            .width(width)
            .height(SYMBOL_TREE_TAKE_OVER_ROW_HEIGHT),
        );

        if let Some(selected_container_type) = selected_container_type {
            *container_type = selected_container_type;
        }
    }

    fn render_module_field_type_combo(
        &self,
        user_interface: &mut Ui,
        data_type_selection: &mut crate::ui::widgets::controls::data_type_selector::data_type_selection::DataTypeSelection,
        menu_id: &str,
        width: f32,
    ) {
        let type_options = Self::build_module_field_type_options(self.project_symbol_catalog);
        let selected_data_type_id = data_type_selection
            .visible_data_type()
            .get_data_type_id()
            .to_string();
        let selected_type_option = type_options
            .iter()
            .find(|type_option| type_option.data_type_ref.get_data_type_id() == selected_data_type_id.as_str());
        let combo_label = selected_type_option
            .map(|type_option| type_option.label.clone())
            .unwrap_or_else(|| DataTypeToStringConverter::convert_data_type_to_string(&selected_data_type_id));
        let combo_icon = selected_type_option.and_then(|type_option| {
            Self::module_field_type_option_uses_icon(type_option.kind)
                .then(|| DataTypeToIconConverter::convert_data_type_to_icon(type_option.data_type_ref.get_data_type_id(), &self.app_context.theme.icon_library))
        });
        let search_storage_id = Self::module_field_type_search_storage_id(menu_id);
        let popup_width = Self::define_field_type_popup_width(width);
        let built_in_type_item_width = Self::define_field_builtin_type_item_width(popup_width);

        user_interface.add(
            ComboBoxView::new(
                self.app_context.clone(),
                combo_label,
                menu_id,
                combo_icon,
                |popup_user_interface: &mut Ui, should_close: &mut bool| {
                    let mut search_text = popup_user_interface
                        .ctx()
                        .data_mut(|data| data.get_temp::<String>(search_storage_id).unwrap_or_default());

                    popup_user_interface.add_space(4.0);
                    let search_box_id = format!("symbol_tree_module_field_type_search_{}", menu_id);
                    popup_user_interface.add(
                        SearchBoxView::new(self.app_context.clone(), &mut search_text, "Search types", &search_box_id)
                            .width((popup_width - 8.0).max(1.0))
                            .height(SYMBOL_TREE_TAKE_OVER_ROW_HEIGHT),
                    );
                    popup_user_interface.add_space(4.0);

                    popup_user_interface
                        .ctx()
                        .data_mut(|data| data.insert_temp(search_storage_id, search_text.clone()));

                    let filtered_type_options = Self::filter_module_field_type_options(&type_options, &search_text);

                    if filtered_type_options.is_empty() {
                        popup_user_interface.label(RichText::new("No matching types").color(self.app_context.theme.foreground_preview));
                        return;
                    }

                    let (built_in_type_options, symbol_layout_type_options): (Vec<_>, Vec<_>) = filtered_type_options
                        .into_iter()
                        .partition(|type_option| type_option.kind == ModuleFieldTypeOptionKind::BuiltIn);

                    ScrollArea::vertical()
                        .max_height(240.0)
                        .auto_shrink([false, false])
                        .show(popup_user_interface, |scroll_user_interface| {
                            if !built_in_type_options.is_empty() {
                                Grid::new(Id::new(("symbol_tree_define_field_builtin_type_grid", menu_id)))
                                    .spacing(vec2(DEFINE_FIELD_BUILT_IN_TYPE_COLUMN_SPACING, 0.0))
                                    .min_col_width(DEFINE_FIELD_BUILT_IN_TYPE_ITEM_WIDTH)
                                    .show(scroll_user_interface, |grid_user_interface| {
                                        for (type_option_position, type_option) in built_in_type_options.iter().enumerate() {
                                            let data_type_id = type_option.data_type_ref.get_data_type_id();
                                            let row_icon = Some(DataTypeToIconConverter::convert_data_type_to_icon(
                                                data_type_id,
                                                &self.app_context.theme.icon_library,
                                            ));
                                            let item_response = grid_user_interface.add(ComboBoxItemView::new(
                                                self.app_context.clone(),
                                                &type_option.label,
                                                row_icon,
                                                built_in_type_item_width,
                                            ));

                                            if item_response.clicked() {
                                                data_type_selection.select_single_data_type(type_option.data_type_ref.clone());
                                                grid_user_interface
                                                    .ctx()
                                                    .data_mut(|data| data.insert_temp(search_storage_id, String::new()));
                                                *should_close = true;
                                            }

                                            if (type_option_position + 1) % DEFINE_FIELD_BUILT_IN_TYPE_COLUMN_COUNT == 0 {
                                                grid_user_interface.end_row();
                                            }
                                        }

                                        if built_in_type_options.len() % DEFINE_FIELD_BUILT_IN_TYPE_COLUMN_COUNT != 0 {
                                            grid_user_interface.end_row();
                                        }
                                    });
                            }

                            if !built_in_type_options.is_empty() && !symbol_layout_type_options.is_empty() {
                                scroll_user_interface.separator();
                            }

                            for type_option in symbol_layout_type_options {
                                let item_response =
                                    scroll_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), &type_option.label, None, popup_width));

                                if item_response.clicked() {
                                    data_type_selection.select_single_data_type(type_option.data_type_ref);
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
            .height(SYMBOL_TREE_TAKE_OVER_ROW_HEIGHT),
        );
    }
}
