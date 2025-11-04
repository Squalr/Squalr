use crate::{
    app_context::AppContext,
    ui::{
        draw::icon_draw::IconDraw,
        widgets::controls::{
            button::Button, data_type_selector::data_type_selector_view::DataTypeSelectorView,
            scan_constraint_selector::scan_compare_type_selector_view::ScanCompareTypeSelectorView,
        },
    },
    views::element_scanner::element_scanner_view_data::ElementScannerViewData,
};
use eframe::egui::{Align, Layout, Response, Sense, Ui, UiBuilder, Widget};
use epaint::{Color32, CornerRadius, vec2};
use squalr_engine_api::dependency_injection::dependency::Dependency;
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
}

impl Widget for ElementScannerToolbarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let height = 28.0;
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), height), Sense::empty());
        let theme = &self.app_context.theme;
        let element_scanner_view_data = match self.element_scanner_view_data.read() {
            Ok(element_scanner_view_data) => element_scanner_view_data,
            Err(error) => {
                log::error!("Failed to acquire element scanner view data: {}", error);

                return response;
            }
        };

        user_interface
            .painter()
            .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.background_primary);

        // Create a child ui constrained to the title bar.
        let builder = UiBuilder::new()
            .max_rect(allocated_size_rectangle)
            .layout(Layout::left_to_right(Align::Center));
        let mut toolbar_user_interface = user_interface.new_child(builder);

        toolbar_user_interface.with_layout(Layout::left_to_right(Align::Center), |user_interface| {
            let button_size = vec2(36.0, 28.0);

            // New scan.
            let button_new_scan = user_interface.add_sized(
                button_size,
                Button::new_from_theme(&theme)
                    .background_color(Color32::TRANSPARENT)
                    .tooltip_text("New scan."),
            );
            IconDraw::draw(user_interface, button_new_scan.rect, &theme.icon_library.icon_handle_scan_new);

            if button_new_scan.clicked() {
                //
            }

            // Collect values.
            let button_collect_values = user_interface.add_sized(
                button_size,
                Button::new_from_theme(&theme)
                    .background_color(Color32::TRANSPARENT)
                    .tooltip_text("Collect values."),
            );
            IconDraw::draw(user_interface, button_collect_values.rect, &theme.icon_library.icon_handle_scan_collect_values);

            if button_collect_values.clicked() {
                //
            }

            user_interface.add(ScanCompareTypeSelectorView::new(
                self.app_context.clone(),
                element_scanner_view_data.selected_scan_compare_type.clone(),
            ));

            user_interface.add(DataTypeSelectorView::new(
                self.app_context.clone(),
                element_scanner_view_data.selected_data_type.clone(),
            ));

            // Start scan.
            let button_start_scan = user_interface.add_sized(
                button_size,
                Button::new_from_theme(&theme)
                    .background_color(Color32::TRANSPARENT)
                    .tooltip_text("Start scan."),
            );
            IconDraw::draw(user_interface, button_start_scan.rect, &theme.icon_library.icon_handle_navigation_right_arrow);

            if button_start_scan.clicked() {
                //
            }
        });

        response
    }
}
