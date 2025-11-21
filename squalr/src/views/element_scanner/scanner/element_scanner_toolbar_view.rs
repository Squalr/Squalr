use crate::{
    app_context::AppContext,
    ui::{
        draw::icon_draw::IconDraw,
        widgets::controls::{
            button::Button, data_type_selector::data_type_selector_view::DataTypeSelectorView, data_value_box::data_value_box_view::DataValueBoxView,
            scan_constraint_selector::scan_compare_type_selector_view::ScanCompareTypeSelectorView,
        },
    },
    views::element_scanner::scanner::view_data::element_scanner_view_data::ElementScannerViewData,
};
use eframe::egui::{Align, Layout, Response, Sense, Ui, UiBuilder, Widget};
use epaint::{Color32, CornerRadius, vec2};
use squalr_engine_api::{dependency_injection::dependency::Dependency, structures::scanning::comparisons::scan_compare_type::ScanCompareType};
use std::sync::Arc;

#[derive(Clone)]
pub struct ElementScannerToolbarView {
    app_context: Arc<AppContext>,
    element_scanner_view_data: Dependency<ElementScannerViewData>,
}

impl ElementScannerToolbarView {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let element_scanner_view_data = app_context
            .dependency_container
            .get_dependency::<ElementScannerViewData>();
        let instance = Self {
            app_context,
            element_scanner_view_data,
        };

        instance
    }

    pub fn get_height(&self) -> f32 {
        68.0
    }
}

impl Widget for ElementScannerToolbarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let height = self.get_height();
        let row_height = height * 0.5;

        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), height), Sense::hover());

        // Background.
        user_interface
            .painter()
            .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.background_primary);

        // Create a child UI constrained to the title bar.
        let builder = UiBuilder::new()
            .max_rect(allocated_size_rectangle)
            .layout(Layout::top_down(Align::Min));

        let mut toolbar_user_interface = user_interface.new_child(builder);

        let mut element_scanner_view_data = match self.element_scanner_view_data.write() {
            Ok(data) => data,
            Err(error) => {
                log::error!("Failed to acquire element scanner view data: {}", error);
                return response;
            }
        };

        let button_size = vec2(36.0, 28.0);
        let mut should_perform_new_scan = false;
        let mut should_collect_values = false;
        let mut should_start_scan = false;

        // Top row.
        toolbar_user_interface.allocate_ui(vec2(toolbar_user_interface.available_width(), row_height), |user_interface| {
            user_interface.with_layout(Layout::left_to_right(Align::Center), |user_interface| {
                // Data type selector.
                user_interface.add_space(8.0);
                user_interface.add(DataTypeSelectorView::new(
                    self.app_context.clone(),
                    &mut element_scanner_view_data.selected_data_type,
                ));

                // Scan compare type selector.
                user_interface.add_space(8.0);
                user_interface.add(ScanCompareTypeSelectorView::new(
                    self.app_context.clone(),
                    &mut element_scanner_view_data.selected_scan_compare_type,
                ));

                // Collect values.
                let button_collect_values = user_interface.add_sized(
                    button_size,
                    Button::new_from_theme(theme)
                        .background_color(Color32::TRANSPARENT)
                        .tooltip_text("Collect values."),
                );
                IconDraw::draw(user_interface, button_collect_values.rect, &theme.icon_library.icon_handle_scan_collect_values);

                if button_collect_values.clicked() {
                    should_collect_values = true;
                }

                // New scan.
                let button_new_scan = user_interface.add_sized(
                    button_size,
                    Button::new_from_theme(theme)
                        .background_color(Color32::TRANSPARENT)
                        .tooltip_text("New scan."),
                );
                IconDraw::draw(user_interface, button_new_scan.rect, &theme.icon_library.icon_handle_scan_new);

                if button_new_scan.clicked() {
                    should_perform_new_scan = true;
                }
            });
        });

        // Bottom row.
        toolbar_user_interface.allocate_ui(vec2(toolbar_user_interface.available_width(), row_height), |user_interface| {
            user_interface.with_layout(Layout::left_to_right(Align::Center), |user_interface| {
                // Scan value (primary).
                match &element_scanner_view_data.selected_scan_compare_type {
                    ScanCompareType::Relative(_) => {
                        // Nothing to display for relative scans.
                    }
                    _ => {
                        let data_type_ref = element_scanner_view_data.selected_data_type.clone();

                        user_interface.add_space(8.0);
                        user_interface.add(DataValueBoxView::new(
                            self.app_context.clone(),
                            &mut element_scanner_view_data.current_scan_value,
                            &data_type_ref,
                            false,
                            true,
                            "Enter a scan value...",
                            "data_value_box_scan_value",
                        ));
                    }
                }

                // Scan value (upper limit).
                match &element_scanner_view_data.selected_scan_compare_type {
                    ScanCompareType::Relative(_) => {
                        // Nothing to display for relative scans.
                    }
                    ScanCompareType::Delta(_) | ScanCompareType::Immediate(_) => {
                        // JIRA: Temp disabled until we actually support a max scan value.
                    }
                    _ => {
                        let data_type_ref = element_scanner_view_data.selected_data_type.clone();

                        user_interface.add_space(8.0);
                        user_interface.add(DataValueBoxView::new(
                            self.app_context.clone(),
                            &mut element_scanner_view_data.max_scan_value,
                            &data_type_ref,
                            false,
                            true,
                            "Enter a max scan value...",
                            "data_value_box_scan_value_upper_limit",
                        ));
                    }
                }

                user_interface.add_space(8.0);

                // Start scan.
                let button_start_scan = user_interface.add_sized(
                    button_size,
                    Button::new_from_theme(theme)
                        .background_color(Color32::TRANSPARENT)
                        .tooltip_text("Start scan."),
                );
                IconDraw::draw(user_interface, button_start_scan.rect, &theme.icon_library.icon_handle_navigation_right_arrow);

                if button_start_scan.clicked() {
                    should_start_scan = true;
                }
            });
        });

        drop(element_scanner_view_data);

        if should_perform_new_scan {
            ElementScannerViewData::reset_scan(self.element_scanner_view_data.clone(), self.app_context.engine_execution_context.clone());
        } else if should_collect_values {
            ElementScannerViewData::collect_values(self.app_context.engine_execution_context.clone());
        } else if should_start_scan {
            ElementScannerViewData::start_scan(self.element_scanner_view_data.clone(), self.app_context.engine_execution_context.clone());
        }

        response
    }
}
