use crate::views::struct_viewer::struct_viewer_entry_view::StructViewerEntryView;
use crate::views::struct_viewer::view_data::struct_viewer_field_presentation::{StructViewerFieldEditorKind, StructViewerFieldPresentation};
use crate::views::struct_viewer::view_data::struct_viewer_frame_action::StructViewerFrameAction;
use crate::{
    app_context::AppContext,
    ui::{
        draw::icon_draw::IconDraw,
        geometry::safe_clamp_f32,
        widgets::controls::{button::Button, data_value_box::data_value_box_view::DataValueBoxView, groupbox::GroupBox},
    },
    views::{
        code_viewer::{code_viewer_view::CodeViewerView, view_data::code_viewer_view_data::CodeViewerViewData},
        memory_viewer::{memory_viewer_view::MemoryViewerView, view_data::memory_viewer_view_data::MemoryViewerViewData},
        struct_viewer::view_data::{struct_viewer_take_over_state::StructViewerTakeOverState, struct_viewer_view_data::StructViewerViewData},
    },
};
use eframe::egui::{Align, Align2, CursorIcon, Id, Key, Layout, Response, RichText, ScrollArea, Sense, Ui, UiBuilder, Widget, vec2};
use epaint::{CornerRadius, Rect, pos2};
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::{
    engine::engine_execution_context::EngineExecutionContext,
    structures::{
        data_types::{
            built_in_types::{i64::data_type_i64::DataTypeI64, string::utf8::data_type_string_utf8::DataTypeStringUtf8},
            data_type_ref::DataTypeRef,
        },
        data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
        memory::pointer::Pointer,
        pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
        projects::project_items::built_in_types::{project_item_type_address::ProjectItemTypeAddress, project_item_type_pointer::ProjectItemTypePointer},
        structs::{symbolic_field_definition::SymbolicFieldDefinition, valued_struct::ValuedStruct, valued_struct_field::ValuedStructField},
    },
};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Clone)]
pub struct StructViewerView {
    app_context: Arc<AppContext>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
    code_viewer_view_data: Dependency<CodeViewerViewData>,
    memory_viewer_view_data: Dependency<MemoryViewerViewData>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PointerOffsetRowAction {
    AppendOffset,
    RemoveOffset,
}

impl StructViewerView {
    pub const WINDOW_ID: &'static str = "window_struct_viewer";
    const POINTER_OFFSET_FIELD_ROW_HEIGHT: f32 = 28.0;
    const POINTER_OFFSET_ICON_BUTTON_WIDTH: f32 = 36.0;
    const POINTER_OFFSET_INPUT_SPACING: f32 = 8.0;
    const POINTER_OFFSET_SECTION_VERTICAL_SPACING: f32 = 10.0;
    const POINTER_OFFSET_VALUE_BOX_WIDTH: f32 = 120.0;
    const TAKE_OVER_HEADER_HEIGHT: f32 = 32.0;
    const TAKE_OVER_PADDING_X: f32 = 0.0;
    const TAKE_OVER_PADDING_Y: f32 = 0.0;
    const TAKE_OVER_CONTENT_PADDING_X: f32 = 12.0;
    const TAKE_OVER_HEADER_TITLE_PADDING_X: f32 = 8.0;
    const TAKE_OVER_SECTION_SPACING: f32 = 12.0;

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let struct_viewer_view_data = if app_context
            .dependency_container
            .get_existing::<StructViewerViewData>()
            .is_ok()
        {
            app_context
                .dependency_container
                .get_dependency::<StructViewerViewData>()
        } else {
            app_context
                .dependency_container
                .register(StructViewerViewData::new())
        };
        let code_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<CodeViewerViewData>();
        let memory_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<MemoryViewerViewData>();

        Self {
            app_context: app_context.clone(),
            struct_viewer_view_data,
            code_viewer_view_data,
            memory_viewer_view_data,
        }
    }

    fn resolve_memory_viewer_target_for_field(
        &self,
        field_name: &str,
    ) -> Option<(u64, String, u64)> {
        let struct_viewer_view_data = self
            .struct_viewer_view_data
            .read("Struct viewer resolve memory viewer target")?;
        let source_struct_under_view = struct_viewer_view_data
            .source_struct_under_view
            .as_ref()
            .as_ref()?;

        if field_name != ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE && field_name != ProjectItemTypePointer::PROPERTY_FREEZE_DISPLAY_VALUE {
            return None;
        }

        let symbolic_field_definition = StructViewerViewData::read_symbolic_field_definition_reference_from_field_set(
            source_struct_under_view.get_field(ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE)?,
        )?;

        if !matches!(
            symbolic_field_definition.get_container_type(),
            ContainerType::Array | ContainerType::ArrayFixed(_)
        ) {
            return None;
        }

        let selection_byte_count = Self::resolve_symbolic_field_definition_byte_count(&self.app_context.engine_unprivileged_state, &symbolic_field_definition)?;
        let engine_execution_context: Arc<dyn EngineExecutionContext> = self.app_context.engine_unprivileged_state.clone();

        Self::resolve_struct_runtime_value_target(&engine_execution_context, source_struct_under_view)
            .map(|(address, module_name)| (address, module_name, selection_byte_count.max(1)))
    }

    fn pointer_offsets_edit_storage_id(field_name: &str) -> Id {
        Id::new(("struct_viewer_pointer_offsets_edit", field_name.to_string()))
    }

    fn clear_pointer_offsets_edit_state(
        user_interface: &Ui,
        field_name: &str,
    ) {
        let pointer_offsets_storage_id = Self::pointer_offsets_edit_storage_id(field_name);

        user_interface.ctx().data_mut(|data| {
            data.remove::<Vec<AnonymousValueString>>(pointer_offsets_storage_id);
        });
    }

    fn parse_pointer_offsets_text(pointer_offsets_text: &str) -> Vec<i64> {
        if let Ok(pointer_offsets) = serde_json::from_str::<Vec<i64>>(pointer_offsets_text) {
            return pointer_offsets;
        }

        pointer_offsets_text
            .split(',')
            .filter_map(|pointer_offset_text| Self::parse_pointer_offset_text(pointer_offset_text))
            .collect()
    }

    fn parse_pointer_offset_text(pointer_offset_text: &str) -> Option<i64> {
        let pointer_offset_text = pointer_offset_text.trim();

        if pointer_offset_text.is_empty() {
            return None;
        }

        let (sign, pointer_offset_text) = pointer_offset_text
            .strip_prefix('-')
            .map(|pointer_offset_text| (-1_i64, pointer_offset_text.trim()))
            .unwrap_or((1_i64, pointer_offset_text));
        let pointer_offset_hex_text = pointer_offset_text
            .strip_prefix("0x")
            .or_else(|| pointer_offset_text.strip_prefix("0X"));

        if let Some(pointer_offset_hex_text) = pointer_offset_hex_text {
            i64::from_str_radix(pointer_offset_hex_text, 16)
                .ok()
                .and_then(|pointer_offset| pointer_offset.checked_mul(sign))
        } else {
            pointer_offset_text
                .parse::<i64>()
                .ok()
                .and_then(|pointer_offset| pointer_offset.checked_mul(sign))
        }
    }

    fn parse_pointer_offset_display_value(pointer_offset_value: &AnonymousValueString) -> Option<i64> {
        let pointer_offset_text = pointer_offset_value.get_anonymous_value_string().trim();

        if pointer_offset_text.is_empty() {
            return None;
        }

        if pointer_offset_value.get_anonymous_value_string_format() != AnonymousValueStringFormat::Hexadecimal {
            return Self::parse_pointer_offset_text(pointer_offset_text);
        }

        let (sign, pointer_offset_text) = pointer_offset_text
            .strip_prefix('-')
            .map(|pointer_offset_text| (-1_i64, pointer_offset_text.trim()))
            .unwrap_or((1_i64, pointer_offset_text));
        let pointer_offset_hex_text = pointer_offset_text
            .strip_prefix("0x")
            .or_else(|| pointer_offset_text.strip_prefix("0X"))
            .unwrap_or(pointer_offset_text);

        i64::from_str_radix(pointer_offset_hex_text, 16)
            .ok()
            .and_then(|pointer_offset| pointer_offset.checked_mul(sign))
    }

    fn pointer_offset_display_text(pointer_offset: i64) -> String {
        if pointer_offset < 0 {
            format!("-{:X}", pointer_offset.saturating_abs())
        } else {
            format!("{:X}", pointer_offset)
        }
    }

    fn pointer_offset_display_values(pointer_offsets: Vec<i64>) -> Vec<AnonymousValueString> {
        pointer_offsets
            .into_iter()
            .map(|pointer_offset| {
                AnonymousValueString::new(
                    Self::pointer_offset_display_text(pointer_offset),
                    AnonymousValueStringFormat::Hexadecimal,
                    ContainerType::None,
                )
            })
            .collect()
    }

    fn pointer_offset_data_type_ref() -> DataTypeRef {
        DataTypeRef::new(DataTypeI64::DATA_TYPE_ID)
    }

    fn render_take_over_header_icon_button(
        &self,
        user_interface: &mut Ui,
        icon_handle: &eframe::egui::TextureHandle,
        tooltip_text: &str,
    ) -> Response {
        let theme = &self.app_context.theme;
        let button_response = user_interface.add_sized(
            vec2(Self::POINTER_OFFSET_ICON_BUTTON_WIDTH, Self::TAKE_OVER_HEADER_HEIGHT),
            Button::new_from_theme(theme)
                .background_color(epaint::Color32::TRANSPARENT)
                .with_tooltip_text(tooltip_text),
        );

        IconDraw::draw(user_interface, button_response.rect, icon_handle);

        button_response
    }

    fn render_pointer_offset_icon_button(
        &self,
        user_interface: &mut Ui,
        icon_handle: &eframe::egui::TextureHandle,
        tooltip_text: &str,
    ) -> Response {
        let theme = &self.app_context.theme;
        let button_response = user_interface.add_sized(
            vec2(Self::POINTER_OFFSET_ICON_BUTTON_WIDTH, Self::POINTER_OFFSET_FIELD_ROW_HEIGHT),
            Button::new_from_theme(theme)
                .with_tooltip_text(tooltip_text)
                .background_color(theme.background_control_secondary)
                .border_color(theme.submenu_border)
                .border_width(1.0),
        );

        IconDraw::draw(user_interface, button_response.rect, icon_handle);

        button_response
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
            .id_salt(format!("struct_viewer_take_over_body_{title}"))
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

    fn render_pointer_offset_editor_section(
        &self,
        user_interface: &mut Ui,
        pointer_offset_value: &mut AnonymousValueString,
        pointer_offset_index: usize,
        pointer_offset_data_type_ref: &DataTypeRef,
    ) -> Option<PointerOffsetRowAction> {
        let theme = &self.app_context.theme;
        let mut pending_row_action = None;

        user_interface.allocate_ui_with_layout(
            vec2(user_interface.available_width().max(1.0), Self::POINTER_OFFSET_FIELD_ROW_HEIGHT),
            Layout::left_to_right(Align::Center),
            |user_interface| {
                user_interface.label(
                    RichText::new(format!("Offset {}", pointer_offset_index + 1))
                        .strong()
                        .color(theme.foreground),
                );

                user_interface.add_space(Self::POINTER_OFFSET_INPUT_SPACING);
                let data_value_box_id = format!("struct_viewer_pointer_offset_value_{}", pointer_offset_index);
                user_interface.add_sized(
                    vec2(Self::POINTER_OFFSET_VALUE_BOX_WIDTH, Self::POINTER_OFFSET_FIELD_ROW_HEIGHT),
                    DataValueBoxView::new(
                        self.app_context.clone(),
                        pointer_offset_value,
                        pointer_offset_data_type_ref,
                        false,
                        true,
                        "offset",
                        &data_value_box_id,
                    )
                    .allowed_anonymous_value_string_formats(vec![
                        AnonymousValueStringFormat::Hexadecimal,
                        AnonymousValueStringFormat::Decimal,
                    ])
                    .normalize_value_format(false)
                    .use_format_text_colors(false)
                    .width(Self::POINTER_OFFSET_VALUE_BOX_WIDTH)
                    .height(Self::POINTER_OFFSET_FIELD_ROW_HEIGHT),
                );

                user_interface.add_space(Self::POINTER_OFFSET_INPUT_SPACING);

                let append_offset_response =
                    self.render_pointer_offset_icon_button(user_interface, &theme.icon_library.icon_handle_common_add, "Append a new offset.");
                if append_offset_response.clicked() {
                    pending_row_action = Some(PointerOffsetRowAction::AppendOffset);
                }

                user_interface.add_space(Self::POINTER_OFFSET_INPUT_SPACING);

                let remove_offset_response =
                    self.render_pointer_offset_icon_button(user_interface, &theme.icon_library.icon_handle_common_delete, "Remove this offset.");
                if remove_offset_response.clicked() {
                    pending_row_action = Some(PointerOffsetRowAction::RemoveOffset);
                }
            },
        );

        pending_row_action
    }

    fn apply_pointer_offset_row_action(
        pointer_offset_values: &mut Vec<AnonymousValueString>,
        pointer_offset_index: usize,
        pointer_offset_row_action: PointerOffsetRowAction,
    ) {
        match pointer_offset_row_action {
            PointerOffsetRowAction::AppendOffset => {
                pointer_offset_values.push(AnonymousValueString::new(
                    String::from("0"),
                    AnonymousValueStringFormat::Hexadecimal,
                    ContainerType::None,
                ));
            }
            PointerOffsetRowAction::RemoveOffset => {
                if pointer_offset_index < pointer_offset_values.len() {
                    pointer_offset_values.remove(pointer_offset_index);
                }
            }
        }
    }

    fn show_pointer_offsets_editor(
        &self,
        user_interface: &mut Ui,
        valued_struct_field: &ValuedStructField,
        pointer_offsets_submission: &mut Option<ValuedStructField>,
        should_cancel_take_over: &mut bool,
    ) {
        let theme = &self.app_context.theme;
        let pointer_offsets_storage_id = Self::pointer_offsets_edit_storage_id(valued_struct_field.get_name());
        let initial_pointer_offsets = Self::parse_pointer_offsets_text(&StructViewerViewData::read_utf8_field_text(valued_struct_field));
        let mut pointer_offset_values = user_interface
            .ctx()
            .data_mut(|data| data.get_temp::<Vec<AnonymousValueString>>(pointer_offsets_storage_id))
            .unwrap_or_else(|| Self::pointer_offset_display_values(initial_pointer_offsets));
        let pointer_offset_data_type_ref = Self::pointer_offset_data_type_ref();
        let mut should_save_offsets = false;

        self.render_take_over_panel(
            user_interface,
            "Edit pointer offsets",
            Self::POINTER_OFFSET_ICON_BUTTON_WIDTH * 2.0,
            |user_interface| {
                let save_response =
                    self.render_take_over_header_icon_button(user_interface, &theme.icon_library.icon_handle_common_check_mark, "Save offsets.");
                if save_response.clicked() {
                    should_save_offsets = true;
                }

                let cancel_response =
                    self.render_take_over_header_icon_button(user_interface, &theme.icon_library.icon_handle_navigation_cancel, "Cancel offset edit.");
                if cancel_response.clicked() {
                    *should_cancel_take_over = true;
                }
            },
            |user_interface| {
                user_interface.add(
                    GroupBox::new_from_theme(theme, "Offsets", |user_interface| {
                        let mut pending_pointer_offset_row_action = None;
                        let pointer_offset_count = pointer_offset_values.len();

                        for pointer_offset_index in 0..pointer_offset_count {
                            let Some(pointer_offset_value) = pointer_offset_values.get_mut(pointer_offset_index) else {
                                continue;
                            };

                            if let Some(pointer_offset_row_action) = self.render_pointer_offset_editor_section(
                                user_interface,
                                pointer_offset_value,
                                pointer_offset_index,
                                &pointer_offset_data_type_ref,
                            ) {
                                pending_pointer_offset_row_action = Some((pointer_offset_index, pointer_offset_row_action));
                            }

                            if pointer_offset_index + 1 < pointer_offset_count {
                                user_interface.add_space(Self::POINTER_OFFSET_SECTION_VERTICAL_SPACING);
                            }
                        }

                        if pointer_offset_count == 0 {
                            let add_response =
                                self.render_pointer_offset_icon_button(user_interface, &theme.icon_library.icon_handle_common_add, "Append a new offset.");

                            if add_response.clicked() {
                                pending_pointer_offset_row_action = Some((0, PointerOffsetRowAction::AppendOffset));
                            }
                        }

                        if let Some((pointer_offset_index, pointer_offset_row_action)) = pending_pointer_offset_row_action {
                            Self::apply_pointer_offset_row_action(&mut pointer_offset_values, pointer_offset_index, pointer_offset_row_action);
                        }
                    })
                    .desired_width(user_interface.available_width()),
                );
            },
        );

        if should_save_offsets {
            let pointer_offsets = pointer_offset_values
                .iter()
                .filter_map(Self::parse_pointer_offset_display_value)
                .collect::<Vec<i64>>();

            match serde_json::to_string(&pointer_offsets) {
                Ok(pointer_offsets_json) => {
                    *pointer_offsets_submission = Some(
                        DataTypeStringUtf8::get_value_from_primitive_string(&pointer_offsets_json)
                            .to_named_valued_struct_field(valued_struct_field.get_name().to_string(), true),
                    );
                }
                Err(error) => {
                    log::warn!("Failed to serialize Struct Viewer pointer offsets edit: {}", error);
                }
            }
        }

        user_interface
            .ctx()
            .data_mut(|data| data.insert_temp(pointer_offsets_storage_id, pointer_offset_values));
    }

    fn resolve_struct_runtime_value_target(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        source_struct_under_view: &ValuedStruct,
    ) -> Option<(u64, String)> {
        if let Some(address_field) = source_struct_under_view.get_field(ProjectItemTypeAddress::PROPERTY_ADDRESS) {
            let address = Self::read_u64_field_value(address_field)?;
            let module_name = source_struct_under_view
                .get_field(ProjectItemTypeAddress::PROPERTY_MODULE)
                .map(StructViewerViewData::read_utf8_field_text)
                .unwrap_or_default();

            return Some((address, module_name));
        }

        let pointer_offset_field = source_struct_under_view.get_field(ProjectItemTypePointer::PROPERTY_OFFSET)?;
        let pointer_module_field = source_struct_under_view.get_field(ProjectItemTypePointer::PROPERTY_MODULE)?;
        let pointer_offsets_field = source_struct_under_view.get_field(ProjectItemTypePointer::PROPERTY_POINTER_OFFSETS)?;
        let pointer_size_field = source_struct_under_view.get_field(ProjectItemTypePointer::PROPERTY_POINTER_SIZE)?;
        let pointer_offsets_json = StructViewerViewData::read_utf8_field_text(pointer_offsets_field);
        let pointer_offsets = serde_json::from_str::<Vec<i64>>(&pointer_offsets_json).ok()?;
        let pointer_size_text = StructViewerViewData::read_utf8_field_text(pointer_size_field);
        let pointer_size = PointerScanPointerSize::from_str(&pointer_size_text).ok()?;
        let pointer = Pointer::new_with_size(
            Self::read_u64_field_value(pointer_offset_field)?,
            pointer_offsets,
            StructViewerViewData::read_utf8_field_text(pointer_module_field),
            pointer_size,
        );

        Self::resolve_pointer_write_target(engine_execution_context, &pointer)
    }

    fn resolve_pointer_write_target(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        pointer: &Pointer,
    ) -> Option<(u64, String)> {
        let mut current_address = pointer.get_address();
        let mut current_module_name = pointer.get_module_name().to_string();

        for pointer_offset in pointer.get_offsets() {
            let pointer_value = Self::read_pointer_value(engine_execution_context, current_address, &current_module_name, pointer.get_pointer_size())?;
            current_address = Pointer::apply_pointer_offset(pointer_value, *pointer_offset)?;
            current_module_name.clear();
        }

        Some((current_address, current_module_name))
    }

    fn read_pointer_value(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        address: u64,
        module_name: &str,
        pointer_size: PointerScanPointerSize,
    ) -> Option<u64> {
        let symbolic_struct_definition =
            squalr_engine_api::structures::structs::symbolic_struct_definition::SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
                pointer_size.to_data_type_ref(),
                ContainerType::None,
            )]);
        let memory_read_request = squalr_engine_api::commands::memory::read::memory_read_request::MemoryReadRequest {
            address,
            module_name: module_name.to_string(),
            symbolic_struct_definition,
            suppress_logging: true,
        };
        let (memory_read_response_sender, memory_read_response_receiver) = std::sync::mpsc::channel();
        let memory_read_command = memory_read_request.to_engine_command();
        let dispatch_result = match engine_execution_context.get_bindings().read() {
            Ok(engine_bindings) => engine_bindings.dispatch_privileged_command(
                memory_read_command,
                Box::new(move |engine_response| {
                    let conversion_result = squalr_engine_api::commands::memory::read::memory_read_response::MemoryReadResponse::from_engine_response(
                        engine_response,
                    )
                    .map_err(|unexpected_response| format!("Unexpected response variant for Struct Viewer memory read request: {:?}", unexpected_response));
                    let _ = memory_read_response_sender.send(conversion_result);
                }),
            ),
            Err(error) => {
                log::error!("Failed to acquire engine bindings lock for Struct Viewer memory read request: {}", error);
                return None;
            }
        };

        if let Err(error) = dispatch_result {
            log::error!("Failed to dispatch Struct Viewer memory read request: {}", error);
            return None;
        }

        let memory_read_response = match memory_read_response_receiver.recv_timeout(std::time::Duration::from_secs(2)) {
            Ok(Ok(memory_read_response)) => memory_read_response,
            Ok(Err(error)) => {
                log::error!("Failed to convert Struct Viewer memory read response: {}", error);
                return None;
            }
            Err(error) => {
                log::error!("Timed out waiting for Struct Viewer memory read response: {}", error);
                return None;
            }
        };

        if !memory_read_response.success {
            return None;
        }

        let data_value = memory_read_response
            .valued_struct
            .get_fields()
            .first()
            .and_then(|valued_struct_field: &ValuedStructField| valued_struct_field.get_data_value())?;

        pointer_size.read_address_value(data_value)
    }

    fn read_u64_field_value(valued_struct_field: &ValuedStructField) -> Option<u64> {
        let data_value = valued_struct_field.get_data_value()?;
        let value_bytes = data_value.get_value_bytes();

        match value_bytes.len() {
            8 => <[u8; 8]>::try_from(value_bytes.as_slice())
                .ok()
                .map(u64::from_le_bytes),
            4 => <[u8; 4]>::try_from(value_bytes.as_slice())
                .ok()
                .map(|value_bytes| u32::from_le_bytes(value_bytes) as u64),
            _ => None,
        }
    }

    fn resolve_symbolic_field_definition_byte_count(
        engine_unprivileged_state: &Arc<squalr_engine_session::engine_unprivileged_state::EngineUnprivilegedState>,
        symbolic_field_definition: &SymbolicFieldDefinition,
    ) -> Option<u64> {
        let unit_size_in_bytes = engine_unprivileged_state
            .get_default_value(symbolic_field_definition.get_data_type_ref())
            .map(|default_value| default_value.get_size_in_bytes())
            .unwrap_or(1);

        Some(
            symbolic_field_definition
                .get_container_type()
                .get_total_size_in_bytes(unit_size_in_bytes),
        )
    }

    fn focus_memory_viewer_for_address_range(
        &self,
        address: u64,
        module_name: &str,
        selection_byte_count: u64,
    ) {
        MemoryViewerViewData::request_focus_address_range(
            self.memory_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            address,
            module_name.to_string(),
            selection_byte_count,
        );

        match self.app_context.docking_manager.write() {
            Ok(mut docking_manager) => {
                docking_manager.set_window_visibility(MemoryViewerView::WINDOW_ID, true);
                docking_manager.select_tab_by_window_id(MemoryViewerView::WINDOW_ID);
            }
            Err(error) => {
                log::error!(
                    "Failed to acquire docking manager while opening the memory viewer from the Struct Viewer: {}",
                    error
                );
            }
        }
    }

    fn focus_code_viewer_for_address(
        &self,
        address: u64,
        module_name: &str,
    ) {
        CodeViewerViewData::request_focus_address(
            self.code_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            address,
            module_name.to_string(),
        );

        match self.app_context.docking_manager.write() {
            Ok(mut docking_manager) => {
                docking_manager.set_window_visibility(CodeViewerView::WINDOW_ID, true);
                docking_manager.select_tab_by_window_id(CodeViewerView::WINDOW_ID);
            }
            Err(error) => {
                log::error!(
                    "Failed to acquire docking manager while opening the code viewer from the Struct Viewer: {}",
                    error
                );
            }
        }
    }
}

impl Widget for StructViewerView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        const ICON_COLUMN_WIDTH: f32 = 32.0;
        const BAR_THICKNESS: f32 = 4.0;
        const MINIMUM_COLUMN_PIXEL_WIDTH: f32 = 80.0;

        let theme = &self.app_context.theme;
        let mut frame_action = StructViewerFrameAction::None;

        let mut new_value_splitter_ratio: Option<f32> = None;
        let mut pointer_offsets_submission: Option<ValuedStructField> = None;
        let mut should_cancel_take_over = false;

        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |mut user_interface| {
                let mut struct_viewer_view_data = match self.struct_viewer_view_data.write("Struct viewer view") {
                    Some(data) => data,
                    None => return,
                };
                let take_over_state = struct_viewer_view_data.take_over_state.clone();
                let mut value_splitter_ratio = struct_viewer_view_data.value_splitter_ratio;
                let content_rect = user_interface.available_rect_before_wrap();
                let content_width = content_rect.width();
                let content_min_x = content_rect.min.x;

                if content_width <= 0.0 {
                    return;
                }

                if let Some(StructViewerTakeOverState::EditPointerOffsets { valued_struct_field }) = take_over_state {
                    drop(struct_viewer_view_data);
                    self.show_pointer_offsets_editor(
                        &mut user_interface,
                        &valued_struct_field,
                        &mut pointer_offsets_submission,
                        &mut should_cancel_take_over,
                    );
                    return;
                }

                if value_splitter_ratio <= 0.0 {
                    value_splitter_ratio = StructViewerViewData::DEFAULT_NAME_SPLITTER_RATIO;

                    new_value_splitter_ratio = Some(value_splitter_ratio);
                }

                let value_splitter_x = content_min_x + content_width * value_splitter_ratio;

                let splitter_min_y = content_rect.min.y;
                let splitter_max_y = content_rect.max.y;

                let value_splitter_rect = Rect::from_min_max(
                    pos2(value_splitter_x - BAR_THICKNESS * 0.5, splitter_min_y),
                    pos2(value_splitter_x + BAR_THICKNESS * 0.5, splitter_max_y),
                );

                // Rows.
                ScrollArea::vertical()
                    .id_salt("struct_viewer")
                    .auto_shrink([false, false])
                    .show(&mut user_interface, |inner_ui| {
                        if let Some(struct_under_view) = struct_viewer_view_data.struct_under_view.as_ref() {
                            let struct_fields = struct_under_view.get_fields().to_vec();
                            let selected_field_name = struct_viewer_view_data.selected_field_name.as_ref().clone();
                            let field_display_values_map = struct_viewer_view_data.field_display_values.clone();
                            let field_presentations_map = struct_viewer_view_data.field_presentations.clone();

                            for (field_row_index, field) in struct_fields.into_iter().enumerate() {
                                let is_selected = selected_field_name.as_deref().unwrap_or_default() == field.get_name();
                                let validation_data_type_ref = struct_viewer_view_data
                                    .field_validation_data_type_refs
                                    .get(field.get_name())
                                    .cloned();
                                let field_display_values = field_display_values_map
                                    .get(field.get_name())
                                    .map(Vec::as_slice);
                                let field_presentation = field_presentations_map
                                    .get(field.get_name())
                                    .cloned()
                                    .unwrap_or_else(|| StructViewerFieldPresentation::new(field.get_name().to_string(), StructViewerFieldEditorKind::ValueBox));

                                match field_presentation.editor_kind() {
                                    StructViewerFieldEditorKind::ValueBox => {
                                        let field_edit_value = struct_viewer_view_data
                                            .field_edit_values
                                            .get_mut(field.get_name());

                                        inner_ui.add(StructViewerEntryView::new(
                                            self.app_context.clone(),
                                            &field,
                                            &field_presentation,
                                            field_row_index,
                                            is_selected,
                                            &mut frame_action,
                                            field_edit_value,
                                            field_display_values,
                                            None,
                                            validation_data_type_ref.as_ref(),
                                            ICON_COLUMN_WIDTH + BAR_THICKNESS,
                                            value_splitter_x + BAR_THICKNESS,
                                        ));
                                    }
                                    StructViewerFieldEditorKind::MemoryViewerButton | StructViewerFieldEditorKind::CodeViewerButton => {
                                        inner_ui.add(StructViewerEntryView::new(
                                            self.app_context.clone(),
                                            &field,
                                            &field_presentation,
                                            field_row_index,
                                            is_selected,
                                            &mut frame_action,
                                            None,
                                            field_display_values,
                                            None,
                                            validation_data_type_ref.as_ref(),
                                            ICON_COLUMN_WIDTH + BAR_THICKNESS,
                                            value_splitter_x + BAR_THICKNESS,
                                        ));
                                    }
                                    StructViewerFieldEditorKind::DataTypeSelector => {
                                        let field_data_type_selection = struct_viewer_view_data
                                            .field_data_type_selections
                                            .get_mut(field.get_name());

                                        inner_ui.add(StructViewerEntryView::new(
                                            self.app_context.clone(),
                                            &field,
                                            &field_presentation,
                                            field_row_index,
                                            is_selected,
                                            &mut frame_action,
                                            None,
                                            field_display_values,
                                            field_data_type_selection,
                                            validation_data_type_ref.as_ref(),
                                            ICON_COLUMN_WIDTH + BAR_THICKNESS,
                                            value_splitter_x + BAR_THICKNESS,
                                        ));
                                    }
                                    StructViewerFieldEditorKind::ContainerTypeSelector | StructViewerFieldEditorKind::ProjectItemPointerSizeSelector => {
                                        inner_ui.add(StructViewerEntryView::new(
                                            self.app_context.clone(),
                                            &field,
                                            &field_presentation,
                                            field_row_index,
                                            is_selected,
                                            &mut frame_action,
                                            None,
                                            field_display_values,
                                            None,
                                            validation_data_type_ref.as_ref(),
                                            ICON_COLUMN_WIDTH + BAR_THICKNESS,
                                            value_splitter_x + BAR_THICKNESS,
                                        ));
                                    }
                                    StructViewerFieldEditorKind::ProjectItemPointerOffsetsEditor => {
                                        let field_edit_value = struct_viewer_view_data
                                            .field_edit_values
                                            .get_mut(field.get_name());

                                        inner_ui.add(StructViewerEntryView::new(
                                            self.app_context.clone(),
                                            &field,
                                            &field_presentation,
                                            field_row_index,
                                            is_selected,
                                            &mut frame_action,
                                            field_edit_value,
                                            field_display_values,
                                            None,
                                            validation_data_type_ref.as_ref(),
                                            ICON_COLUMN_WIDTH + BAR_THICKNESS,
                                            value_splitter_x + BAR_THICKNESS,
                                        ));
                                    }
                                }
                            }
                        }
                    });

                // Draw non-resizable icon/name divider.
                let icon_divider_x = content_min_x + ICON_COLUMN_WIDTH;
                let icon_divider_rect = Rect::from_min_max(
                    pos2(icon_divider_x - BAR_THICKNESS * 0.5, splitter_min_y),
                    pos2(icon_divider_x + BAR_THICKNESS * 0.5, splitter_max_y),
                );

                user_interface
                    .painter()
                    .rect_filled(icon_divider_rect, 0.0, theme.background_control);

                // Draw the name/value divider.
                let value_splitter_response = user_interface
                    .interact(value_splitter_rect, user_interface.id().with("value_splitter"), Sense::drag())
                    .on_hover_cursor(CursorIcon::ResizeHorizontal);

                user_interface
                    .painter()
                    .rect_filled(value_splitter_rect, 0.0, theme.background_control);

                if value_splitter_response.dragged() {
                    let drag_delta = value_splitter_response.drag_delta();
                    let mut new_x = value_splitter_x + drag_delta.x;
                    let min_x = content_min_x + ICON_COLUMN_WIDTH + MINIMUM_COLUMN_PIXEL_WIDTH;
                    let max_x = content_min_x + content_width - MINIMUM_COLUMN_PIXEL_WIDTH;

                    new_x = safe_clamp_f32(new_x, min_x, max_x);
                    new_value_splitter_ratio = Some((new_x - content_min_x) / content_width);
                }
            })
            .response;

        // Commit splitter changes.
        if new_value_splitter_ratio.is_some() {
            if let Some(mut data) = self.struct_viewer_view_data.write("Struct viewer view") {
                if let Some(ratio) = new_value_splitter_ratio {
                    data.value_splitter_ratio = ratio;
                }
            }
        }

        let active_pointer_offsets_field_name = self
            .struct_viewer_view_data
            .read("Struct Viewer active pointer offsets editor")
            .and_then(|struct_viewer_view_data| match struct_viewer_view_data.take_over_state.as_ref() {
                Some(StructViewerTakeOverState::EditPointerOffsets { valued_struct_field }) => Some(valued_struct_field.get_name().to_string()),
                None => None,
            });

        if active_pointer_offsets_field_name.is_some()
            && self
                .app_context
                .window_focus_manager
                .can_window_handle_shortcuts(user_interface.ctx(), Self::WINDOW_ID)
            && user_interface.input(|input_state| input_state.key_pressed(Key::Escape) || input_state.key_pressed(Key::Backspace))
        {
            should_cancel_take_over = true;
        }

        if should_cancel_take_over {
            if let Some(active_pointer_offsets_field_name) = active_pointer_offsets_field_name.as_deref() {
                Self::clear_pointer_offsets_edit_state(user_interface, active_pointer_offsets_field_name);
            }

            StructViewerViewData::cancel_take_over_state(self.struct_viewer_view_data.clone());
        }

        if let Some(edited_field) = pointer_offsets_submission {
            Self::clear_pointer_offsets_edit_state(user_interface, edited_field.get_name());
            StructViewerViewData::cancel_take_over_state(self.struct_viewer_view_data.clone());
            frame_action = StructViewerFrameAction::EditValue(edited_field);
        }

        match frame_action {
            StructViewerFrameAction::None => {}
            StructViewerFrameAction::SelectField(field_name) => {
                StructViewerViewData::set_selected_field(self.struct_viewer_view_data.clone(), field_name);
            }
            StructViewerFrameAction::EditValue(edited_field) => {
                let mut modified_field_callback = None;
                let mut modified_field = None;

                if let Some(mut struct_viewer_view_data) = self.struct_viewer_view_data.write("Struct viewer edit value") {
                    let Some(source_edited_field) = struct_viewer_view_data.resolve_source_field_edit(&edited_field) else {
                        return response;
                    };

                    if let Some(source_struct_under_view) = Arc::make_mut(&mut struct_viewer_view_data.source_struct_under_view).as_mut() {
                        if let Some(field_under_view) = source_struct_under_view.get_field_mut(source_edited_field.get_name()) {
                            field_under_view.set_field_data(source_edited_field.get_field_data().clone());
                        } else {
                            source_struct_under_view.set_field_data(
                                source_edited_field.get_name(),
                                source_edited_field.get_field_data().clone(),
                                source_edited_field.get_is_read_only(),
                            );
                        }
                    }

                    struct_viewer_view_data.refresh_cached_field_state(&self.app_context.engine_unprivileged_state);

                    modified_field_callback = struct_viewer_view_data.struct_field_modified_callback.clone();
                    modified_field = Some(source_edited_field);
                }

                if let (Some(struct_field_modified_callback), Some(modified_field)) = (modified_field_callback, modified_field) {
                    struct_field_modified_callback(modified_field);
                }
            }
            StructViewerFrameAction::RequestFieldEditor(requested_field) => {
                StructViewerViewData::request_pointer_offsets_editor(self.struct_viewer_view_data.clone(), requested_field);
            }
            StructViewerFrameAction::OpenInMemoryViewer(field_name) => {
                if let Some((address, module_name, selection_byte_count)) = self.resolve_memory_viewer_target_for_field(&field_name) {
                    self.focus_memory_viewer_for_address_range(address, &module_name, selection_byte_count);
                } else {
                    log::warn!("Failed to resolve Struct Viewer memory viewer target for field: {}.", field_name);
                }
            }
            StructViewerFrameAction::OpenInCodeViewer(field_name) => {
                if let Some((address, module_name, _selection_byte_count)) = self.resolve_memory_viewer_target_for_field(&field_name) {
                    self.focus_code_viewer_for_address(address, &module_name);
                } else {
                    log::warn!("Failed to resolve Struct Viewer code viewer target for field: {}.", field_name);
                }
            }
        }

        response
    }
}
