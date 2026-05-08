use crate::app_context::AppContext;
use crate::ui::draw::icon_draw::IconDraw;
use crate::ui::list_navigation::ListNavigationDirection;
use crate::ui::widgets::controls::{
    button::Button as ThemeButton, data_value_box::data_value_box_view::DataValueBoxView, groupbox::GroupBox, state_layer::StateLayer,
};
use crate::views::struct_viewer::view_data::{struct_viewer_focus_target::StructViewerFocusTarget, struct_viewer_view_data::StructViewerViewData};
use crate::views::symbol_struct_editor::view_data::symbol_struct_editor_view_data::{
    SymbolStructEditorTakeOverState, SymbolStructEditorViewData, SymbolStructFieldEditDraft, SymbolStructFieldOffsetMode, SymbolStructLayoutEditDraft,
};
use crate::views::symbol_struct_editor::view_data::symbol_struct_field_container_edit::SymbolStructFieldContainerKind;
use eframe::egui::{
    Align, Align2, Button as EguiButton, Direction, Key, Layout, Response, RichText, ScrollArea, Sense, Stroke, Ui, UiBuilder, Widget, pos2, vec2,
};
use epaint::{Color32, CornerRadius, Rect, StrokeKind};
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
    structs::{valued_struct::ValuedStruct, valued_struct_field::ValuedStructField},
};
use std::{str::FromStr, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SymbolStructFieldRowAction {
    InsertAfter,
    RemoveField,
    MoveUp,
    MoveDown,
    SelectField,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SymbolStructLayoutRowAction {
    Select,
    Open,
    Rename,
}

#[derive(Clone)]
pub struct SymbolStructEditorView {
    app_context: Arc<AppContext>,
    symbol_struct_editor_view_data: Dependency<SymbolStructEditorViewData>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
}

impl SymbolStructEditorView {
    pub const WINDOW_ID: &'static str = "window_symbol_struct_editor";
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
    const TAKE_OVER_BOTTOM_PADDING: f32 = 8.0;
    const TAKE_OVER_ACTION_BUTTON_WIDTH: f32 = 120.0;
    const TAKE_OVER_ACTION_BUTTON_SPACING: f32 = 12.0;
    const FIELD_ROW_LEFT_PADDING: f32 = 8.0;
    const FIELD_ROW_PREVIEW_GAP: f32 = 12.0;

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let symbol_struct_editor_view_data = app_context
            .dependency_container
            .register(SymbolStructEditorViewData::new());
        let struct_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<StructViewerViewData>();

        Self {
            app_context,
            symbol_struct_editor_view_data,
            struct_viewer_view_data,
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

    fn delete_struct_layout(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        layout_id: &str,
    ) {
        match SymbolStructEditorViewData::remove_struct_layout_from_catalog(project_symbol_catalog, layout_id) {
            Ok(updated_project_symbol_catalog) => {
                self.persist_project_symbol_catalog(updated_project_symbol_catalog);
                SymbolStructEditorViewData::cancel_take_over_state(self.symbol_struct_editor_view_data.clone());
                self.clear_struct_viewer_if_symbol_struct_focused();
            }
            Err(error) => {
                log::error!("Failed to delete struct layout: {}.", error);
            }
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

    fn string_data_type_ref() -> DataTypeRef {
        DataTypeRef::new(DataTypeStringUtf8::DATA_TYPE_ID)
    }

    fn render_icon_button(
        &self,
        user_interface: &mut Ui,
        icon_handle: &eframe::egui::TextureHandle,
        tooltip_text: &str,
        is_disabled: bool,
    ) -> Response {
        let theme = &self.app_context.theme;

        self.render_icon_button_with_style(
            user_interface,
            icon_handle,
            tooltip_text,
            theme.background_control_secondary,
            theme.submenu_border,
            is_disabled,
        )
    }

    fn render_icon_button_with_style(
        &self,
        user_interface: &mut Ui,
        icon_handle: &eframe::egui::TextureHandle,
        tooltip_text: &str,
        background_color: Color32,
        border_color: Color32,
        is_disabled: bool,
    ) -> Response {
        let theme = &self.app_context.theme;
        let button_response = user_interface.add_sized(
            vec2(Self::ICON_BUTTON_WIDTH, Self::FIELD_ROW_HEIGHT),
            ThemeButton::new_from_theme(theme)
                .with_tooltip_text(tooltip_text)
                .background_color(background_color)
                .border_color(border_color)
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

    fn render_flat_icon_button_at(
        &self,
        user_interface: &mut Ui,
        button_rect: Rect,
        icon_handle: &eframe::egui::TextureHandle,
        tooltip_text: &str,
        is_disabled: bool,
    ) -> Response {
        let theme = &self.app_context.theme;
        let button_response = user_interface.put(
            button_rect,
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

    fn render_take_over_action_buttons(
        &self,
        user_interface: &mut Ui,
        accept_label: &str,
        can_accept: bool,
    ) -> (Response, Response) {
        let theme = &self.app_context.theme;
        let button_size = vec2(Self::TAKE_OVER_ACTION_BUTTON_WIDTH, Self::FIELD_ROW_HEIGHT);
        let total_button_width = button_size.x * 2.0 + Self::TAKE_OVER_ACTION_BUTTON_SPACING;
        let side_spacing = ((user_interface.available_width() - total_button_width) * 0.5).max(0.0);

        let responses = user_interface
            .horizontal(|user_interface| {
                user_interface.add_space(side_spacing);
                user_interface.spacing_mut().item_spacing.x = Self::TAKE_OVER_ACTION_BUTTON_SPACING;

                let cancel_response = user_interface.add_sized(
                    button_size,
                    EguiButton::new(RichText::new("Cancel").color(theme.foreground))
                        .fill(theme.background_control_danger)
                        .stroke(Stroke::new(1.0, theme.background_control_danger_dark)),
                );

                let accept_button = EguiButton::new(RichText::new(accept_label).color(if can_accept { theme.foreground } else { theme.foreground_preview }))
                    .fill(if can_accept {
                        theme.background_control_primary
                    } else {
                        theme.background_control_secondary
                    })
                    .stroke(Stroke::new(
                        1.0,
                        if can_accept {
                            theme.background_control_primary_dark
                        } else {
                            theme.background_control_secondary_dark
                        },
                    ));
                let accept_response = user_interface
                    .add_enabled_ui(can_accept, |user_interface| user_interface.add_sized(button_size, accept_button))
                    .inner;

                (cancel_response, accept_response)
            })
            .inner;

        user_interface.add_space(Self::TAKE_OVER_BOTTOM_PADDING);

        responses
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

    fn clear_struct_viewer_if_symbol_struct_focused(&self) {
        let is_symbol_struct_focused = self
            .struct_viewer_view_data
            .read("SymbolStructEditor check details focus")
            .and_then(|struct_viewer_view_data| struct_viewer_view_data.get_focus_target().cloned())
            .is_some_and(|focus_target| matches!(focus_target, StructViewerFocusTarget::SymbolStructEditor { .. }));

        if is_symbol_struct_focused {
            StructViewerViewData::clear_focus(self.struct_viewer_view_data.clone());
        }
    }

    fn focus_selected_layout_in_struct_viewer(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_layout_id: Option<&str>,
    ) {
        let Some(selected_layout_id) = selected_layout_id else {
            self.clear_struct_viewer_if_symbol_struct_focused();
            return;
        };
        let Some(struct_layout_descriptor) = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == selected_layout_id)
        else {
            self.clear_struct_viewer_if_symbol_struct_focused();
            return;
        };

        let details_struct = ValuedStruct::new_anonymous(vec![
            DataTypeStringUtf8::get_value_from_primitive_string(struct_layout_descriptor.get_struct_layout_id())
                .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_STRUCT_LAYOUT_ID.to_string(), false),
        ]);
        let selection_key = format!("layout|{}", struct_layout_descriptor.get_struct_layout_id());

        StructViewerViewData::focus_valued_struct_with_focus_target(
            self.struct_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            details_struct,
            Arc::new(|_edited_field: ValuedStructField| {}),
            Some(StructViewerFocusTarget::SymbolStructEditor { selection_key }),
        );
    }

    fn focus_field_in_struct_viewer(
        &self,
        draft: &SymbolStructLayoutEditDraft,
        field_index: usize,
    ) {
        let Some(field_draft) = draft.field_drafts.get(field_index) else {
            self.clear_struct_viewer_if_symbol_struct_focused();
            return;
        };

        let details_struct = Self::build_field_details_struct(field_draft);
        let selection_key = format!("field|{}|{}", draft.layout_id, field_index);
        let edit_callback = Self::build_struct_viewer_field_edit_callback(
            self.symbol_struct_editor_view_data.clone(),
            self.struct_viewer_view_data.clone(),
            self.app_context.clone(),
            field_index,
        );

        StructViewerViewData::focus_valued_struct_with_focus_target(
            self.struct_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            details_struct,
            edit_callback,
            Some(StructViewerFocusTarget::SymbolStructEditor { selection_key }),
        );
    }

    fn build_field_details_struct(field_draft: &SymbolStructFieldEditDraft) -> ValuedStruct {
        let mut fields = vec![
            DataTypeStringUtf8::get_value_from_primitive_string(&field_draft.field_name)
                .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_STRUCT_FIELD_NAME.to_string(), false),
            DataTypeStringUtf8::get_value_from_primitive_string(
                field_draft
                    .data_type_selection
                    .visible_data_type()
                    .get_data_type_id(),
            )
            .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_STRUCT_FIELD_DATA_TYPE.to_string(), false),
            DataTypeStringUtf8::get_value_from_primitive_string(field_draft.container_edit.kind.label())
                .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_STRUCT_FIELD_CONTAINER_KIND.to_string(), false),
        ];

        match field_draft.container_edit.kind {
            SymbolStructFieldContainerKind::FixedArray => {
                let length = field_draft
                    .container_edit
                    .fixed_array_length
                    .trim()
                    .parse::<u64>()
                    .unwrap_or(1);
                fields.push(
                    DataTypeU64::get_value_from_primitive(length)
                        .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_STRUCT_FIELD_FIXED_ARRAY_LENGTH.to_string(), false),
                );
            }
            SymbolStructFieldContainerKind::DynamicArray => {
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(&field_draft.container_edit.dynamic_array_count_resolver_id)
                        .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_STRUCT_FIELD_COUNT_RESOLVER.to_string(), false),
                );
            }
            SymbolStructFieldContainerKind::Pointer => {
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(&field_draft.container_edit.pointer_size.to_string())
                        .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_STRUCT_FIELD_POINTER_SIZE.to_string(), false),
                );
            }
            SymbolStructFieldContainerKind::Element | SymbolStructFieldContainerKind::Array => {}
        }

        fields.push(
            DataTypeStringUtf8::get_value_from_primitive_string(field_draft.offset_mode.label())
                .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_STRUCT_FIELD_OFFSET_MODE.to_string(), false),
        );
        if field_draft.offset_mode == SymbolStructFieldOffsetMode::Resolver {
            fields.push(
                DataTypeStringUtf8::get_value_from_primitive_string(&field_draft.offset_resolver_id)
                    .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_STRUCT_FIELD_OFFSET_RESOLVER.to_string(), false),
            );
        }

        ValuedStruct::new_anonymous(fields)
    }

    fn build_struct_viewer_field_edit_callback(
        symbol_struct_editor_view_data: Dependency<SymbolStructEditorViewData>,
        struct_viewer_view_data: Dependency<StructViewerViewData>,
        app_context: Arc<AppContext>,
        field_index: usize,
    ) -> Arc<dyn Fn(ValuedStructField) + Send + Sync> {
        Arc::new(move |edited_field: ValuedStructField| {
            let updated_draft = {
                let Some(mut view_data) = symbol_struct_editor_view_data.write("SymbolStructEditor apply field details edit") else {
                    return;
                };
                let Some(mut draft) = view_data.get_draft().cloned() else {
                    return;
                };
                let Some(field_draft) = draft.field_drafts.get_mut(field_index) else {
                    return;
                };

                Self::apply_field_details_edit(field_draft, &edited_field);
                view_data.replace_draft(draft.clone());
                draft
            };

            let Some(updated_field_draft) = updated_draft.field_drafts.get(field_index) else {
                return;
            };
            let details_struct = Self::build_field_details_struct(updated_field_draft);
            let selection_key = format!("field|{}|{}", updated_draft.layout_id, field_index);
            let edit_callback = Self::build_struct_viewer_field_edit_callback(
                symbol_struct_editor_view_data.clone(),
                struct_viewer_view_data.clone(),
                app_context.clone(),
                field_index,
            );

            StructViewerViewData::focus_valued_struct_with_focus_target(
                struct_viewer_view_data.clone(),
                app_context.engine_unprivileged_state.clone(),
                details_struct,
                edit_callback,
                Some(StructViewerFocusTarget::SymbolStructEditor { selection_key }),
            );
        })
    }

    fn apply_field_details_edit(
        field_draft: &mut SymbolStructFieldEditDraft,
        edited_field: &ValuedStructField,
    ) {
        let edited_text = StructViewerViewData::read_utf8_field_text(edited_field);

        match edited_field.get_name() {
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_STRUCT_FIELD_NAME => {
                field_draft.field_name = edited_text;
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_STRUCT_FIELD_DATA_TYPE => {
                field_draft
                    .data_type_selection
                    .replace_selected_data_types(vec![DataTypeRef::new(edited_text.trim())]);
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_STRUCT_FIELD_CONTAINER_KIND => {
                if let Some(container_kind) = Self::container_kind_from_label(&edited_text) {
                    field_draft.container_edit.kind = container_kind;
                }
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_STRUCT_FIELD_FIXED_ARRAY_LENGTH => {
                if let Some(length) = Self::read_u64_field_value(edited_field) {
                    field_draft.container_edit.fixed_array_length = length.max(1).to_string();
                }
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_STRUCT_FIELD_COUNT_RESOLVER => {
                field_draft.container_edit.dynamic_array_count_resolver_id = edited_text;
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_STRUCT_FIELD_POINTER_SIZE => {
                if let Ok(pointer_size) = PointerScanPointerSize::from_str(edited_text.trim()) {
                    field_draft.container_edit.pointer_size = pointer_size;
                }
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_STRUCT_FIELD_OFFSET_MODE => {
                if let Some(offset_mode) = Self::offset_mode_from_label(&edited_text) {
                    field_draft.offset_mode = offset_mode;
                }
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_STRUCT_FIELD_OFFSET_RESOLVER => {
                field_draft.offset_resolver_id = edited_text;
            }
            _ => {}
        }
    }

    fn container_kind_from_label(label: &str) -> Option<SymbolStructFieldContainerKind> {
        SymbolStructFieldContainerKind::ALL
            .iter()
            .copied()
            .find(|container_kind| container_kind.label() == label)
    }

    fn offset_mode_from_label(label: &str) -> Option<SymbolStructFieldOffsetMode> {
        SymbolStructFieldOffsetMode::ALL
            .iter()
            .copied()
            .find(|offset_mode| offset_mode.label() == label)
    }

    fn read_u64_field_value(valued_struct_field: &ValuedStructField) -> Option<u64> {
        let value_bytes = valued_struct_field.get_data_value()?.get_value_bytes();
        let value_bytes: [u8; 8] = value_bytes.as_slice().try_into().ok()?;

        Some(u64::from_le_bytes(value_bytes))
    }

    fn measure_text_width(
        user_interface: &Ui,
        text: &str,
        font_id: &eframe::egui::FontId,
        text_color: Color32,
    ) -> f32 {
        if text.is_empty() {
            return 0.0;
        }

        user_interface.ctx().fonts_mut(|fonts| {
            fonts
                .layout_no_wrap(text.to_string(), font_id.clone(), text_color)
                .size()
                .x
        })
    }

    fn truncate_text_to_width(
        user_interface: &Ui,
        text: &str,
        max_text_width: f32,
        font_id: &eframe::egui::FontId,
        text_color: Color32,
    ) -> String {
        if text.is_empty() || max_text_width <= 0.0 {
            return String::new();
        }

        let full_text_width = Self::measure_text_width(user_interface, text, font_id, text_color);
        if full_text_width <= max_text_width {
            return text.to_string();
        }

        let ellipsis = "...";
        let ellipsis_width = Self::measure_text_width(user_interface, ellipsis, font_id, text_color);
        if ellipsis_width > max_text_width {
            return String::new();
        }

        let mut truncated_text = text.to_string();
        while !truncated_text.is_empty() {
            truncated_text.pop();
            let candidate_text = format!("{}{}", truncated_text, ellipsis);
            let candidate_width = Self::measure_text_width(user_interface, &candidate_text, font_id, text_color);
            if candidate_width <= max_text_width {
                return candidate_text;
            }
        }

        String::new()
    }

    fn render_struct_layout_row(
        &self,
        user_interface: &mut Ui,
        layout_id: &str,
        field_count: usize,
        usage_count: usize,
        is_selected: bool,
    ) -> Option<SymbolStructLayoutRowAction> {
        let theme = &self.app_context.theme;
        let (row_rect, row_response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::LIST_ROW_HEIGHT), Sense::click());
        let mut row_action = None;

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

        let mut row_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(row_rect)
                .layout(Layout::right_to_left(Align::Center)),
        );
        row_user_interface.set_clip_rect(row_rect);

        let rename_response = self.render_icon_button(
            &mut row_user_interface,
            &theme.icon_library.icon_handle_common_edit,
            "Rename this struct layout.",
            false,
        );
        if rename_response.clicked() {
            row_action = Some(SymbolStructLayoutRowAction::Rename);
        }

        row_user_interface.add_space(Self::FIELD_INPUT_SPACING);
        row_user_interface.label(RichText::new(format!("{} fields | {} uses", field_count, usage_count)).color(if is_selected {
            theme.foreground
        } else {
            theme.foreground_preview
        }));

        if row_response.double_clicked() && row_action.is_none() {
            row_action = Some(SymbolStructLayoutRowAction::Open);
        } else if row_response.clicked() && row_action.is_none() {
            row_action = Some(SymbolStructLayoutRowAction::Select);
        }

        row_action
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
                    let row_action = self.render_struct_layout_row(
                        user_interface,
                        struct_layout_id,
                        field_count,
                        usage_count,
                        selected_layout_id == Some(struct_layout_id),
                    );
                    match row_action {
                        Some(SymbolStructLayoutRowAction::Select) => {
                            SymbolStructEditorViewData::select_struct_layout(self.symbol_struct_editor_view_data.clone(), Some(struct_layout_id.to_string()));
                            self.focus_selected_layout_in_struct_viewer(project_symbol_catalog, Some(struct_layout_id));
                        }
                        Some(SymbolStructLayoutRowAction::Open) if !is_take_over_active => {
                            SymbolStructEditorViewData::begin_open_struct_layout(
                                self.symbol_struct_editor_view_data.clone(),
                                project_symbol_catalog,
                                struct_layout_id,
                            );
                        }
                        Some(SymbolStructLayoutRowAction::Rename) if !is_take_over_active => {
                            SymbolStructEditorViewData::begin_rename_struct_layout(
                                self.symbol_struct_editor_view_data.clone(),
                                project_symbol_catalog,
                                struct_layout_id,
                            );
                        }
                        _ => {}
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
                        user_interface.add_space(Self::TAKE_OVER_BOTTOM_PADDING);
                    });
                });
            });
    }

    fn render_field_editor_section(
        &self,
        user_interface: &mut Ui,
        field_draft: &mut SymbolStructFieldEditDraft,
        field_index: usize,
        is_selected: bool,
        can_remove_field: bool,
        can_move_up: bool,
        can_move_down: bool,
    ) -> Option<SymbolStructFieldRowAction> {
        let theme = &self.app_context.theme;
        let mut pending_field_row_action = None;

        let (row_rect, row_response) =
            user_interface.allocate_exact_size(vec2(user_interface.available_width().max(1.0), Self::LIST_ROW_HEIGHT), Sense::click());
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
            has_focus: row_response.has_focus(),
            corner_radius: CornerRadius::ZERO,
            border_width: 0.0,
            hover_color: theme.hover_tint,
            pressed_color: theme.pressed_tint,
            border_color: theme.background_control_secondary_dark,
            border_color_focused: theme.background_control_secondary_dark,
        }
        .ui(user_interface);

        let button_area_width = Self::ICON_BUTTON_WIDTH * 4.0;
        let button_area_left = (row_rect.max.x - button_area_width).max(row_rect.min.x);
        let mut button_min_x = button_area_left;
        let mut render_next_button = |icon_handle: &eframe::egui::TextureHandle, tooltip_text: &str, is_disabled: bool| -> Response {
            let button_rect = Rect::from_min_size(pos2(button_min_x, row_rect.min.y), vec2(Self::ICON_BUTTON_WIDTH, Self::LIST_ROW_HEIGHT));
            button_min_x += Self::ICON_BUTTON_WIDTH;

            self.render_flat_icon_button_at(user_interface, button_rect, icon_handle, tooltip_text, is_disabled)
        };

        let move_up_response = render_next_button(&theme.icon_library.icon_handle_navigation_up_arrow_small, "Move this field up.", !can_move_up);
        if move_up_response.clicked() {
            pending_field_row_action = Some(SymbolStructFieldRowAction::MoveUp);
        }

        let move_down_response = render_next_button(
            &theme.icon_library.icon_handle_navigation_down_arrow_small,
            "Move this field down.",
            !can_move_down,
        );
        if move_down_response.clicked() {
            pending_field_row_action = Some(SymbolStructFieldRowAction::MoveDown);
        }

        let remove_field_response = render_next_button(
            &theme.icon_library.icon_handle_common_delete,
            "Remove this field from the draft struct layout.",
            !can_remove_field,
        );
        if remove_field_response.clicked() {
            pending_field_row_action = Some(SymbolStructFieldRowAction::RemoveField);
        }

        let insert_field_response = render_next_button(&theme.icon_library.icon_handle_common_add, "Insert a new field after this one.", false);
        if insert_field_response.clicked() {
            pending_field_row_action = Some(SymbolStructFieldRowAction::InsertAfter);
        }

        let field_name = if field_draft.field_name.trim().is_empty() {
            format!("Field {}", field_index + 1)
        } else {
            field_draft.field_name.trim().to_string()
        };
        let preview_text = field_draft
            .data_type_selection
            .visible_data_type()
            .get_data_type_id();
        let preview_right = button_area_left - Self::FIELD_ROW_LEFT_PADDING;
        let label_position = pos2(row_rect.min.x + Self::FIELD_ROW_LEFT_PADDING, row_rect.center().y);
        let preview_max_width = (preview_right - label_position.x - Self::FIELD_ROW_PREVIEW_GAP).max(0.0);
        let preview_text = Self::truncate_text_to_width(
            user_interface,
            preview_text,
            preview_max_width,
            &theme.font_library.font_noto_sans.font_small,
            theme.foreground_preview,
        );
        let preview_width = Self::measure_text_width(
            user_interface,
            &preview_text,
            &theme.font_library.font_noto_sans.font_small,
            theme.foreground_preview,
        );
        let label_max_width = (preview_right - preview_width - Self::FIELD_ROW_PREVIEW_GAP - label_position.x).max(0.0);
        let label_text = Self::truncate_text_to_width(
            user_interface,
            &field_name,
            label_max_width,
            &theme.font_library.font_noto_sans.font_normal,
            theme.foreground,
        );
        user_interface.painter().text(
            label_position,
            Align2::LEFT_CENTER,
            label_text,
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );

        if !preview_text.is_empty() {
            user_interface.painter().text(
                pos2(preview_right, row_rect.center().y),
                Align2::RIGHT_CENTER,
                preview_text,
                theme.font_library.font_noto_sans.font_small.clone(),
                theme.foreground_preview,
            );
        }

        if row_response.clicked() && pending_field_row_action.is_none() {
            pending_field_row_action = Some(SymbolStructFieldRowAction::SelectField);
        }

        pending_field_row_action
    }

    fn render_field_rows(
        &self,
        user_interface: &mut Ui,
        draft: &mut SymbolStructLayoutEditDraft,
        selected_field_index: Option<usize>,
    ) {
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
                selected_field_index == Some(field_index),
                can_remove_field,
                can_move_up,
                can_move_down,
            ) {
                pending_field_row_action = Some((field_index, field_row_action));
            }
        }

        if let Some((field_index, field_row_action)) = pending_field_row_action {
            let mut field_index_to_focus = None;
            match field_row_action {
                SymbolStructFieldRowAction::InsertAfter => {
                    let insert_index = field_index.saturating_add(1).min(draft.field_drafts.len());
                    draft
                        .field_drafts
                        .insert(insert_index, SymbolStructFieldEditDraft::new(self.default_data_type_ref()));
                    field_index_to_focus = Some(insert_index);
                }
                SymbolStructFieldRowAction::RemoveField => {
                    draft.field_drafts.remove(field_index);
                    if draft.field_drafts.is_empty() {
                        draft
                            .field_drafts
                            .push(SymbolStructFieldEditDraft::new(self.default_data_type_ref()));
                    }
                    field_index_to_focus = Some(field_index.min(draft.field_drafts.len().saturating_sub(1)));
                }
                SymbolStructFieldRowAction::MoveUp => {
                    if field_index > 0 {
                        draft.field_drafts.swap(field_index, field_index - 1);
                        field_index_to_focus = Some(field_index - 1);
                    }
                }
                SymbolStructFieldRowAction::MoveDown => {
                    if field_index + 1 < draft.field_drafts.len() {
                        draft.field_drafts.swap(field_index, field_index + 1);
                        field_index_to_focus = Some(field_index + 1);
                    }
                }
                SymbolStructFieldRowAction::SelectField => {
                    field_index_to_focus = Some(field_index);
                }
            }

            if let Some(field_index_to_focus) = field_index_to_focus {
                SymbolStructEditorViewData::select_field(self.symbol_struct_editor_view_data.clone(), field_index_to_focus);
                self.focus_field_in_struct_viewer(draft, field_index_to_focus);
            }
        }
    }

    fn render_struct_layout_take_over(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        take_over_title: &str,
        baseline_draft: Option<&SymbolStructLayoutEditDraft>,
        draft: Option<&SymbolStructLayoutEditDraft>,
        selected_field_index: Option<usize>,
        show_layout_name_editor: bool,
    ) {
        let Some(draft) = draft else {
            return;
        };
        let baseline_draft = baseline_draft.unwrap_or(draft);

        let mut edited_draft = draft.clone();
        let validation_result = SymbolStructEditorViewData::build_struct_layout_descriptor(project_symbol_catalog, &edited_draft);
        let usage_count = edited_draft
            .original_layout_id
            .as_deref()
            .map(|selected_layout_id| SymbolStructEditorViewData::count_symbol_claim_usages(project_symbol_catalog, selected_layout_id))
            .unwrap_or(0);
        let has_unsaved_changes = edited_draft != *baseline_draft;
        let is_creating_new_layout = edited_draft.original_layout_id.is_none();
        let can_save = validation_result.is_ok() && has_unsaved_changes;
        let mut should_cancel_take_over = false;
        let mut should_save_draft = false;

        self.render_take_over_panel(
            user_interface,
            take_over_title,
            0.0,
            |_user_interface| {},
            |user_interface| {
                if show_layout_name_editor {
                    user_interface.add(
                        GroupBox::new_from_theme(&self.app_context.theme, "Struct Layout", |user_interface| {
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
                } else {
                    self.render_field_rows(user_interface, &mut edited_draft, selected_field_index);
                    user_interface.add_space(Self::TAKE_OVER_SECTION_SPACING);
                }

                if let Err(validation_error) = validation_result.as_ref() {
                    user_interface.label(RichText::new(validation_error).color(self.app_context.theme.error_red));
                    user_interface.add_space(8.0);
                }

                user_interface.add_space(Self::TAKE_OVER_SECTION_SPACING);
                let (cancel_response, accept_response) = self.render_take_over_action_buttons(user_interface, "Accept", can_save);
                if cancel_response.clicked() {
                    should_cancel_take_over = true;
                }
                if accept_response.clicked() {
                    should_save_draft = true;
                }
            },
        );

        if should_cancel_take_over {
            SymbolStructEditorViewData::cancel_take_over_state(self.symbol_struct_editor_view_data.clone());
            self.clear_struct_viewer_if_symbol_struct_focused();
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
                    self.clear_struct_viewer_if_symbol_struct_focused();
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

        let mut should_cancel_take_over = false;
        let mut should_delete_layout = false;
        let can_handle_window_shortcuts = self
            .app_context
            .window_focus_manager
            .can_window_handle_shortcuts(user_interface.ctx(), Self::WINDOW_ID);

        if can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::Enter)) {
            should_delete_layout = true;
        }

        self.render_take_over_panel(
            user_interface,
            "Delete Struct Layout",
            0.0,
            |_user_interface| {},
            |user_interface| {
                let theme = &self.app_context.theme;
                user_interface.add(
                    GroupBox::new_from_theme(theme, "Confirmation", |user_interface| {
                        user_interface.label(RichText::new(format!("Delete `{}`?", layout_id)).color(theme.foreground));
                        user_interface.add_space(4.0);
                        let (usage_text, usage_text_color) = if usage_count == 0 {
                            (String::from("No existing references will be changed."), theme.foreground_preview)
                        } else {
                            (format!("{} existing references will be changed to raw u8.", usage_count), theme.warning)
                        };
                        user_interface.label(RichText::new(usage_text).color(usage_text_color));
                    })
                    .desired_width(user_interface.available_width()),
                );

                user_interface.add_space(Self::TAKE_OVER_SECTION_SPACING);
                let (cancel_response, accept_response) = self.render_take_over_action_buttons(user_interface, "Accept", true);
                if cancel_response.clicked() {
                    should_cancel_take_over = true;
                }
                if accept_response.clicked() {
                    should_delete_layout = true;
                }
            },
        );

        if should_cancel_take_over {
            SymbolStructEditorViewData::cancel_take_over_state(self.symbol_struct_editor_view_data.clone());
            return;
        }

        if should_delete_layout {
            self.delete_struct_layout(project_symbol_catalog, layout_id);
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
        let (selected_layout_id, filter_text, take_over_state, baseline_draft, draft, selected_field_index) = self
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
                    symbol_struct_editor_view_data.get_selected_field_index(),
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
            SymbolStructEditorViewData::cancel_take_over_state(self.symbol_struct_editor_view_data.clone());
            self.clear_struct_viewer_if_symbol_struct_focused();
        }

        if !is_take_over_active && can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::Enter)) {
            if let Some(selected_layout_id) = selected_layout_id.as_deref() {
                SymbolStructEditorViewData::begin_open_struct_layout(self.symbol_struct_editor_view_data.clone(), &project_symbol_catalog, selected_layout_id);
            }
        }

        if !is_take_over_active && can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::ArrowUp)) {
            let next_layout_id = SymbolStructEditorViewData::navigate_struct_layout_selection(
                self.symbol_struct_editor_view_data.clone(),
                &project_symbol_catalog,
                ListNavigationDirection::Up,
            );
            self.focus_selected_layout_in_struct_viewer(&project_symbol_catalog, next_layout_id.as_deref());
        }

        if !is_take_over_active && can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::ArrowDown)) {
            let next_layout_id = SymbolStructEditorViewData::navigate_struct_layout_selection(
                self.symbol_struct_editor_view_data.clone(),
                &project_symbol_catalog,
                ListNavigationDirection::Down,
            );
            self.focus_selected_layout_in_struct_viewer(&project_symbol_catalog, next_layout_id.as_deref());
        }

        if !is_take_over_active && can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::Delete)) {
            if let Some(selected_layout_id) = selected_layout_id.as_deref() {
                SymbolStructEditorViewData::request_delete_confirmation(self.symbol_struct_editor_view_data.clone(), selected_layout_id.to_string());
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
                            selected_field_index,
                            true,
                        );
                    }
                    Some(SymbolStructEditorTakeOverState::RenameStructLayout { .. }) => {
                        self.render_struct_layout_take_over(
                            &mut content_user_interface,
                            &project_symbol_catalog,
                            "Rename Struct Layout",
                            baseline_draft.as_ref(),
                            draft.as_ref(),
                            selected_field_index,
                            true,
                        );
                    }
                    Some(SymbolStructEditorTakeOverState::OpenStructLayout { .. }) => {
                        self.render_struct_layout_take_over(
                            &mut content_user_interface,
                            &project_symbol_catalog,
                            "Edit Struct Layout",
                            baseline_draft.as_ref(),
                            draft.as_ref(),
                            selected_field_index,
                            false,
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
