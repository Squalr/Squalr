use crate::app_context::AppContext;
use crate::ui::draw::icon_draw::IconDraw;
use crate::ui::widgets::controls::combo_box::{combo_box_item_view::ComboBoxItemView, combo_box_view::ComboBoxView};
use crate::ui::widgets::controls::data_type_selector::data_type_selector_view::DataTypeSelectorView;
use crate::ui::widgets::controls::{
    button::Button as ThemeButton, data_value_box::data_value_box_view::DataValueBoxView, groupbox::GroupBox, state_layer::StateLayer,
};
use crate::views::symbol_struct_editor::view_data::symbol_struct_editor_view_data::{
    SymbolStructEditorTakeOverState, SymbolStructEditorViewData, SymbolStructFieldEditDraft, SymbolStructFieldOffsetMode, SymbolStructLayoutEditDraft,
};
use crate::views::symbol_struct_editor::view_data::symbol_struct_field_container_edit::{
    SymbolStructFieldContainerEdit, SymbolStructFieldContainerKind, SymbolStructFieldDynamicCountMode,
};
use eframe::egui::{Align, Align2, ComboBox, Direction, Key, Layout, Response, RichText, ScrollArea, Sense, Stroke, Ui, UiBuilder, Widget, pos2, vec2};
use epaint::{Color32, CornerRadius, StrokeKind};
use squalr_engine_api::commands::{
    privileged_command_request::PrivilegedCommandRequest, project::save::project_save_request::ProjectSaveRequest,
    registry::set_project_symbols::registry_set_project_symbols_request::RegistrySetProjectSymbolsRequest,
    unprivileged_command_request::UnprivilegedCommandRequest,
};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::{
    data_types::{
        built_in_types::{i32::data_type_i32::DataTypeI32, string::utf8::data_type_string_utf8::DataTypeStringUtf8, u64::data_type_u64::DataTypeU64},
        data_type_ref::DataTypeRef,
    },
    data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
    pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
    projects::project_symbol_catalog::ProjectSymbolCatalog,
    structs::symbolic_field_definition::SymbolicFieldDefinition,
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SymbolStructFieldRowAction {
    InsertAfter,
    RemoveField,
    MoveUp,
    MoveDown,
    EditLayout,
}

#[derive(Clone)]
pub struct SymbolStructEditorView {
    app_context: Arc<AppContext>,
    symbol_struct_editor_view_data: Dependency<SymbolStructEditorViewData>,
}

impl SymbolStructEditorView {
    pub const WINDOW_ID: &'static str = "window_symbol_struct_editor";
    const FIELD_ROW_HEIGHT: f32 = 28.0;
    const LIST_ROW_HEIGHT: f32 = 28.0;
    const ICON_BUTTON_WIDTH: f32 = 36.0;
    const FIELD_SECTION_VERTICAL_SPACING: f32 = 10.0;
    const FIELD_INPUT_SPACING: f32 = 8.0;
    const FIELD_CONTAINER_MODE_WIDTH: f32 = 160.0;
    const FIELD_CONTAINER_DETAIL_WIDTH: f32 = 140.0;
    const FIELD_OFFSET_MODE_WIDTH: f32 = 160.0;
    const FIELD_EXPRESSION_PICKER_WIDTH: f32 = 190.0;
    const FIELD_EXPRESSION_OPERATOR_BUTTON_WIDTH: f32 = 32.0;
    const TAKE_OVER_HEADER_HEIGHT: f32 = 32.0;
    const TAKE_OVER_PADDING_X: f32 = 0.0;
    const TAKE_OVER_PADDING_Y: f32 = 0.0;
    const TAKE_OVER_CONTENT_PADDING_X: f32 = 12.0;
    const TAKE_OVER_HEADER_TITLE_PADDING_X: f32 = 8.0;
    const TAKE_OVER_SECTION_SPACING: f32 = 12.0;
    const TAKE_OVER_GROUPBOX_SPACING: f32 = 8.0;

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let symbol_struct_editor_view_data = app_context
            .dependency_container
            .register(SymbolStructEditorViewData::new());

        Self {
            app_context,
            symbol_struct_editor_view_data,
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

    fn persist_project_symbol_catalog(
        &self,
        updated_project_symbol_catalog: ProjectSymbolCatalog,
    ) {
        let opened_project_lock = self
            .app_context
            .engine_unprivileged_state
            .get_project_manager()
            .get_opened_project();
        let did_update_project = match opened_project_lock.write() {
            Ok(mut opened_project) => {
                if let Some(opened_project) = opened_project.as_mut() {
                    let project_info = opened_project.get_project_info_mut();

                    *project_info.get_project_symbol_catalog_mut() = updated_project_symbol_catalog.clone();
                    project_info.set_has_unsaved_changes(true);
                    true
                } else {
                    false
                }
            }
            Err(error) => {
                log::error!("Failed to acquire opened project while persisting symbol struct changes: {}.", error);
                false
            }
        };

        if !did_update_project {
            return;
        }

        let project_save_request = ProjectSaveRequest {};
        project_save_request.send(&self.app_context.engine_unprivileged_state, |project_save_response| {
            if !project_save_response.success {
                log::error!("Failed to save project after applying symbol struct changes.");
            }
        });

        let registry_set_project_symbols_request = RegistrySetProjectSymbolsRequest {
            project_symbol_catalog: updated_project_symbol_catalog,
        };
        let did_dispatch_registry_sync = registry_set_project_symbols_request.send(&self.app_context.engine_unprivileged_state, |_response| {});
        if !did_dispatch_registry_sync {
            log::error!("Failed to dispatch project symbol registry sync after symbol struct changes.");
        }
    }

    fn default_data_type_ref(&self) -> DataTypeRef {
        let registered_data_types = self
            .app_context
            .engine_unprivileged_state
            .get_registered_data_type_refs();

        registered_data_types
            .iter()
            .find(|data_type_ref| data_type_ref.get_data_type_id() == DataTypeI32::DATA_TYPE_ID)
            .cloned()
            .or_else(|| registered_data_types.first().cloned())
            .unwrap_or_else(|| DataTypeRef::new(DataTypeI32::DATA_TYPE_ID))
    }

    fn available_data_types(&self) -> Vec<DataTypeRef> {
        let mut available_data_types = self
            .app_context
            .engine_unprivileged_state
            .get_registered_data_type_refs();
        if available_data_types.is_empty() {
            available_data_types.push(DataTypeRef::new(DataTypeI32::DATA_TYPE_ID));
        }

        available_data_types
    }

    fn string_data_type_ref() -> DataTypeRef {
        DataTypeRef::new(DataTypeStringUtf8::DATA_TYPE_ID)
    }

    fn unsigned_integer_data_type_ref() -> DataTypeRef {
        DataTypeRef::new(DataTypeU64::DATA_TYPE_ID)
    }

    fn render_icon_button(
        &self,
        user_interface: &mut Ui,
        icon_handle: &eframe::egui::TextureHandle,
        tooltip_text: &str,
        is_disabled: bool,
    ) -> Response {
        let theme = &self.app_context.theme;
        let button_response = user_interface.add_sized(
            vec2(Self::ICON_BUTTON_WIDTH, Self::FIELD_ROW_HEIGHT),
            ThemeButton::new_from_theme(theme)
                .with_tooltip_text(tooltip_text)
                .background_color(theme.background_control_secondary)
                .border_color(theme.submenu_border)
                .border_width(1.0)
                .disabled(is_disabled),
        );

        IconDraw::draw_tinted(
            user_interface,
            button_response.rect,
            icon_handle,
            if is_disabled { theme.foreground_preview } else { theme.foreground },
        );

        button_response
    }

    fn render_take_over_header_icon_button(
        &self,
        user_interface: &mut Ui,
        icon_handle: &eframe::egui::TextureHandle,
        tooltip_text: &str,
        is_disabled: bool,
    ) -> Response {
        let theme = &self.app_context.theme;
        let button_response = user_interface.add_sized(
            vec2(Self::ICON_BUTTON_WIDTH, Self::TAKE_OVER_HEADER_HEIGHT),
            ThemeButton::new_from_theme(theme)
                .with_tooltip_text(tooltip_text)
                .background_color(Color32::TRANSPARENT)
                .disabled(is_disabled),
        );

        IconDraw::draw_tinted(
            user_interface,
            button_response.rect,
            icon_handle,
            if is_disabled { theme.foreground_preview } else { theme.foreground },
        );

        button_response
    }

    fn render_string_value_box(
        &self,
        user_interface: &mut Ui,
        value: &mut String,
        preview_text: &str,
        id: &str,
        width: f32,
        height: f32,
    ) {
        let validation_data_type_ref = Self::string_data_type_ref();
        let mut value_string = AnonymousValueString::new(value.clone(), AnonymousValueStringFormat::String, ContainerType::None);

        user_interface.add(
            DataValueBoxView::new(
                self.app_context.clone(),
                &mut value_string,
                &validation_data_type_ref,
                false,
                true,
                preview_text,
                id,
            )
            .allowed_anonymous_value_string_formats(vec![AnonymousValueStringFormat::String])
            .show_format_button(false)
            .normalize_value_format(false)
            .use_format_text_colors(false)
            .width(width)
            .height(height),
        );

        *value = value_string.get_anonymous_value_string().to_string();
    }

    fn render_unsigned_integer_value_box(
        &self,
        user_interface: &mut Ui,
        value: &mut String,
        preview_text: &str,
        id: &str,
        width: f32,
        height: f32,
    ) {
        let validation_data_type_ref = Self::unsigned_integer_data_type_ref();
        let mut value_string = AnonymousValueString::new(value.clone(), AnonymousValueStringFormat::Decimal, ContainerType::None);

        user_interface.add(
            DataValueBoxView::new(
                self.app_context.clone(),
                &mut value_string,
                &validation_data_type_ref,
                false,
                true,
                preview_text,
                id,
            )
            .allowed_anonymous_value_string_formats(vec![AnonymousValueStringFormat::Decimal])
            .show_format_button(false)
            .normalize_value_format(false)
            .use_format_text_colors(false)
            .width(width)
            .height(height),
        );

        *value = value_string.get_anonymous_value_string().to_string();
    }

    fn render_container_kind_selector(
        &self,
        user_interface: &mut Ui,
        container_edit: &mut SymbolStructFieldContainerEdit,
        field_index: usize,
        width: f32,
    ) {
        let selector_id = format!("symbol_struct_editor_container_kind_{}", field_index);
        let current_label = container_edit.kind.label();
        let mut selected_container_kind = None;

        user_interface.add(
            ComboBoxView::new(
                self.app_context.clone(),
                current_label,
                &selector_id,
                None,
                |popup_user_interface: &mut Ui, should_close: &mut bool| {
                    for container_kind in SymbolStructFieldContainerKind::ALL {
                        let container_kind_response =
                            popup_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), container_kind.label(), None, width));

                        if container_kind_response.clicked() {
                            selected_container_kind = Some(container_kind);
                            *should_close = true;
                        }
                    }
                },
            )
            .width(width)
            .height(Self::FIELD_ROW_HEIGHT),
        );

        if let Some(selected_container_kind) = selected_container_kind {
            container_edit.kind = selected_container_kind;
        }
    }

    fn render_dynamic_count_mode_selector(
        &self,
        user_interface: &mut Ui,
        container_edit: &mut SymbolStructFieldContainerEdit,
        field_index: usize,
        width: f32,
    ) {
        let selector_id = format!("symbol_struct_editor_dynamic_count_mode_{}", field_index);
        let current_label = container_edit.dynamic_array_count_mode.label();
        let mut selected_dynamic_count_mode = None;

        user_interface.add(
            ComboBoxView::new(
                self.app_context.clone(),
                current_label,
                &selector_id,
                None,
                |popup_user_interface: &mut Ui, should_close: &mut bool| {
                    for dynamic_count_mode in SymbolStructFieldDynamicCountMode::ALL {
                        let dynamic_count_mode_response =
                            popup_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), dynamic_count_mode.label(), None, width));

                        if dynamic_count_mode_response.clicked() {
                            selected_dynamic_count_mode = Some(dynamic_count_mode);
                            *should_close = true;
                        }
                    }
                },
            )
            .width(width)
            .height(Self::FIELD_ROW_HEIGHT),
        );

        if let Some(selected_dynamic_count_mode) = selected_dynamic_count_mode {
            container_edit.dynamic_array_count_mode = selected_dynamic_count_mode;
        }
    }

    fn render_pointer_size_selector(
        &self,
        user_interface: &mut Ui,
        container_edit: &mut SymbolStructFieldContainerEdit,
        field_index: usize,
        width: f32,
    ) {
        let selector_id = format!("symbol_struct_editor_pointer_size_{}", field_index);
        let current_label = container_edit.pointer_size.to_string();
        let mut selected_pointer_size = None;

        user_interface.add(
            ComboBoxView::new(
                self.app_context.clone(),
                &current_label,
                &selector_id,
                None,
                |popup_user_interface: &mut Ui, should_close: &mut bool| {
                    for pointer_size in PointerScanPointerSize::ALL {
                        let pointer_size_label = pointer_size.to_string();
                        let pointer_size_response = popup_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), &pointer_size_label, None, width));

                        if pointer_size_response.clicked() {
                            selected_pointer_size = Some(pointer_size);
                            *should_close = true;
                        }
                    }
                },
            )
            .width(width)
            .height(Self::FIELD_ROW_HEIGHT),
        );

        if let Some(selected_pointer_size) = selected_pointer_size {
            container_edit.pointer_size = selected_pointer_size;
        }
    }

    fn render_offset_mode_selector(
        &self,
        user_interface: &mut Ui,
        field_draft: &mut SymbolStructFieldEditDraft,
        field_index: usize,
        width: f32,
    ) {
        let selector_id = format!("symbol_struct_editor_offset_mode_{}", field_index);
        let current_label = field_draft.offset_mode.label();
        let mut selected_offset_mode = None;

        user_interface.add(
            ComboBoxView::new(
                self.app_context.clone(),
                current_label,
                &selector_id,
                None,
                |popup_user_interface: &mut Ui, should_close: &mut bool| {
                    for offset_mode in SymbolStructFieldOffsetMode::ALL {
                        let offset_mode_response = popup_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), offset_mode.label(), None, width));

                        if offset_mode_response.clicked() {
                            selected_offset_mode = Some(offset_mode);
                            *should_close = true;
                        }
                    }
                },
            )
            .width(width)
            .height(Self::FIELD_ROW_HEIGHT),
        );

        if let Some(selected_offset_mode) = selected_offset_mode {
            field_draft.offset_mode = selected_offset_mode;
        }
    }

    fn render_text_button(
        &self,
        user_interface: &mut Ui,
        text: &str,
        tooltip_text: &str,
        width: f32,
        height: f32,
    ) -> Response {
        let theme = &self.app_context.theme;
        let response = user_interface.add_sized(
            vec2(width, height),
            ThemeButton::new_from_theme(theme)
                .with_tooltip_text(tooltip_text)
                .border_width(1.0)
                .border_color(theme.background_control_primary_dark),
        );

        user_interface.painter().text(
            response.rect.center(),
            Align2::CENTER_CENTER,
            text,
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );

        response
    }

    fn append_expression_token(
        expression_text: &mut String,
        token: &str,
    ) {
        let trimmed_expression = expression_text.trim_end().to_string();
        let token_is_operator = matches!(token, "+" | "-" | "*" | "/");
        let next_expression = if trimmed_expression.is_empty() {
            token.to_string()
        } else if token_is_operator {
            format!("{} {} ", trimmed_expression, token)
        } else if trimmed_expression.ends_with('(') || token == ")" {
            format!("{}{}", trimmed_expression, token)
        } else {
            format!("{} {}", trimmed_expression, token)
        };

        *expression_text = next_expression;
    }

    fn collect_expression_field_names(
        draft: &SymbolStructLayoutEditDraft,
        selected_field_index: usize,
    ) -> Vec<String> {
        let mut field_names = draft
            .field_drafts
            .iter()
            .enumerate()
            .filter_map(|(field_index, field_draft)| {
                let field_name = field_draft.field_name.trim();

                (!field_name.is_empty() && field_index != selected_field_index).then_some(field_name.to_string())
            })
            .collect::<Vec<_>>();
        field_names.sort();
        field_names.dedup();

        field_names
    }

    fn collect_expression_type_ids(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) -> Vec<String> {
        let mut type_ids = self
            .available_data_types()
            .into_iter()
            .map(|data_type_ref| data_type_ref.get_data_type_id().to_string())
            .chain(
                project_symbol_catalog
                    .get_struct_layout_descriptors()
                    .iter()
                    .map(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id().to_string()),
            )
            .collect::<Vec<_>>();
        type_ids.sort_by_key(|type_id| type_id.to_ascii_lowercase());
        type_ids.dedup_by(|left_type_id, right_type_id| left_type_id.eq_ignore_ascii_case(right_type_id));

        type_ids
    }

    fn render_expression_editor(
        &self,
        user_interface: &mut Ui,
        expression_text: &mut String,
        preview_text: &str,
        id_prefix: &str,
        field_names: &[String],
        type_ids: &[String],
    ) {
        self.render_string_value_box(
            user_interface,
            expression_text,
            preview_text,
            &format!("{}_text", id_prefix),
            user_interface.available_width(),
            Self::FIELD_ROW_HEIGHT,
        );
        user_interface.add_space(Self::FIELD_INPUT_SPACING);

        user_interface.horizontal_wrapped(|user_interface| {
            for operator in ["+", "-", "*", "/", "(", ")"] {
                let operator_response = self.render_text_button(
                    user_interface,
                    operator,
                    "Append this operator to the expression.",
                    Self::FIELD_EXPRESSION_OPERATOR_BUTTON_WIDTH,
                    Self::FIELD_ROW_HEIGHT,
                );

                if operator_response.clicked() {
                    Self::append_expression_token(expression_text, operator);
                }
            }

            let field_picker_width = Self::FIELD_EXPRESSION_PICKER_WIDTH.min(user_interface.available_width().max(1.0));
            let mut selected_field_name = None;
            user_interface.add(
                ComboBoxView::new(
                    self.app_context.clone(),
                    "Field...",
                    &format!("{}_field_picker", id_prefix),
                    None,
                    |popup_user_interface: &mut Ui, should_close: &mut bool| {
                        for field_name in field_names {
                            let field_response =
                                popup_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), field_name, None, field_picker_width));

                            if field_response.clicked() {
                                selected_field_name = Some(field_name.clone());
                                *should_close = true;
                            }
                        }
                    },
                )
                .width(field_picker_width)
                .height(Self::FIELD_ROW_HEIGHT),
            );
            if let Some(selected_field_name) = selected_field_name {
                Self::append_expression_token(expression_text, &selected_field_name);
            }

            let type_picker_width = Self::FIELD_EXPRESSION_PICKER_WIDTH.min(user_interface.available_width().max(1.0));
            let mut selected_type_id = None;
            user_interface.add(
                ComboBoxView::new(
                    self.app_context.clone(),
                    "sizeof...",
                    &format!("{}_sizeof_picker", id_prefix),
                    None,
                    |popup_user_interface: &mut Ui, should_close: &mut bool| {
                        for type_id in type_ids {
                            let type_response = popup_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), type_id, None, type_picker_width));

                            if type_response.clicked() {
                                selected_type_id = Some(type_id.clone());
                                *should_close = true;
                            }
                        }
                    },
                )
                .width(type_picker_width)
                .height(Self::FIELD_ROW_HEIGHT),
            );
            if let Some(selected_type_id) = selected_type_id {
                Self::append_expression_token(expression_text, &format!("sizeof({})", selected_type_id));
            }
        });
    }

    fn render_resolver_picker(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_resolver_id: &mut String,
        id_prefix: &str,
    ) {
        let resolver_descriptors = project_symbol_catalog.get_symbolic_resolver_descriptors();
        if resolver_descriptors.is_empty() {
            user_interface.label(RichText::new("No reusable resolvers yet.").color(self.app_context.theme.foreground_preview));
            return;
        }

        let selected_text = if selected_resolver_id.trim().is_empty() {
            String::from("Pick resolver...")
        } else {
            selected_resolver_id.trim().to_string()
        };

        ComboBox::from_id_salt(id_prefix)
            .selected_text(&selected_text)
            .show_ui(user_interface, |user_interface| {
                for resolver_descriptor in resolver_descriptors {
                    let resolver_id = resolver_descriptor.get_resolver_id();
                    if user_interface
                        .selectable_label(selected_text == resolver_id, resolver_id)
                        .clicked()
                    {
                        *selected_resolver_id = resolver_id.to_string();
                    }
                }
            });
    }

    fn format_field_layout_summary(field_draft: &SymbolStructFieldEditDraft) -> String {
        let shape_text = match field_draft.container_edit.kind {
            SymbolStructFieldContainerKind::Element => String::from("element"),
            SymbolStructFieldContainerKind::Array => String::from("array"),
            SymbolStructFieldContainerKind::FixedArray => {
                let fixed_array_length = field_draft.container_edit.fixed_array_length.trim();

                if fixed_array_length.is_empty() {
                    String::from("fixed array")
                } else {
                    format!("fixed array [{}]", fixed_array_length)
                }
            }
            SymbolStructFieldContainerKind::DynamicArray => match field_draft.container_edit.dynamic_array_count_mode {
                SymbolStructFieldDynamicCountMode::Resolver => {
                    let resolver_id = field_draft
                        .container_edit
                        .dynamic_array_count_resolver_id
                        .trim();

                    if resolver_id.is_empty() {
                        String::from("dynamic array")
                    } else {
                        format!("dynamic array [resolver({})]", resolver_id)
                    }
                }
                SymbolStructFieldDynamicCountMode::Expression => {
                    let count_expression = field_draft.container_edit.dynamic_array_count_expression.trim();

                    if count_expression.is_empty() {
                        String::from("dynamic array")
                    } else {
                        format!("dynamic array [{}]", count_expression)
                    }
                }
            },
            SymbolStructFieldContainerKind::Pointer => format!("pointer ({})", field_draft.container_edit.pointer_size),
        };
        let offset_text = match field_draft.offset_mode {
            SymbolStructFieldOffsetMode::Sequential => String::from("sequential"),
            SymbolStructFieldOffsetMode::Resolver => {
                let resolver_id = field_draft.offset_resolver_id.trim();

                if resolver_id.is_empty() {
                    String::from("resolver offset")
                } else {
                    format!("@ resolver({})", resolver_id)
                }
            }
            SymbolStructFieldOffsetMode::Expression => {
                let offset_expression = field_draft.offset_expression.trim();

                if offset_expression.is_empty() {
                    String::from("expression offset")
                } else {
                    format!("@ {}", offset_expression)
                }
            }
        };

        format!("{} | {}", shape_text, offset_text)
    }

    fn render_struct_layout_row(
        &self,
        user_interface: &mut Ui,
        layout_id: &str,
        field_count: usize,
        usage_count: usize,
        is_selected: bool,
    ) -> Response {
        let theme = &self.app_context.theme;
        let (row_rect, row_response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::LIST_ROW_HEIGHT), Sense::click());

        if is_selected {
            user_interface
                .painter()
                .rect_filled(row_rect, CornerRadius::ZERO, theme.selected_background);
            user_interface
                .painter()
                .rect_stroke(row_rect, CornerRadius::ZERO, Stroke::new(1.0, theme.selected_border), StrokeKind::Inside);
        }

        StateLayer {
            bounds_min: row_rect.min,
            bounds_max: row_rect.max,
            enabled: true,
            pressed: row_response.is_pointer_button_down_on(),
            has_hover: row_response.hovered(),
            has_focus: false,
            corner_radius: CornerRadius::ZERO,
            border_width: 0.0,
            hover_color: theme.hover_tint,
            pressed_color: theme.pressed_tint,
            border_color: theme.background_control_secondary_dark,
            border_color_focused: theme.background_control_secondary_dark,
        }
        .ui(user_interface);

        user_interface.painter().text(
            pos2(row_rect.min.x + 8.0, row_rect.center().y),
            Align2::LEFT_CENTER,
            layout_id,
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );

        user_interface.painter().text(
            pos2(row_rect.max.x - 8.0, row_rect.center().y),
            Align2::RIGHT_CENTER,
            format!("{} fields | {} uses", field_count, usage_count),
            theme.font_library.font_noto_sans.font_normal.clone(),
            if is_selected { theme.foreground } else { theme.foreground_preview },
        );

        row_response
    }

    fn render_list_header(
        &self,
        user_interface: &mut Ui,
    ) {
        let theme = &self.app_context.theme;
        let (header_rect, _) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::LIST_ROW_HEIGHT), Sense::hover());

        user_interface
            .painter()
            .rect_filled(header_rect, CornerRadius::ZERO, theme.background_primary);
        user_interface.painter().text(
            pos2(header_rect.min.x + 8.0, header_rect.center().y),
            Align2::LEFT_CENTER,
            "Struct Layout",
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground_preview,
        );
        user_interface.painter().text(
            pos2(header_rect.max.x - 8.0, header_rect.center().y),
            Align2::RIGHT_CENTER,
            "Fields | Uses",
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground_preview,
        );
    }

    fn render_list_panel(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_layout_id: Option<&str>,
        filter_text: &str,
        is_take_over_active: bool,
    ) {
        let theme = &self.app_context.theme;

        user_interface.horizontal(|user_interface| {
            let new_layout_response = self.render_icon_button(
                user_interface,
                &theme.icon_library.icon_handle_common_add,
                "Create a new reusable struct layout.",
                is_take_over_active,
            );
            if new_layout_response.clicked() {
                SymbolStructEditorViewData::begin_create_struct_layout(
                    self.symbol_struct_editor_view_data.clone(),
                    project_symbol_catalog,
                    self.default_data_type_ref(),
                );
            }

            let can_edit_selected_layout = !is_take_over_active && selected_layout_id.is_some();
            let edit_layout_response = self.render_icon_button(
                user_interface,
                &theme.icon_library.icon_handle_common_edit,
                "Edit the selected struct layout.",
                !can_edit_selected_layout,
            );
            if edit_layout_response.clicked() {
                if let Some(selected_layout_id) = selected_layout_id {
                    SymbolStructEditorViewData::begin_edit_struct_layout(
                        self.symbol_struct_editor_view_data.clone(),
                        project_symbol_catalog,
                        selected_layout_id,
                    );
                }
            }

            let usage_count = selected_layout_id
                .map(|selected_layout_id| SymbolStructEditorViewData::count_symbol_claim_usages(project_symbol_catalog, selected_layout_id))
                .unwrap_or(0);
            let can_delete_selected_layout = !is_take_over_active && selected_layout_id.is_some() && usage_count == 0;
            let delete_layout_response = self.render_icon_button(
                user_interface,
                &theme.icon_library.icon_handle_common_delete,
                "Delete the selected struct layout.",
                !can_delete_selected_layout,
            );
            if delete_layout_response.clicked() {
                if let Some(selected_layout_id) = selected_layout_id {
                    SymbolStructEditorViewData::request_delete_confirmation(self.symbol_struct_editor_view_data.clone(), selected_layout_id.to_string());
                }
            }
        });

        user_interface.add_space(8.0);
        let mut edited_filter_text = filter_text.to_string();
        self.render_string_value_box(
            user_interface,
            &mut edited_filter_text,
            "Filter struct layouts...",
            "symbol_struct_editor_filter_text",
            user_interface.available_width(),
            Self::FIELD_ROW_HEIGHT,
        );
        if edited_filter_text != filter_text {
            SymbolStructEditorViewData::set_filter_text(self.symbol_struct_editor_view_data.clone(), edited_filter_text);
        }

        user_interface.add_space(8.0);
        self.render_list_header(user_interface);
        ScrollArea::vertical()
            .id_salt("symbol_struct_editor_layout_list")
            .auto_shrink([false, false])
            .show(user_interface, |user_interface| {
                for struct_layout_descriptor in project_symbol_catalog
                    .get_struct_layout_descriptors()
                    .iter()
                    .filter(|struct_layout_descriptor| SymbolStructEditorViewData::layout_matches_filter(struct_layout_descriptor, filter_text))
                {
                    let struct_layout_id = struct_layout_descriptor.get_struct_layout_id();
                    let usage_count = SymbolStructEditorViewData::count_symbol_claim_usages(project_symbol_catalog, struct_layout_id);
                    let field_count = struct_layout_descriptor
                        .get_struct_layout_definition()
                        .get_fields()
                        .len();
                    let row_response = self.render_struct_layout_row(
                        user_interface,
                        struct_layout_id,
                        field_count,
                        usage_count,
                        selected_layout_id == Some(struct_layout_id),
                    );
                    if row_response.clicked() {
                        SymbolStructEditorViewData::select_struct_layout(self.symbol_struct_editor_view_data.clone(), Some(struct_layout_id.to_string()));
                    }

                    if row_response.double_clicked() && !is_take_over_active {
                        SymbolStructEditorViewData::begin_edit_struct_layout(
                            self.symbol_struct_editor_view_data.clone(),
                            project_symbol_catalog,
                            struct_layout_id,
                        );
                    }
                }

                if project_symbol_catalog
                    .get_struct_layout_descriptors()
                    .is_empty()
                {
                    user_interface.label(RichText::new("No struct layouts yet.").color(self.app_context.theme.foreground_preview));
                }
            });
    }

    fn render_field_label(
        &self,
        user_interface: &mut Ui,
        label: &str,
    ) {
        let theme = &self.app_context.theme;
        user_interface.label(RichText::new(label).strong().color(theme.foreground));
    }

    fn render_take_over_panel(
        &self,
        user_interface: &mut Ui,
        title: &str,
        header_action_width: f32,
        render_header_actions: impl FnOnce(&mut Ui),
        add_contents: impl FnOnce(&mut Ui),
    ) {
        let theme = &self.app_context.theme;
        let (panel_rect, _) = user_interface.allocate_exact_size(user_interface.available_size(), Sense::hover());
        user_interface
            .painter()
            .rect_filled(panel_rect, CornerRadius::ZERO, theme.background_panel);

        let inner_rect = panel_rect.shrink2(vec2(Self::TAKE_OVER_PADDING_X, Self::TAKE_OVER_PADDING_Y));
        let mut panel_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(inner_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        panel_user_interface.set_clip_rect(inner_rect);

        let (header_rect, _) = panel_user_interface.allocate_exact_size(
            vec2(panel_user_interface.available_width().max(1.0), Self::TAKE_OVER_HEADER_HEIGHT),
            Sense::hover(),
        );
        panel_user_interface
            .painter()
            .rect_filled(header_rect, CornerRadius::ZERO, theme.background_primary);
        let header_inner_rect = header_rect;
        let mut header_user_interface = panel_user_interface.new_child(
            UiBuilder::new()
                .max_rect(header_inner_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        header_user_interface.set_clip_rect(header_inner_rect);

        let title_width = (header_inner_rect.width() - header_action_width - Self::TAKE_OVER_HEADER_TITLE_PADDING_X).max(0.0);
        let (title_rect, _) = header_user_interface.allocate_exact_size(vec2(title_width, Self::TAKE_OVER_HEADER_HEIGHT), Sense::hover());
        header_user_interface.painter().text(
            pos2(title_rect.left() + Self::TAKE_OVER_HEADER_TITLE_PADDING_X, title_rect.center().y),
            Align2::LEFT_CENTER,
            title,
            theme.font_library.font_noto_sans.font_window_title.clone(),
            theme.foreground,
        );

        if header_action_width > 0.0 {
            header_user_interface.allocate_ui_with_layout(
                vec2(header_action_width, Self::TAKE_OVER_HEADER_HEIGHT),
                Layout::right_to_left(Align::Center),
                |user_interface| {
                    render_header_actions(user_interface);
                },
            );
        }

        panel_user_interface.add_space(Self::TAKE_OVER_SECTION_SPACING);
        ScrollArea::vertical()
            .id_salt(format!("symbol_struct_editor_take_over_body_{title}"))
            .auto_shrink([false, false])
            .show(&mut panel_user_interface, |user_interface| {
                let content_width = (user_interface.available_width() - Self::TAKE_OVER_CONTENT_PADDING_X * 2.0).max(0.0);
                user_interface.horizontal(|user_interface| {
                    user_interface.add_space(Self::TAKE_OVER_CONTENT_PADDING_X);
                    user_interface.allocate_ui_with_layout(vec2(content_width, 0.0), Layout::top_down(Align::Min), |user_interface| {
                        add_contents(user_interface);
                    });
                });
            });
    }

    fn render_field_editor_section(
        &self,
        user_interface: &mut Ui,
        field_draft: &mut SymbolStructFieldEditDraft,
        field_index: usize,
        can_remove_field: bool,
        can_move_up: bool,
        can_move_down: bool,
        available_data_types: &[DataTypeRef],
    ) -> Option<SymbolStructFieldRowAction> {
        let theme = &self.app_context.theme;
        let mut pending_field_row_action = None;

        user_interface.allocate_ui_with_layout(vec2(user_interface.available_width(), 0.0), Layout::top_down(Align::Min), |user_interface| {
            user_interface.allocate_ui_with_layout(
                vec2(user_interface.available_width().max(1.0), Self::FIELD_ROW_HEIGHT),
                Layout::left_to_right(Align::Center),
                |user_interface| {
                    user_interface.label(
                        RichText::new(format!("Field {}", field_index + 1))
                            .strong()
                            .color(theme.foreground),
                    );

                    user_interface.allocate_ui_with_layout(
                        vec2(user_interface.available_width().max(0.0), Self::FIELD_ROW_HEIGHT),
                        Layout::right_to_left(Align::Center),
                        |user_interface| {
                            let insert_field_response = self.render_icon_button(
                                user_interface,
                                &theme.icon_library.icon_handle_common_add,
                                "Insert a new field after this one.",
                                false,
                            );
                            if insert_field_response.clicked() {
                                pending_field_row_action = Some(SymbolStructFieldRowAction::InsertAfter);
                            }

                            user_interface.add_space(Self::FIELD_INPUT_SPACING);

                            let remove_field_response = self.render_icon_button(
                                user_interface,
                                &theme.icon_library.icon_handle_common_delete,
                                "Remove this field from the draft struct layout.",
                                !can_remove_field,
                            );
                            if remove_field_response.clicked() {
                                pending_field_row_action = Some(SymbolStructFieldRowAction::RemoveField);
                            }

                            user_interface.add_space(Self::FIELD_INPUT_SPACING);

                            let move_down_response = self.render_icon_button(
                                user_interface,
                                &theme.icon_library.icon_handle_navigation_down_arrow_small,
                                "Move this field down.",
                                !can_move_down,
                            );
                            if move_down_response.clicked() {
                                pending_field_row_action = Some(SymbolStructFieldRowAction::MoveDown);
                            }

                            user_interface.add_space(Self::FIELD_INPUT_SPACING);

                            let move_up_response = self.render_icon_button(
                                user_interface,
                                &theme.icon_library.icon_handle_navigation_up_arrow_small,
                                "Move this field up.",
                                !can_move_up,
                            );
                            if move_up_response.clicked() {
                                pending_field_row_action = Some(SymbolStructFieldRowAction::MoveUp);
                            }

                            user_interface.add_space(Self::FIELD_INPUT_SPACING);

                            let edit_layout_response = self.render_icon_button(
                                user_interface,
                                &theme.icon_library.icon_handle_common_properties,
                                "Edit this field's layout.",
                                false,
                            );
                            if edit_layout_response.clicked() {
                                pending_field_row_action = Some(SymbolStructFieldRowAction::EditLayout);
                            }
                        },
                    );
                },
            );

            user_interface.add_space(Self::FIELD_INPUT_SPACING);
            self.render_string_value_box(
                user_interface,
                &mut field_draft.field_name,
                "field_name",
                &format!("symbol_struct_editor_field_name_{}", field_index),
                user_interface.available_width(),
                Self::FIELD_ROW_HEIGHT,
            );

            let selector_id = format!("symbol_struct_editor_data_type_{}", field_index);
            user_interface.add_space(Self::FIELD_INPUT_SPACING);
            user_interface.allocate_ui_with_layout(
                vec2(user_interface.available_width(), Self::FIELD_ROW_HEIGHT),
                Layout::left_to_right(Align::Center),
                |user_interface| {
                    let available_width = user_interface.available_width().max(0.0);
                    let container_mode_width = Self::FIELD_CONTAINER_MODE_WIDTH.min(available_width);
                    let type_width = (available_width - container_mode_width - Self::FIELD_INPUT_SPACING).max(0.0);

                    user_interface.add_sized(
                        vec2(type_width, Self::FIELD_ROW_HEIGHT),
                        DataTypeSelectorView::new(self.app_context.clone(), &mut field_draft.data_type_selection, &selector_id)
                            .available_data_types(available_data_types.to_vec())
                            .single_select()
                            .stacked_list()
                            .width(type_width)
                            .height(Self::FIELD_ROW_HEIGHT),
                    );

                    user_interface.add_space(Self::FIELD_INPUT_SPACING);
                    self.render_container_kind_selector(user_interface, &mut field_draft.container_edit, field_index, container_mode_width);
                },
            );

            user_interface.add_space(Self::FIELD_INPUT_SPACING);
            user_interface.label(RichText::new(Self::format_field_layout_summary(field_draft)).color(theme.foreground_preview));
        });

        pending_field_row_action
    }

    fn render_field_rows(
        &self,
        user_interface: &mut Ui,
        draft: &mut SymbolStructLayoutEditDraft,
    ) {
        let available_data_types = self.available_data_types();
        let field_count = draft.field_drafts.len();
        let can_remove_field = field_count > 1;
        let mut pending_field_row_action = None;
        for field_index in 0..field_count {
            let Some(field_draft) = draft.field_drafts.get_mut(field_index) else {
                continue;
            };
            let can_move_up = field_index > 0;
            let can_move_down = field_index + 1 < field_count;
            if let Some(field_row_action) = self.render_field_editor_section(
                user_interface,
                field_draft,
                field_index,
                can_remove_field,
                can_move_up,
                can_move_down,
                &available_data_types,
            ) {
                pending_field_row_action = Some((field_index, field_row_action));
            }
            if field_index + 1 < draft.field_drafts.len() {
                user_interface.add_space(Self::FIELD_SECTION_VERTICAL_SPACING);
                user_interface.add_space(Self::FIELD_SECTION_VERTICAL_SPACING);
            }
        }

        if let Some((field_index, field_row_action)) = pending_field_row_action {
            match field_row_action {
                SymbolStructFieldRowAction::InsertAfter => {
                    let insert_index = field_index.saturating_add(1).min(draft.field_drafts.len());
                    draft
                        .field_drafts
                        .insert(insert_index, SymbolStructFieldEditDraft::new(self.default_data_type_ref()));
                }
                SymbolStructFieldRowAction::RemoveField => {
                    draft.field_drafts.remove(field_index);
                    if draft.field_drafts.is_empty() {
                        draft
                            .field_drafts
                            .push(SymbolStructFieldEditDraft::new(self.default_data_type_ref()));
                    }
                }
                SymbolStructFieldRowAction::MoveUp => {
                    if field_index > 0 {
                        draft.field_drafts.swap(field_index, field_index - 1);
                    }
                }
                SymbolStructFieldRowAction::MoveDown => {
                    if field_index + 1 < draft.field_drafts.len() {
                        draft.field_drafts.swap(field_index, field_index + 1);
                    }
                }
                SymbolStructFieldRowAction::EditLayout => {
                    SymbolStructEditorViewData::begin_field_layout_editor(self.symbol_struct_editor_view_data.clone(), field_index);
                }
            }
        }
    }

    fn render_field_layout_take_over(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &mut SymbolStructLayoutEditDraft,
        field_index: usize,
    ) {
        let Some(field_draft) = draft.field_drafts.get(field_index) else {
            SymbolStructEditorViewData::cancel_field_layout_editor(self.symbol_struct_editor_view_data.clone());
            return;
        };
        let field_title = if field_draft.field_name.trim().is_empty() {
            format!("Field {}", field_index + 1)
        } else {
            field_draft.field_name.trim().to_string()
        };
        let field_type_id = field_draft
            .data_type_selection
            .visible_data_type()
            .get_data_type_id()
            .to_string();
        let field_names = Self::collect_expression_field_names(draft, field_index);
        let type_ids = self.collect_expression_type_ids(project_symbol_catalog);
        let mut should_close_field_layout_editor = false;

        self.render_take_over_panel(
            user_interface,
            "Field Layout",
            Self::ICON_BUTTON_WIDTH,
            |user_interface| {
                let back_response = self.render_take_over_header_icon_button(
                    user_interface,
                    &self
                        .app_context
                        .theme
                        .icon_library
                        .icon_handle_navigation_left_arrow,
                    "Return to struct layout editing.",
                    false,
                );
                if back_response.clicked() {
                    should_close_field_layout_editor = true;
                }
            },
            |user_interface| {
                user_interface.add(
                    GroupBox::new_from_theme(&self.app_context.theme, "Field", |user_interface| {
                        user_interface.label(RichText::new(&field_title).color(self.app_context.theme.foreground));
                        user_interface.add_space(4.0);
                        user_interface.label(RichText::new(&field_type_id).color(self.app_context.theme.foreground_preview));
                    })
                    .desired_width(user_interface.available_width()),
                );
                user_interface.add_space(Self::TAKE_OVER_GROUPBOX_SPACING);

                user_interface.add(
                    GroupBox::new_from_theme(&self.app_context.theme, "Shape", |user_interface| {
                        let Some(field_draft) = draft.field_drafts.get_mut(field_index) else {
                            return;
                        };

                        self.render_container_kind_selector(
                            user_interface,
                            &mut field_draft.container_edit,
                            field_index,
                            Self::FIELD_CONTAINER_MODE_WIDTH.min(user_interface.available_width()),
                        );

                        match field_draft.container_edit.kind {
                            SymbolStructFieldContainerKind::Element | SymbolStructFieldContainerKind::Array => {}
                            SymbolStructFieldContainerKind::FixedArray => {
                                user_interface.add_space(Self::FIELD_INPUT_SPACING);
                                self.render_unsigned_integer_value_box(
                                    user_interface,
                                    &mut field_draft.container_edit.fixed_array_length,
                                    "length",
                                    &format!("symbol_struct_editor_field_layout_fixed_array_length_{}", field_index),
                                    Self::FIELD_CONTAINER_DETAIL_WIDTH.min(user_interface.available_width()),
                                    Self::FIELD_ROW_HEIGHT,
                                );
                            }
                            SymbolStructFieldContainerKind::DynamicArray => {
                                user_interface.add_space(Self::FIELD_INPUT_SPACING);
                                self.render_dynamic_count_mode_selector(
                                    user_interface,
                                    &mut field_draft.container_edit,
                                    field_index,
                                    Self::FIELD_CONTAINER_MODE_WIDTH.min(user_interface.available_width()),
                                );
                                user_interface.add_space(Self::FIELD_INPUT_SPACING);
                                match field_draft.container_edit.dynamic_array_count_mode {
                                    SymbolStructFieldDynamicCountMode::Resolver => {
                                        self.render_resolver_picker(
                                            user_interface,
                                            project_symbol_catalog,
                                            &mut field_draft.container_edit.dynamic_array_count_resolver_id,
                                            &format!("symbol_struct_editor_field_layout_count_resolver_{}", field_index),
                                        );
                                    }
                                    SymbolStructFieldDynamicCountMode::Expression => {
                                        self.render_expression_editor(
                                            user_interface,
                                            &mut field_draft.container_edit.dynamic_array_count_expression,
                                            "count expression",
                                            &format!("symbol_struct_editor_field_layout_count_expression_{}", field_index),
                                            &field_names,
                                            &type_ids,
                                        );
                                    }
                                }
                            }
                            SymbolStructFieldContainerKind::Pointer => {
                                user_interface.add_space(Self::FIELD_INPUT_SPACING);
                                self.render_pointer_size_selector(
                                    user_interface,
                                    &mut field_draft.container_edit,
                                    field_index,
                                    Self::FIELD_CONTAINER_DETAIL_WIDTH.min(user_interface.available_width()),
                                );
                            }
                        }
                    })
                    .desired_width(user_interface.available_width()),
                );
                user_interface.add_space(Self::TAKE_OVER_GROUPBOX_SPACING);

                user_interface.add(
                    GroupBox::new_from_theme(&self.app_context.theme, "Offset", |user_interface| {
                        let Some(field_draft) = draft.field_drafts.get_mut(field_index) else {
                            return;
                        };

                        self.render_offset_mode_selector(
                            user_interface,
                            field_draft,
                            field_index,
                            Self::FIELD_OFFSET_MODE_WIDTH.min(user_interface.available_width()),
                        );

                        match field_draft.offset_mode {
                            SymbolStructFieldOffsetMode::Sequential => {}
                            SymbolStructFieldOffsetMode::Resolver => {
                                user_interface.add_space(Self::FIELD_INPUT_SPACING);
                                self.render_resolver_picker(
                                    user_interface,
                                    project_symbol_catalog,
                                    &mut field_draft.offset_resolver_id,
                                    &format!("symbol_struct_editor_field_layout_offset_resolver_{}", field_index),
                                );
                            }
                            SymbolStructFieldOffsetMode::Expression => {
                                user_interface.add_space(Self::FIELD_INPUT_SPACING);
                                self.render_expression_editor(
                                    user_interface,
                                    &mut field_draft.offset_expression,
                                    "offset expression",
                                    &format!("symbol_struct_editor_field_layout_offset_expression_{}", field_index),
                                    &field_names,
                                    &type_ids,
                                );
                            }
                        }
                    })
                    .desired_width(user_interface.available_width()),
                );
                user_interface.add_space(Self::TAKE_OVER_GROUPBOX_SPACING);

                let preview_result =
                    SymbolStructEditorViewData::build_struct_layout_descriptor(project_symbol_catalog, draft).map(|struct_layout_descriptor| {
                        struct_layout_descriptor
                            .get_struct_layout_definition()
                            .get_fields()
                            .get(field_index)
                            .map(SymbolicFieldDefinition::to_string)
                            .unwrap_or_default()
                    });

                user_interface.add(
                    GroupBox::new_from_theme(&self.app_context.theme, "Preview", |user_interface| match preview_result {
                        Ok(preview_text) => {
                            user_interface.label(RichText::new(preview_text).color(self.app_context.theme.foreground_preview));
                        }
                        Err(error) => {
                            user_interface.label(RichText::new(error).color(self.app_context.theme.error_red));
                        }
                    })
                    .desired_width(user_interface.available_width()),
                );
            },
        );

        if should_close_field_layout_editor {
            SymbolStructEditorViewData::cancel_field_layout_editor(self.symbol_struct_editor_view_data.clone());
        }
    }

    fn render_struct_layout_take_over(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        take_over_title: &str,
        baseline_draft: Option<&SymbolStructLayoutEditDraft>,
        draft: Option<&SymbolStructLayoutEditDraft>,
        field_layout_editor_index: Option<usize>,
    ) {
        let Some(draft) = draft else {
            return;
        };
        let baseline_draft = baseline_draft.unwrap_or(draft);

        let mut edited_draft = draft.clone();
        if let Some(field_layout_editor_index) = field_layout_editor_index {
            self.render_field_layout_take_over(user_interface, project_symbol_catalog, &mut edited_draft, field_layout_editor_index);
            SymbolStructEditorViewData::update_draft(self.symbol_struct_editor_view_data.clone(), edited_draft);
            return;
        }

        let validation_result = SymbolStructEditorViewData::build_struct_layout_descriptor(project_symbol_catalog, &edited_draft);
        let usage_count = edited_draft
            .original_layout_id
            .as_deref()
            .map(|selected_layout_id| SymbolStructEditorViewData::count_symbol_claim_usages(project_symbol_catalog, selected_layout_id))
            .unwrap_or(0);
        let has_unsaved_changes = edited_draft != *baseline_draft;
        let is_creating_new_layout = edited_draft.original_layout_id.is_none();
        let can_save = validation_result.is_ok() && has_unsaved_changes;
        let save_tooltip = if !has_unsaved_changes {
            "No struct layout changes to save."
        } else if validation_result.is_err() {
            "Fix validation errors before saving this struct layout."
        } else if is_creating_new_layout {
            "Save this new struct layout."
        } else {
            "Save these struct layout changes."
        };
        let mut should_cancel_take_over = false;
        let mut should_save_draft = false;

        self.render_take_over_panel(
            user_interface,
            take_over_title,
            Self::ICON_BUTTON_WIDTH * 2.0,
            |user_interface| {
                let save_response = self.render_take_over_header_icon_button(
                    user_interface,
                    &self.app_context.theme.icon_library.icon_handle_file_system_save,
                    save_tooltip,
                    !can_save,
                );
                if save_response.clicked() {
                    should_save_draft = true;
                }

                let cancel_response = self.render_take_over_header_icon_button(
                    user_interface,
                    &self
                        .app_context
                        .theme
                        .icon_library
                        .icon_handle_navigation_cancel,
                    "Cancel struct layout editing.",
                    false,
                );
                if cancel_response.clicked() {
                    should_cancel_take_over = true;
                }
            },
            |user_interface| {
                user_interface.add(
                    GroupBox::new_from_theme(&self.app_context.theme, "Struct Layout", |user_interface| {
                        self.render_field_label(user_interface, "Struct Layout Id");
                        self.render_string_value_box(
                            user_interface,
                            &mut edited_draft.layout_id,
                            "module.type",
                            "symbol_struct_editor_layout_id",
                            user_interface.available_width(),
                            Self::FIELD_ROW_HEIGHT,
                        );
                        user_interface.add_space(6.0);

                        let status_text = if is_creating_new_layout {
                            String::from("Creating a new reusable struct layout.")
                        } else if usage_count == 0 {
                            String::from("Not used by any symbol claims yet.")
                        } else if usage_count == 1 {
                            String::from("Used by 1 symbol claim.")
                        } else {
                            format!("Used by {} symbol claims.", usage_count)
                        };
                        user_interface.label(RichText::new(status_text).color(self.app_context.theme.foreground_preview));
                    })
                    .desired_width(user_interface.available_width()),
                );
                user_interface.add_space(Self::TAKE_OVER_GROUPBOX_SPACING);

                user_interface.add(
                    GroupBox::new_from_theme(&self.app_context.theme, "Fields", |user_interface| {
                        self.render_field_rows(user_interface, &mut edited_draft);
                    })
                    .desired_width(user_interface.available_width()),
                );
                user_interface.add_space(Self::TAKE_OVER_SECTION_SPACING);

                if let Err(validation_error) = validation_result.as_ref() {
                    user_interface.label(RichText::new(validation_error).color(self.app_context.theme.error_red));
                    user_interface.add_space(8.0);
                }
            },
        );

        if should_cancel_take_over {
            SymbolStructEditorViewData::cancel_take_over_state(self.symbol_struct_editor_view_data.clone());
            return;
        }

        if should_save_draft {
            match SymbolStructEditorViewData::apply_draft_to_catalog(project_symbol_catalog, &edited_draft) {
                Ok(updated_project_symbol_catalog) => {
                    self.persist_project_symbol_catalog(updated_project_symbol_catalog.clone());
                    SymbolStructEditorViewData::select_struct_layout(
                        self.symbol_struct_editor_view_data.clone(),
                        Some(edited_draft.layout_id.trim().to_string()),
                    );
                    SymbolStructEditorViewData::cancel_take_over_state(self.symbol_struct_editor_view_data.clone());
                    return;
                }
                Err(error) => {
                    log::error!("Failed to apply symbol struct draft: {}.", error);
                }
            }
        }

        SymbolStructEditorViewData::update_draft(self.symbol_struct_editor_view_data.clone(), edited_draft);
    }

    fn render_delete_confirmation_take_over(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        layout_id: &str,
    ) {
        let usage_count = SymbolStructEditorViewData::count_symbol_claim_usages(project_symbol_catalog, layout_id);

        let can_delete_layout = usage_count == 0;
        let mut should_cancel_take_over = false;
        let mut should_delete_layout = false;

        self.render_take_over_panel(
            user_interface,
            "Delete Struct Layout",
            Self::ICON_BUTTON_WIDTH * 2.0,
            |user_interface| {
                let delete_response = self.render_take_over_header_icon_button(
                    user_interface,
                    &self.app_context.theme.icon_library.icon_handle_common_delete,
                    "Delete the selected struct layout.",
                    !can_delete_layout,
                );
                if delete_response.clicked() {
                    should_delete_layout = true;
                }

                let cancel_response = self.render_take_over_header_icon_button(
                    user_interface,
                    &self
                        .app_context
                        .theme
                        .icon_library
                        .icon_handle_navigation_cancel,
                    "Cancel struct layout deletion.",
                    false,
                );
                if cancel_response.clicked() {
                    should_cancel_take_over = true;
                }
            },
            |user_interface| {
                let theme = &self.app_context.theme;
                user_interface.add(
                    GroupBox::new_from_theme(theme, "Confirmation", |user_interface| {
                        user_interface.label(RichText::new(format!("Delete `{}`?", layout_id)).color(theme.foreground));
                        user_interface.add_space(4.0);
                        user_interface.label(RichText::new(format!("{} symbol claim uses.", usage_count)).color(theme.foreground_preview));
                    })
                    .desired_width(user_interface.available_width()),
                );
            },
        );

        if should_cancel_take_over {
            SymbolStructEditorViewData::cancel_take_over_state(self.symbol_struct_editor_view_data.clone());
            return;
        }

        if should_delete_layout {
            match SymbolStructEditorViewData::remove_struct_layout_from_catalog(project_symbol_catalog, layout_id) {
                Ok(updated_project_symbol_catalog) => {
                    self.persist_project_symbol_catalog(updated_project_symbol_catalog);
                    SymbolStructEditorViewData::cancel_take_over_state(self.symbol_struct_editor_view_data.clone());
                }
                Err(error) => {
                    log::error!("Failed to delete struct layout: {}.", error);
                }
            }
        }
    }
}

impl Widget for SymbolStructEditorView {
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
                            .label(RichText::new("Open a project to author reusable struct layouts.").color(self.app_context.theme.foreground_preview));
                    },
                )
                .response;
        };

        SymbolStructEditorViewData::synchronize(self.symbol_struct_editor_view_data.clone(), &project_symbol_catalog);
        let (selected_layout_id, filter_text, take_over_state, baseline_draft, draft, field_layout_editor_index) = self
            .symbol_struct_editor_view_data
            .read("SymbolStructEditor view")
            .map(|symbol_struct_editor_view_data| {
                (
                    symbol_struct_editor_view_data
                        .get_selected_layout_id()
                        .map(str::to_string),
                    symbol_struct_editor_view_data.get_filter_text().to_string(),
                    symbol_struct_editor_view_data.get_take_over_state().cloned(),
                    symbol_struct_editor_view_data.get_baseline_draft().cloned(),
                    symbol_struct_editor_view_data.get_draft().cloned(),
                    symbol_struct_editor_view_data.get_field_layout_editor_index(),
                )
            })
            .unwrap_or((None, String::new(), None, None, None, None));
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
            if field_layout_editor_index.is_some() {
                SymbolStructEditorViewData::cancel_field_layout_editor(self.symbol_struct_editor_view_data.clone());
            } else {
                SymbolStructEditorViewData::cancel_take_over_state(self.symbol_struct_editor_view_data.clone());
            }
        }

        if !is_take_over_active && can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::Enter)) {
            if let Some(selected_layout_id) = selected_layout_id.as_deref() {
                SymbolStructEditorViewData::begin_edit_struct_layout(self.symbol_struct_editor_view_data.clone(), &project_symbol_catalog, selected_layout_id);
            }
        }

        if !is_take_over_active && can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::Delete)) {
            if let Some(selected_layout_id) = selected_layout_id.as_deref() {
                let usage_count = SymbolStructEditorViewData::count_symbol_claim_usages(&project_symbol_catalog, selected_layout_id);
                if usage_count == 0 {
                    SymbolStructEditorViewData::request_delete_confirmation(self.symbol_struct_editor_view_data.clone(), selected_layout_id.to_string());
                }
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
                    Some(SymbolStructEditorTakeOverState::CreateStructLayout) => {
                        self.render_struct_layout_take_over(
                            &mut content_user_interface,
                            &project_symbol_catalog,
                            "New Struct Layout",
                            baseline_draft.as_ref(),
                            draft.as_ref(),
                            field_layout_editor_index,
                        );
                    }
                    Some(SymbolStructEditorTakeOverState::EditStructLayout { .. }) => {
                        self.render_struct_layout_take_over(
                            &mut content_user_interface,
                            &project_symbol_catalog,
                            "Edit Struct Layout",
                            baseline_draft.as_ref(),
                            draft.as_ref(),
                            field_layout_editor_index,
                        );
                    }
                    Some(SymbolStructEditorTakeOverState::DeleteConfirmation { layout_id }) => {
                        self.render_delete_confirmation_take_over(&mut content_user_interface, &project_symbol_catalog, layout_id);
                    }
                    None => {
                        self.render_list_panel(
                            &mut content_user_interface,
                            &project_symbol_catalog,
                            selected_layout_id.as_deref(),
                            &filter_text,
                            false,
                        );
                    }
                }
            })
            .response
    }
}
