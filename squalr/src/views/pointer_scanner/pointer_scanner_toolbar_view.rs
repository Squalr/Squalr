use crate::app_context::AppContext;
use crate::ui::draw::icon_draw::IconDraw;
use crate::ui::widgets::controls::{
    button::Button, data_type_selector::data_type_selector_view::DataTypeSelectorView, data_value_box::data_value_box_view::DataValueBoxView,
};
use crate::views::pointer_scanner::view_data::pointer_scanner_view_data::PointerScannerViewData;
use crate::views::process_selector::view_data::process_selector_view_data::ProcessSelectorViewData;
use crate::views::project_explorer::project_hierarchy::view_data::project_hierarchy_view_data::ProjectHierarchyViewData;
use eframe::egui::{Align, Color32, ComboBox, Layout, Response, Sense, Ui, UiBuilder, Widget, vec2};
use epaint::{CornerRadius, Stroke, StrokeKind, TextureHandle};
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::data_types::{
    built_in_types::{
        u32::data_type_u32::DataTypeU32, u32be::data_type_u32be::DataTypeU32be, u64::data_type_u64::DataTypeU64, u64be::data_type_u64be::DataTypeU64be,
    },
    data_type_ref::DataTypeRef,
};
use squalr_engine_api::structures::pointer_scans::pointer_scan_address_space::PointerScanAddressSpace;
use std::sync::Arc;

#[derive(Clone)]
pub struct PointerScannerToolbarView {
    app_context: Arc<AppContext>,
    pointer_scanner_view_data: Dependency<PointerScannerViewData>,
    process_selector_view_data: Dependency<ProcessSelectorViewData>,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
}

impl PointerScannerToolbarView {
    const CONTROL_HEIGHT: f32 = 28.0;
    const ROW_SPACING: f32 = 6.0;
    const LEADING_ROW_PADDING: f32 = 8.0;
    const GROUP_SPACING: f32 = 8.0;
    const BOTTOM_PADDING: f32 = 4.0;

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let pointer_scanner_view_data = app_context
            .dependency_container
            .get_dependency::<PointerScannerViewData>();
        let process_selector_view_data = app_context
            .dependency_container
            .get_dependency::<ProcessSelectorViewData>();
        let project_hierarchy_view_data = app_context
            .dependency_container
            .get_dependency::<ProjectHierarchyViewData>();

        Self {
            app_context,
            pointer_scanner_view_data,
            process_selector_view_data,
            project_hierarchy_view_data,
        }
    }

    pub fn get_height(&self) -> f32 {
        Self::CONTROL_HEIGHT * 3.0 + Self::ROW_SPACING * 3.0 + Self::BOTTOM_PADDING
    }

    fn draw_icon_button(
        &self,
        user_interface: &mut Ui,
        icon_handle: &TextureHandle,
        tooltip_text: &str,
        size: [f32; 2],
        fill_color: Color32,
        disabled: bool,
    ) -> Response {
        let theme = &self.app_context.theme;
        let button_response = user_interface.add_sized(
            size,
            Button::new_from_theme(theme)
                .disabled(disabled)
                .background_color(fill_color)
                .with_tooltip_text(tooltip_text),
        );
        IconDraw::draw_sized_tinted(
            user_interface,
            button_response.rect.center(),
            vec2(16.0, 16.0),
            icon_handle,
            if disabled { theme.foreground_preview } else { Color32::WHITE },
        );

        button_response
    }
}

impl Widget for PointerScannerToolbarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let (toolbar_rectangle, response) =
            user_interface.allocate_exact_size(vec2(user_interface.available_width().max(1.0), self.get_height()), Sense::hover());

        user_interface
            .painter()
            .rect_filled(toolbar_rectangle, CornerRadius::ZERO, theme.background_primary);
        user_interface.painter().rect_stroke(
            toolbar_rectangle,
            CornerRadius::ZERO,
            Stroke::new(1.0, theme.background_control),
            StrokeKind::Inside,
        );

        let mut should_start_scan = false;
        let mut should_reset_scan = false;
        let mut should_start_value_scan = false;
        let mut should_add_to_project = false;
        let mut project_item_create_request = None;
        let opened_process_bitness = self
            .process_selector_view_data
            .read("Pointer scanner toolbar view opened process")
            .and_then(|process_selector_view_data| {
                process_selector_view_data
                    .opened_process
                    .as_ref()
                    .map(|opened_process| opened_process.get_bitness())
            });
        let has_opened_process = opened_process_bitness.is_some();
        let available_data_types = self
            .app_context
            .engine_unprivileged_state
            .get_registered_data_type_refs();
        let available_pointer_size_data_types = [
            "u24",
            "u24be",
            DataTypeU32::DATA_TYPE_ID,
            DataTypeU32be::DATA_TYPE_ID,
            DataTypeU64::DATA_TYPE_ID,
            DataTypeU64be::DATA_TYPE_ID,
        ]
        .iter()
        .map(|data_type_id| DataTypeRef::new(data_type_id))
        .filter(|data_type_ref| available_data_types.contains(data_type_ref))
        .collect::<Vec<_>>();

        {
            let mut pointer_scanner_view_data = match self
                .pointer_scanner_view_data
                .write("Pointer scanner toolbar view")
            {
                Some(pointer_scanner_view_data) => pointer_scanner_view_data,
                None => return response,
            };
            if matches!(
                pointer_scanner_view_data.resolve_requested_address_space_for_new_address_scan(),
                PointerScanAddressSpace::EmulatorMemory
            ) {
                if let Some(process_bitness) = opened_process_bitness {
                    pointer_scanner_view_data.synchronize_pointer_size_with_process_bitness(process_bitness);
                }
            }

            let builder = UiBuilder::new()
                .max_rect(toolbar_rectangle)
                .layout(Layout::top_down(Align::Min));
            let unsigned_data_type = DataTypeRef::new(DataTypeU64::DATA_TYPE_ID);
            let action_button_size = [36.0, 28.0];
            let has_active_pointer_scan_session = pointer_scanner_view_data.has_active_session();
            let target_placeholder = if has_active_pointer_scan_session {
                "Enter validation address..."
            } else {
                "Enter target address..."
            };
            let are_session_actions_disabled = pointer_scanner_view_data.has_mutating_session_request_in_progress();
            let start_scan_tooltip = if has_active_pointer_scan_session {
                if has_opened_process {
                    "Validate the active pointer scan session with the validation target address."
                } else {
                    "Attach to a process before validating the active pointer scan session."
                }
            } else {
                if has_opened_process {
                    "Start a new pointer scan session."
                } else {
                    "Attach to a process before starting a pointer scan session."
                }
            };
            let start_value_scan_tooltip = if has_active_pointer_scan_session {
                if has_opened_process {
                    "Validate the active value-seeded pointer scan session."
                } else {
                    "Attach to a process before validating the active value-seeded pointer scan session."
                }
            } else if has_opened_process {
                "Start a value-seeded pointer scan session."
            } else {
                "Attach to a process before starting a value-seeded pointer scan session."
            };

            user_interface.scope_builder(builder, |user_interface| {
                user_interface.set_clip_rect(toolbar_rectangle);
                user_interface.add_space(Self::ROW_SPACING);

                user_interface.allocate_ui(vec2(user_interface.available_width(), Self::CONTROL_HEIGHT), |user_interface| {
                    user_interface.with_layout(eframe::egui::Layout::left_to_right(eframe::egui::Align::Center), |user_interface| {
                        user_interface.add_space(Self::LEADING_ROW_PADDING);
                        if self
                            .draw_icon_button(
                                user_interface,
                                &theme.icon_library.icon_handle_scan_new,
                                "Clear the active pointer scan session.",
                                action_button_size,
                                Color32::TRANSPARENT,
                                are_session_actions_disabled,
                            )
                            .clicked()
                        {
                            should_reset_scan = true;
                        }

                        user_interface.add_space(Self::GROUP_SPACING);
                        user_interface.add(
                            DataValueBoxView::new(
                                self.app_context.clone(),
                                &mut pointer_scanner_view_data.max_depth_input,
                                &unsigned_data_type,
                                false,
                                true,
                                "Depth",
                                "pointer_scanner_max_depth",
                            )
                            .width(84.0)
                            .height(Self::CONTROL_HEIGHT)
                            .use_format_text_colors(false),
                        );

                        user_interface.add_space(Self::GROUP_SPACING);
                        user_interface.add(
                            DataValueBoxView::new(
                                self.app_context.clone(),
                                &mut pointer_scanner_view_data.offset_radius_input,
                                &unsigned_data_type,
                                false,
                                true,
                                "Offset",
                                "pointer_scanner_offset_radius",
                            )
                            .width(100.0)
                            .height(Self::CONTROL_HEIGHT)
                            .use_format_text_colors(false),
                        );

                        user_interface.add_space(Self::GROUP_SPACING);
                        user_interface.add(
                            DataTypeSelectorView::new(
                                self.app_context.clone(),
                                &mut pointer_scanner_view_data.pointer_size_data_type_selection,
                                "pointer_scanner_pointer_size",
                            )
                            .width(116.0)
                            .height(Self::CONTROL_HEIGHT)
                            .available_data_types(available_pointer_size_data_types.clone())
                            .stacked_list(),
                        );
                        pointer_scanner_view_data.synchronize_pointer_size_from_selection();

                        user_interface.add_space(Self::GROUP_SPACING);
                        user_interface.add(
                            DataTypeSelectorView::new(
                                self.app_context.clone(),
                                &mut pointer_scanner_view_data.target_data_type_selection,
                                "pointer_scanner_target_data_type",
                            )
                            .width(132.0)
                            .height(Self::CONTROL_HEIGHT)
                            .available_data_types(available_data_types.clone()),
                        );

                        user_interface.add_space(Self::GROUP_SPACING);
                        ComboBox::from_id_salt("pointer_scanner_address_space")
                            .width(138.0)
                            .selected_text(pointer_scanner_view_data.pointer_scan_address_space.label())
                            .show_ui(user_interface, |user_interface| {
                                user_interface.selectable_value(
                                    &mut pointer_scanner_view_data.pointer_scan_address_space,
                                    PointerScanAddressSpace::Auto,
                                    PointerScanAddressSpace::Auto.label(),
                                );
                                user_interface.selectable_value(
                                    &mut pointer_scanner_view_data.pointer_scan_address_space,
                                    PointerScanAddressSpace::GameMemory,
                                    PointerScanAddressSpace::GameMemory.label(),
                                );
                                user_interface.selectable_value(
                                    &mut pointer_scanner_view_data.pointer_scan_address_space,
                                    PointerScanAddressSpace::EmulatorMemory,
                                    PointerScanAddressSpace::EmulatorMemory.label(),
                                );
                            });
                    });
                });

                user_interface.add_space(Self::ROW_SPACING);

                user_interface.allocate_ui(vec2(user_interface.available_width(), Self::CONTROL_HEIGHT), |user_interface| {
                    user_interface.with_layout(eframe::egui::Layout::left_to_right(eframe::egui::Align::Center), |user_interface| {
                        let pointer_size_data_type = pointer_scanner_view_data
                            .pointer_size_data_type_selection
                            .visible_data_type()
                            .clone();
                        let active_target_input = if has_active_pointer_scan_session {
                            &mut pointer_scanner_view_data.validation_target_address_input
                        } else {
                            &mut pointer_scanner_view_data.target_address_input
                        };

                        user_interface.add_space(Self::LEADING_ROW_PADDING);
                        user_interface.add(
                            DataValueBoxView::new(
                                self.app_context.clone(),
                                active_target_input,
                                &pointer_size_data_type,
                                false,
                                true,
                                target_placeholder,
                                "pointer_scanner_active_target_address",
                            )
                            .height(Self::CONTROL_HEIGHT)
                            .use_preview_foreground(true)
                            .use_format_text_colors(false),
                        );

                        user_interface.add_space(Self::GROUP_SPACING);
                        if self
                            .draw_icon_button(
                                user_interface,
                                &theme.icon_library.icon_handle_navigation_right_arrow,
                                start_scan_tooltip,
                                action_button_size,
                                Color32::TRANSPARENT,
                                are_session_actions_disabled || !has_opened_process,
                            )
                            .clicked()
                        {
                            if !has_active_pointer_scan_session {
                                if matches!(
                                    pointer_scanner_view_data.resolve_requested_address_space_for_new_address_scan(),
                                    PointerScanAddressSpace::EmulatorMemory
                                ) {
                                    if let Some(process_bitness) = opened_process_bitness {
                                        pointer_scanner_view_data.force_pointer_size_from_process_bitness(process_bitness);
                                    }
                                }
                            }
                            should_start_scan = true;
                        }

                        if self
                            .draw_icon_button(
                                user_interface,
                                &theme.icon_library.icon_handle_common_add,
                                "Persist the selected pointer chain to the current project.",
                                action_button_size,
                                Color32::TRANSPARENT,
                                false,
                            )
                            .clicked()
                        {
                            should_add_to_project = true;
                        }
                    });
                });

                user_interface.add_space(Self::ROW_SPACING);

                user_interface.allocate_ui(vec2(user_interface.available_width(), Self::CONTROL_HEIGHT), |user_interface| {
                    user_interface.with_layout(eframe::egui::Layout::left_to_right(eframe::egui::Align::Center), |user_interface| {
                        let value_data_type_ref = pointer_scanner_view_data
                            .target_data_type_selection
                            .active_data_type()
                            .clone();
                        let active_value_input = if has_active_pointer_scan_session {
                            &mut pointer_scanner_view_data.validation_target_value_input
                        } else {
                            &mut pointer_scanner_view_data.target_value_input
                        };
                        let value_placeholder = if has_active_pointer_scan_session {
                            "Enter validation value..."
                        } else {
                            "Enter target value..."
                        };

                        user_interface.add_space(Self::LEADING_ROW_PADDING);
                        user_interface.add(
                            DataValueBoxView::new(
                                self.app_context.clone(),
                                active_value_input,
                                &value_data_type_ref,
                                false,
                                true,
                                value_placeholder,
                                "pointer_scanner_active_target_value",
                            )
                            .height(Self::CONTROL_HEIGHT)
                            .use_preview_foreground(true)
                            .use_format_text_colors(false),
                        );

                        user_interface.add_space(Self::GROUP_SPACING);
                        if self
                            .draw_icon_button(
                                user_interface,
                                &theme.icon_library.icon_handle_navigation_right_arrow,
                                start_value_scan_tooltip,
                                action_button_size,
                                Color32::TRANSPARENT,
                                are_session_actions_disabled || !has_opened_process,
                            )
                            .clicked()
                        {
                            if !has_active_pointer_scan_session {
                                if matches!(
                                    pointer_scanner_view_data.resolve_requested_address_space_for_new_value_scan(),
                                    PointerScanAddressSpace::EmulatorMemory
                                ) {
                                    if let Some(process_bitness) = opened_process_bitness {
                                        pointer_scanner_view_data.force_pointer_size_from_process_bitness(process_bitness);
                                    }
                                }
                            }
                            should_start_value_scan = true;
                        }
                    });
                });
                user_interface.add_space(Self::BOTTOM_PADDING);
            });
        }

        if should_reset_scan {
            PointerScannerViewData::reset_scan(self.pointer_scanner_view_data.clone(), self.app_context.engine_unprivileged_state.clone());
        }

        if should_start_scan {
            PointerScannerViewData::start_scan(self.pointer_scanner_view_data.clone(), self.app_context.engine_unprivileged_state.clone());
        }

        if should_start_value_scan {
            PointerScannerViewData::start_value_scan(self.pointer_scanner_view_data.clone(), self.app_context.engine_unprivileged_state.clone());
        }

        if should_add_to_project {
            let target_directory_path = ProjectHierarchyViewData::get_selected_directory_path(self.project_hierarchy_view_data.clone());
            project_item_create_request =
                PointerScannerViewData::build_project_item_create_request(self.pointer_scanner_view_data.clone(), target_directory_path);
        }

        if let Some(project_item_create_request) = project_item_create_request {
            project_item_create_request.send(&self.app_context.engine_unprivileged_state, |project_items_create_response| {
                if !project_items_create_response.success {
                    log::error!("Failed to add pointer chain to the project.");
                }
            });
        }

        response
    }
}
