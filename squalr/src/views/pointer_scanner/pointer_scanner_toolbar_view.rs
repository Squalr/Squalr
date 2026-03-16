use crate::app_context::AppContext;
use crate::ui::widgets::controls::{
    button::Button, data_type_selector::data_type_selector_view::DataTypeSelectorView, data_value_box::data_value_box_view::DataValueBoxView,
};
use crate::views::pointer_scanner::view_data::pointer_scanner_view_data::PointerScannerViewData;
use crate::views::project_explorer::project_hierarchy::view_data::project_hierarchy_view_data::ProjectHierarchyViewData;
use eframe::egui::{Align2, Color32, Response, RichText, Sense, Ui, UiBuilder, Widget, vec2};
use epaint::{CornerRadius, Stroke, StrokeKind};
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use std::sync::Arc;

#[derive(Clone)]
pub struct PointerScannerToolbarView {
    app_context: Arc<AppContext>,
    pointer_scanner_view_data: Dependency<PointerScannerViewData>,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
}

impl PointerScannerToolbarView {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let pointer_scanner_view_data = app_context
            .dependency_container
            .get_dependency::<PointerScannerViewData>();
        let project_hierarchy_view_data = app_context
            .dependency_container
            .get_dependency::<ProjectHierarchyViewData>();

        Self {
            app_context,
            pointer_scanner_view_data,
            project_hierarchy_view_data,
        }
    }

    pub fn get_height(&self) -> f32 {
        122.0
    }

    fn draw_action_button(
        &self,
        user_interface: &mut Ui,
        label: &str,
        tooltip_text: &str,
        size: [f32; 2],
        fill_color: Color32,
    ) -> Response {
        let theme = &self.app_context.theme;
        let button_response = user_interface.add_sized(
            size,
            Button::new_from_theme(theme)
                .background_color(fill_color)
                .with_tooltip_text(tooltip_text),
        );

        user_interface.painter().text(
            button_response.rect.center(),
            Align2::CENTER_CENTER,
            label,
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
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
        let mut should_validate_scan = false;
        let mut should_refresh_summary = false;
        let mut should_copy_chain = false;
        let mut should_export_chain = false;
        let mut should_add_to_project = false;
        let mut copy_text = None;
        let mut export_text = None;
        let mut project_item_create_request = None;

        {
            let mut pointer_scanner_view_data = match self
                .pointer_scanner_view_data
                .write("Pointer scanner toolbar view")
            {
                Some(pointer_scanner_view_data) => pointer_scanner_view_data,
                None => return response,
            };

            let builder = UiBuilder::new().max_rect(toolbar_rectangle);
            let unsigned_data_type = DataTypeRef::new("u64");

            user_interface.scope_builder(builder, |user_interface| {
                user_interface.add_space(8.0);

                user_interface.horizontal_wrapped(|user_interface| {
                    user_interface.label("Pointer Size");
                    user_interface.add(
                        DataTypeSelectorView::new(
                            self.app_context.clone(),
                            &mut pointer_scanner_view_data.pointer_size_data_type_selection,
                            "pointer_scanner_pointer_size",
                        )
                        .width(92.0)
                        .height(28.0)
                        .available_data_types(vec![DataTypeRef::new("u32"), DataTypeRef::new("u64")])
                        .stacked_list()
                        .hide_placeholder_entries(),
                    );
                    pointer_scanner_view_data.synchronize_pointer_size_from_selection();
                    let pointer_size_data_type = pointer_scanner_view_data
                        .pointer_size_data_type_selection
                        .visible_data_type()
                        .clone();

                    user_interface.add_space(8.0);
                    user_interface.label("Target");
                    user_interface.add(
                        DataValueBoxView::new(
                            self.app_context.clone(),
                            &mut pointer_scanner_view_data.target_address_input,
                            &pointer_size_data_type,
                            false,
                            true,
                            "Enter target address...",
                            "pointer_scanner_target_address",
                        )
                        .width(192.0),
                    );

                    user_interface.add_space(8.0);
                    user_interface.label("Depth");
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
                        .width(84.0),
                    );

                    user_interface.add_space(8.0);
                    user_interface.label("Radius");
                    user_interface.add(
                        DataValueBoxView::new(
                            self.app_context.clone(),
                            &mut pointer_scanner_view_data.offset_radius_input,
                            &unsigned_data_type,
                            false,
                            true,
                            "Radius",
                            "pointer_scanner_offset_radius",
                        )
                        .width(108.0),
                    );
                });

                user_interface.add_space(8.0);

                user_interface.horizontal_wrapped(|user_interface| {
                    let pointer_size_data_type = pointer_scanner_view_data
                        .pointer_size_data_type_selection
                        .visible_data_type()
                        .clone();

                    user_interface.label("Validate");
                    user_interface.add(
                        DataValueBoxView::new(
                            self.app_context.clone(),
                            &mut pointer_scanner_view_data.validation_target_address_input,
                            &pointer_size_data_type,
                            false,
                            true,
                            "Enter validation address...",
                            "pointer_scanner_validation_target_address",
                        )
                        .width(192.0),
                    );

                    user_interface.add_space(8.0);

                    if self
                        .draw_action_button(
                            user_interface,
                            "Start",
                            "Start a new pointer scan session.",
                            [72.0, 28.0],
                            theme.background_control_primary,
                        )
                        .clicked()
                    {
                        should_start_scan = true;
                    }

                    if self
                        .draw_action_button(
                            user_interface,
                            "Refresh",
                            "Refresh the active pointer scan summary.",
                            [80.0, 28.0],
                            Color32::TRANSPARENT,
                        )
                        .clicked()
                    {
                        should_refresh_summary = true;
                    }

                    if self
                        .draw_action_button(
                            user_interface,
                            "Validate",
                            "Validate the active pointer scan session with a new target.",
                            [84.0, 28.0],
                            theme.background_control_primary,
                        )
                        .clicked()
                    {
                        should_validate_scan = true;
                    }

                    if self
                        .draw_action_button(
                            user_interface,
                            "Copy",
                            "Copy the selected pointer chain text.",
                            [68.0, 28.0],
                            Color32::TRANSPARENT,
                        )
                        .clicked()
                    {
                        should_copy_chain = true;
                    }

                    if self
                        .draw_action_button(
                            user_interface,
                            "Export",
                            "Copy the selected pointer chain metadata to the clipboard.",
                            [76.0, 28.0],
                            Color32::TRANSPARENT,
                        )
                        .clicked()
                    {
                        should_export_chain = true;
                    }

                    if self
                        .draw_action_button(
                            user_interface,
                            "Add To Project",
                            "Persist the selected pointer chain to the current project.",
                            [126.0, 28.0],
                            theme.background_control_primary,
                        )
                        .clicked()
                    {
                        should_add_to_project = true;
                    }
                });

                user_interface.add_space(8.0);
                user_interface.label(
                    RichText::new(&pointer_scanner_view_data.status_message)
                        .font(theme.font_library.font_noto_sans.font_small.clone())
                        .color(theme.foreground_preview),
                );
                user_interface.add_space(8.0);
            });
        }

        if should_start_scan {
            PointerScannerViewData::start_scan(self.pointer_scanner_view_data.clone(), self.app_context.engine_unprivileged_state.clone());
        }

        if should_validate_scan {
            PointerScannerViewData::validate_scan(self.pointer_scanner_view_data.clone(), self.app_context.engine_unprivileged_state.clone());
        }

        if should_refresh_summary {
            PointerScannerViewData::request_summary(self.pointer_scanner_view_data.clone(), self.app_context.engine_unprivileged_state.clone(), None);
        }

        if should_copy_chain {
            copy_text = PointerScannerViewData::build_copy_text(self.pointer_scanner_view_data.clone());
        }

        if should_export_chain {
            export_text = PointerScannerViewData::build_export_text(self.pointer_scanner_view_data.clone());
        }

        if should_add_to_project {
            let target_directory_path = ProjectHierarchyViewData::get_selected_directory_path(self.project_hierarchy_view_data.clone());
            project_item_create_request =
                PointerScannerViewData::build_project_item_create_request(self.pointer_scanner_view_data.clone(), target_directory_path);
        }

        if let Some(copy_text) = copy_text {
            self.app_context.context.copy_text(copy_text);
        }

        if let Some(export_text) = export_text {
            self.app_context.context.copy_text(export_text);
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
