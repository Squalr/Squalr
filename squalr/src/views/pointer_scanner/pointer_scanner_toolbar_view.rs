use crate::app_context::AppContext;
use crate::ui::draw::icon_draw::IconDraw;
use crate::ui::widgets::controls::{
    button::Button, data_type_selector::data_type_selector_view::DataTypeSelectorView, data_value_box::data_value_box_view::DataValueBoxView,
};
use crate::views::pointer_scanner::view_data::pointer_scanner_view_data::PointerScannerViewData;
use crate::views::process_selector::view_data::process_selector_view_data::ProcessSelectorViewData;
use crate::views::project_explorer::project_hierarchy::view_data::project_hierarchy_view_data::ProjectHierarchyViewData;
use eframe::egui::{Color32, Response, Sense, Ui, UiBuilder, Widget, vec2};
use epaint::{CornerRadius, Stroke, StrokeKind, TextureHandle};
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
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
        IconDraw::draw_sized(user_interface, button_response.rect.center(), vec2(16.0, 16.0), icon_handle);

        button_response
    }
}

impl Widget for PointerScannerToolbarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let (toolbar_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), self.get_height()), Sense::hover());

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

        {
            let mut pointer_scanner_view_data = match self
                .pointer_scanner_view_data
                .write("Pointer scanner toolbar view")
            {
                Some(pointer_scanner_view_data) => pointer_scanner_view_data,
                None => return response,
            };
            if let Some(process_bitness) = opened_process_bitness {
                pointer_scanner_view_data.synchronize_pointer_size_with_process_bitness(process_bitness);
            }

            let builder = UiBuilder::new().max_rect(toolbar_rectangle);
            let unsigned_data_type = DataTypeRef::new("u64");
            let action_button_size = [36.0, 28.0];
            let has_active_pointer_scan_session = pointer_scanner_view_data.has_active_session();
            let target_placeholder = if has_active_pointer_scan_session {
                "Enter validation address..."
            } else {
                "Enter target address..."
            };
            let are_session_actions_disabled = pointer_scanner_view_data.is_querying_summary
                || pointer_scanner_view_data.is_starting_scan
                || pointer_scanner_view_data.is_validating_scan
                || pointer_scanner_view_data.is_resetting_scan;
            let start_scan_tooltip = if has_active_pointer_scan_session {
                "Validate the active pointer scan session with the validation target address."
            } else {
                "Start a new pointer scan session."
            };

            user_interface.scope_builder(builder, |user_interface| {
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
                            .width(96.0)
                            .height(Self::CONTROL_HEIGHT)
                            .available_data_types(vec![DataTypeRef::new("u32"), DataTypeRef::new("u64")])
                            .stacked_list()
                            .hide_placeholder_entries(),
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
                            .hide_placeholder_entries(),
                        );
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
                                are_session_actions_disabled,
                            )
                            .clicked()
                        {
                            if !has_active_pointer_scan_session {
                                if let Some(process_bitness) = opened_process_bitness {
                                    pointer_scanner_view_data.force_pointer_size_from_process_bitness(process_bitness);
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
                            .use_format_text_colors(false),
                        );

                        user_interface.add_space(Self::GROUP_SPACING);
                        if self
                            .draw_icon_button(
                                user_interface,
                                &theme.icon_library.icon_handle_navigation_right_arrow,
                                "Start or validate a value-seeded pointer scan session.",
                                action_button_size,
                                Color32::TRANSPARENT,
                                are_session_actions_disabled,
                            )
                            .clicked()
                        {
                            if !has_active_pointer_scan_session {
                                if let Some(process_bitness) = opened_process_bitness {
                                    pointer_scanner_view_data.force_pointer_size_from_process_bitness(process_bitness);
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
