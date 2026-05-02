use crate::app_context::AppContext;
use crate::ui::draw::icon_draw::IconDraw;
use crate::ui::widgets::controls::combo_box::{combo_box_item_view::ComboBoxItemView, combo_box_view::ComboBoxView};
use crate::ui::widgets::controls::data_type_selector::{data_type_selection::DataTypeSelection, data_type_selector_view::DataTypeSelectorView};
use crate::ui::widgets::controls::{
    button::Button as ThemeButton, data_value_box::data_value_box_view::DataValueBoxView, groupbox::GroupBox, state_layer::StateLayer,
};
use crate::views::struct_editor::view_data::struct_editor_view_data::{
    StructEditorTakeOverState, StructEditorViewData, StructFieldEditDraft, StructLayoutEditDraft,
};
use crate::views::struct_editor::view_data::struct_field_container_edit::{StructFieldContainerEdit, StructFieldContainerKind};
use eframe::egui::{Align, Align2, Direction, Key, Layout, Response, RichText, ScrollArea, Sense, Stroke, Ui, UiBuilder, Widget, pos2, vec2};
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
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StructFieldRowAction {
    InsertAfter,
    RemoveField,
    MoveUp,
    MoveDown,
}

#[derive(Clone)]
pub struct StructEditorView {
    app_context: Arc<AppContext>,
    struct_editor_view_data: Dependency<StructEditorViewData>,
}

impl StructEditorView {
    pub const WINDOW_ID: &'static str = "window_struct_editor";
    const FIELD_ROW_HEIGHT: f32 = 28.0;
    const LIST_ROW_HEIGHT: f32 = 28.0;
    const ICON_BUTTON_WIDTH: f32 = 36.0;
    const FIELD_SECTION_VERTICAL_SPACING: f32 = 10.0;
    const FIELD_INPUT_SPACING: f32 = 8.0;
    const FIELD_CONTAINER_MODE_WIDTH: f32 = 160.0;
    const FIELD_CONTAINER_DETAIL_WIDTH: f32 = 140.0;
    const TAKE_OVER_HEADER_HEIGHT: f32 = 32.0;
    const TAKE_OVER_PADDING_X: f32 = 12.0;
    const TAKE_OVER_PADDING_Y: f32 = 8.0;
    const TAKE_OVER_SECTION_SPACING: f32 = 12.0;
    const TAKE_OVER_GROUPBOX_SPACING: f32 = 8.0;

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let struct_editor_view_data = app_context
            .dependency_container
            .register(StructEditorViewData::new());

        Self {
            app_context,
            struct_editor_view_data,
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
                log::error!("Failed to acquire opened project while persisting struct editor changes: {}.", error);
                false
            }
        };

        if !did_update_project {
            return;
        }

        let project_save_request = ProjectSaveRequest {};
        project_save_request.send(&self.app_context.engine_unprivileged_state, |project_save_response| {
            if !project_save_response.success {
                log::error!("Failed to save project after applying struct editor changes.");
            }
        });

        let registry_set_project_symbols_request = RegistrySetProjectSymbolsRequest {
            project_symbol_catalog: updated_project_symbol_catalog,
        };
        let did_dispatch_registry_sync = registry_set_project_symbols_request.send(&self.app_context.engine_unprivileged_state, |_response| {});
        if !did_dispatch_registry_sync {
            log::error!("Failed to dispatch project symbol registry sync after struct editor changes.");
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
        container_edit: &mut StructFieldContainerEdit,
        field_index: usize,
        width: f32,
    ) {
        let selector_id = format!("struct_editor_container_kind_{}", field_index);
        let current_label = container_edit.kind.label();
        let mut selected_container_kind = None;

        user_interface.add(
            ComboBoxView::new(
                self.app_context.clone(),
                current_label,
                &selector_id,
                None,
                |popup_user_interface: &mut Ui, should_close: &mut bool| {
                    for container_kind in StructFieldContainerKind::ALL {
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

    fn render_pointer_size_selector(
        &self,
        user_interface: &mut Ui,
        container_edit: &mut StructFieldContainerEdit,
        field_index: usize,
        width: f32,
    ) {
        let selector_id = format!("struct_editor_pointer_size_{}", field_index);
        let current_label = container_edit.pointer_size.to_string();
        let mut selected_pointer_size = None;

        user_interface.add(
            ComboBoxView::new(
                self.app_context.clone(),
                &current_label,
                &selector_id,
                None,
                |popup_user_interface: &mut Ui, should_close: &mut bool| {
                    for pointer_size in [
                        PointerScanPointerSize::Pointer24,
                        PointerScanPointerSize::Pointer24be,
                        PointerScanPointerSize::Pointer32,
                        PointerScanPointerSize::Pointer32be,
                        PointerScanPointerSize::Pointer64,
                        PointerScanPointerSize::Pointer64be,
                    ] {
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
                StructEditorViewData::begin_create_struct_layout(self.struct_editor_view_data.clone(), project_symbol_catalog, self.default_data_type_ref());
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
                    StructEditorViewData::begin_edit_struct_layout(self.struct_editor_view_data.clone(), project_symbol_catalog, selected_layout_id);
                }
            }

            let usage_count = selected_layout_id
                .map(|selected_layout_id| StructEditorViewData::count_symbol_claim_usages(project_symbol_catalog, selected_layout_id))
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
                    StructEditorViewData::request_delete_confirmation(self.struct_editor_view_data.clone(), selected_layout_id.to_string());
                }
            }
        });

        user_interface.add_space(8.0);
        let mut edited_filter_text = filter_text.to_string();
        self.render_string_value_box(
            user_interface,
            &mut edited_filter_text,
            "Filter struct layouts...",
            "struct_editor_filter_text",
            user_interface.available_width(),
            Self::FIELD_ROW_HEIGHT,
        );
        if edited_filter_text != filter_text {
            StructEditorViewData::set_filter_text(self.struct_editor_view_data.clone(), edited_filter_text);
        }

        user_interface.add_space(8.0);
        self.render_list_header(user_interface);
        ScrollArea::vertical()
            .id_salt("struct_editor_layout_list")
            .auto_shrink([false, false])
            .show(user_interface, |user_interface| {
                for struct_layout_descriptor in project_symbol_catalog
                    .get_struct_layout_descriptors()
                    .iter()
                    .filter(|struct_layout_descriptor| StructEditorViewData::layout_matches_filter(struct_layout_descriptor, filter_text))
                {
                    let struct_layout_id = struct_layout_descriptor.get_struct_layout_id();
                    let usage_count = StructEditorViewData::count_symbol_claim_usages(project_symbol_catalog, struct_layout_id);
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
                        StructEditorViewData::select_struct_layout(self.struct_editor_view_data.clone(), Some(struct_layout_id.to_string()));
                    }

                    if row_response.double_clicked() && !is_take_over_active {
                        StructEditorViewData::begin_edit_struct_layout(self.struct_editor_view_data.clone(), project_symbol_catalog, struct_layout_id);
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
        panel_user_interface
            .painter()
            .rect_stroke(header_rect, CornerRadius::ZERO, Stroke::new(1.0, theme.submenu_border), StrokeKind::Inside);
        let header_inner_rect = header_rect.shrink2(vec2(8.0, 0.0));
        let mut header_user_interface = panel_user_interface.new_child(
            UiBuilder::new()
                .max_rect(header_inner_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        header_user_interface.set_clip_rect(header_inner_rect);

        let title_width = (header_inner_rect.width() - header_action_width).max(0.0);
        let (title_rect, _) = header_user_interface.allocate_exact_size(vec2(title_width, Self::TAKE_OVER_HEADER_HEIGHT), Sense::hover());
        header_user_interface.painter().text(
            title_rect.left_center(),
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
            .id_salt(format!("struct_editor_take_over_body_{title}"))
            .auto_shrink([false, false])
            .show(&mut panel_user_interface, |user_interface| {
                add_contents(user_interface);
            });
    }

    fn render_field_editor_section(
        &self,
        user_interface: &mut Ui,
        field_draft: &mut StructFieldEditDraft,
        field_index: usize,
        can_remove_field: bool,
        can_move_up: bool,
        can_move_down: bool,
        available_data_types: &[DataTypeRef],
    ) -> Option<StructFieldRowAction> {
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
                                pending_field_row_action = Some(StructFieldRowAction::InsertAfter);
                            }

                            user_interface.add_space(Self::FIELD_INPUT_SPACING);

                            let remove_field_response = self.render_icon_button(
                                user_interface,
                                &theme.icon_library.icon_handle_common_delete,
                                "Remove this field from the draft struct layout.",
                                !can_remove_field,
                            );
                            if remove_field_response.clicked() {
                                pending_field_row_action = Some(StructFieldRowAction::RemoveField);
                            }

                            user_interface.add_space(Self::FIELD_INPUT_SPACING);

                            let move_down_response = self.render_icon_button(
                                user_interface,
                                &theme.icon_library.icon_handle_navigation_down_arrow_small,
                                "Move this field down.",
                                !can_move_down,
                            );
                            if move_down_response.clicked() {
                                pending_field_row_action = Some(StructFieldRowAction::MoveDown);
                            }

                            user_interface.add_space(Self::FIELD_INPUT_SPACING);

                            let move_up_response = self.render_icon_button(
                                user_interface,
                                &theme.icon_library.icon_handle_navigation_up_arrow_small,
                                "Move this field up.",
                                !can_move_up,
                            );
                            if move_up_response.clicked() {
                                pending_field_row_action = Some(StructFieldRowAction::MoveUp);
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
                &format!("struct_editor_field_name_{}", field_index),
                user_interface.available_width(),
                Self::FIELD_ROW_HEIGHT,
            );

            let selector_id = format!("struct_editor_data_type_{}", field_index);
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

            match field_draft.container_edit.kind {
                StructFieldContainerKind::Element | StructFieldContainerKind::Array => {}
                StructFieldContainerKind::FixedArray => {
                    user_interface.add_space(Self::FIELD_INPUT_SPACING);
                    self.render_unsigned_integer_value_box(
                        user_interface,
                        &mut field_draft.container_edit.fixed_array_length,
                        "length",
                        &format!("struct_editor_fixed_array_length_{}", field_index),
                        Self::FIELD_CONTAINER_DETAIL_WIDTH.min(user_interface.available_width()),
                        Self::FIELD_ROW_HEIGHT,
                    );
                }
                StructFieldContainerKind::Pointer => {
                    user_interface.add_space(Self::FIELD_INPUT_SPACING);
                    self.render_pointer_size_selector(
                        user_interface,
                        &mut field_draft.container_edit,
                        field_index,
                        Self::FIELD_CONTAINER_DETAIL_WIDTH.min(user_interface.available_width()),
                    );
                }
            }
        });

        pending_field_row_action
    }

    fn render_field_rows(
        &self,
        user_interface: &mut Ui,
        draft: &mut StructLayoutEditDraft,
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
                user_interface.separator();
                user_interface.add_space(Self::FIELD_SECTION_VERTICAL_SPACING);
            }
        }

        if let Some((field_index, field_row_action)) = pending_field_row_action {
            match field_row_action {
                StructFieldRowAction::InsertAfter => {
                    let insert_index = field_index.saturating_add(1).min(draft.field_drafts.len());
                    draft.field_drafts.insert(
                        insert_index,
                        StructFieldEditDraft {
                            field_name: String::new(),
                            data_type_selection: DataTypeSelection::new(self.default_data_type_ref()),
                            container_edit: StructFieldContainerEdit::default(),
                        },
                    );
                }
                StructFieldRowAction::RemoveField => {
                    draft.field_drafts.remove(field_index);
                    if draft.field_drafts.is_empty() {
                        draft.field_drafts.push(StructFieldEditDraft {
                            field_name: String::new(),
                            data_type_selection: DataTypeSelection::new(self.default_data_type_ref()),
                            container_edit: StructFieldContainerEdit::default(),
                        });
                    }
                }
                StructFieldRowAction::MoveUp => {
                    if field_index > 0 {
                        draft.field_drafts.swap(field_index, field_index - 1);
                    }
                }
                StructFieldRowAction::MoveDown => {
                    if field_index + 1 < draft.field_drafts.len() {
                        draft.field_drafts.swap(field_index, field_index + 1);
                    }
                }
            }
        }
    }

    fn render_struct_layout_take_over(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        take_over_title: &str,
        baseline_draft: Option<&StructLayoutEditDraft>,
        draft: Option<&StructLayoutEditDraft>,
    ) {
        let Some(draft) = draft else {
            return;
        };
        let baseline_draft = baseline_draft.unwrap_or(draft);

        let mut edited_draft = draft.clone();
        let validation_result = StructEditorViewData::build_struct_layout_descriptor(project_symbol_catalog, &edited_draft);
        let usage_count = edited_draft
            .original_layout_id
            .as_deref()
            .map(|selected_layout_id| StructEditorViewData::count_symbol_claim_usages(project_symbol_catalog, selected_layout_id))
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
                            "struct_editor_layout_id",
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
            StructEditorViewData::cancel_take_over_state(self.struct_editor_view_data.clone());
            return;
        }

        if should_save_draft {
            match StructEditorViewData::apply_draft_to_catalog(project_symbol_catalog, &edited_draft) {
                Ok(updated_project_symbol_catalog) => {
                    self.persist_project_symbol_catalog(updated_project_symbol_catalog.clone());
                    StructEditorViewData::select_struct_layout(self.struct_editor_view_data.clone(), Some(edited_draft.layout_id.trim().to_string()));
                    StructEditorViewData::cancel_take_over_state(self.struct_editor_view_data.clone());
                    return;
                }
                Err(error) => {
                    log::error!("Failed to apply struct editor draft: {}.", error);
                }
            }
        }

        StructEditorViewData::update_draft(self.struct_editor_view_data.clone(), edited_draft);
    }

    fn render_delete_confirmation_take_over(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        layout_id: &str,
    ) {
        let usage_count = StructEditorViewData::count_symbol_claim_usages(project_symbol_catalog, layout_id);

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
            StructEditorViewData::cancel_take_over_state(self.struct_editor_view_data.clone());
            return;
        }

        if should_delete_layout {
            match StructEditorViewData::remove_struct_layout_from_catalog(project_symbol_catalog, layout_id) {
                Ok(updated_project_symbol_catalog) => {
                    self.persist_project_symbol_catalog(updated_project_symbol_catalog);
                    StructEditorViewData::cancel_take_over_state(self.struct_editor_view_data.clone());
                }
                Err(error) => {
                    log::error!("Failed to delete struct layout: {}.", error);
                }
            }
        }
    }
}

impl Widget for StructEditorView {
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

        StructEditorViewData::synchronize(self.struct_editor_view_data.clone(), &project_symbol_catalog);
        let (selected_layout_id, filter_text, take_over_state, baseline_draft, draft) = self
            .struct_editor_view_data
            .read("Struct editor view")
            .map(|struct_editor_view_data| {
                (
                    struct_editor_view_data
                        .get_selected_layout_id()
                        .map(str::to_string),
                    struct_editor_view_data.get_filter_text().to_string(),
                    struct_editor_view_data.get_take_over_state().cloned(),
                    struct_editor_view_data.get_baseline_draft().cloned(),
                    struct_editor_view_data.get_draft().cloned(),
                )
            })
            .unwrap_or((None, String::new(), None, None, None));
        let is_take_over_active = take_over_state.is_some();

        if user_interface.input(|input_state| input_state.key_pressed(Key::Escape)) && is_take_over_active {
            StructEditorViewData::cancel_take_over_state(self.struct_editor_view_data.clone());
        }

        if !is_take_over_active && user_interface.input(|input_state| input_state.key_pressed(Key::Enter)) {
            if let Some(selected_layout_id) = selected_layout_id.as_deref() {
                StructEditorViewData::begin_edit_struct_layout(self.struct_editor_view_data.clone(), &project_symbol_catalog, selected_layout_id);
            }
        }

        if !is_take_over_active && user_interface.input(|input_state| input_state.key_pressed(Key::Delete)) {
            if let Some(selected_layout_id) = selected_layout_id.as_deref() {
                let usage_count = StructEditorViewData::count_symbol_claim_usages(&project_symbol_catalog, selected_layout_id);
                if usage_count == 0 {
                    StructEditorViewData::request_delete_confirmation(self.struct_editor_view_data.clone(), selected_layout_id.to_string());
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
                    Some(StructEditorTakeOverState::CreateStructLayout) => {
                        self.render_struct_layout_take_over(
                            &mut content_user_interface,
                            &project_symbol_catalog,
                            "New Struct Layout",
                            baseline_draft.as_ref(),
                            draft.as_ref(),
                        );
                    }
                    Some(StructEditorTakeOverState::EditStructLayout { .. }) => {
                        self.render_struct_layout_take_over(
                            &mut content_user_interface,
                            &project_symbol_catalog,
                            "Edit Struct Layout",
                            baseline_draft.as_ref(),
                            draft.as_ref(),
                        );
                    }
                    Some(StructEditorTakeOverState::DeleteConfirmation { layout_id }) => {
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
