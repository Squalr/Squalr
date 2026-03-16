use crate::app_context::AppContext;
use crate::views::pointer_scanner::view_data::pointer_scanner_view_data::PointerScannerViewData;
use crate::views::project_explorer::project_hierarchy::view_data::project_hierarchy_view_data::ProjectHierarchyViewData;
use eframe::egui::{ComboBox, Response, Sense, TextEdit, Ui, UiBuilder, Widget, vec2};
use epaint::{CornerRadius, Stroke, StrokeKind};
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
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
        92.0
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

            user_interface.scope_builder(builder, |user_interface| {
                user_interface.vertical(|user_interface| {
                    user_interface.horizontal_wrapped(|user_interface| {
                        user_interface.label("Target");
                        user_interface.add_sized(
                            [160.0, 24.0],
                            TextEdit::singleline(&mut pointer_scanner_view_data.target_address_input).hint_text("0x1234"),
                        );
                        user_interface.label("Pointer Size");
                        ComboBox::from_id_salt("pointer_scanner_pointer_size")
                            .selected_text(pointer_scanner_view_data.pointer_size.to_string())
                            .show_ui(user_interface, |user_interface| {
                                user_interface.selectable_value(&mut pointer_scanner_view_data.pointer_size, PointerScanPointerSize::Pointer32, "u32");
                                user_interface.selectable_value(&mut pointer_scanner_view_data.pointer_size, PointerScanPointerSize::Pointer64, "u64");
                            });
                        user_interface.label("Depth");
                        user_interface.add_sized([56.0, 24.0], TextEdit::singleline(&mut pointer_scanner_view_data.max_depth_input));
                        user_interface.label("Radius");
                        user_interface.add_sized([72.0, 24.0], TextEdit::singleline(&mut pointer_scanner_view_data.offset_radius_input));

                        if user_interface.button("Start").clicked() {
                            should_start_scan = true;
                        }

                        if user_interface.button("Refresh").clicked() {
                            should_refresh_summary = true;
                        }
                    });

                    user_interface.horizontal_wrapped(|user_interface| {
                        user_interface.label("Validate");
                        user_interface.add_sized(
                            [160.0, 24.0],
                            TextEdit::singleline(&mut pointer_scanner_view_data.validation_target_address_input).hint_text("0x1234"),
                        );

                        if user_interface.button("Validate").clicked() {
                            should_validate_scan = true;
                        }

                        if user_interface.button("Copy Chain").clicked() {
                            should_copy_chain = true;
                        }

                        if user_interface.button("Export").clicked() {
                            should_export_chain = true;
                        }

                        if user_interface.button("Add To Project").clicked() {
                            should_add_to_project = true;
                        }

                        user_interface.separator();
                        user_interface.label(&pointer_scanner_view_data.status_message);
                    });
                });
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
