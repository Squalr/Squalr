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

    pub fn get_top_row_height(&self) -> f32 {
        34.0
    }

    pub fn get_constraint_row_height(&self) -> f32 {
        34.0
    }

    pub fn get_height(&self) -> f32 {
        let item_count = match self
            .element_scanner_view_data
            .read("Element scanner toolbar view get height")
        {
            Some(element_scanner_view_data) => element_scanner_view_data.scan_values_and_constraints.len(),
            None => 1,
        };

        self.get_constraint_row_height() * (item_count as f32) + self.get_top_row_height()
    }
}

impl Widget for ElementScannerToolbarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let total_height = self.get_height();
        let top_row_height = self.get_top_row_height();
        let constraint_row_height = self.get_constraint_row_height();

        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), total_height), Sense::hover());

        // Background.
        user_interface
            .painter()
            .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.background_primary);

        // Create a child UI constrained to the title bar.
        let builder = UiBuilder::new()
            .max_rect(allocated_size_rectangle)
            .layout(Layout::top_down(Align::Min));

        let mut toolbar_user_interface = user_interface.new_child(builder);

        let mut element_scanner_view_data = match self
            .element_scanner_view_data
            .write("Element scanner toolbar view")
        {
            Some(data) => data,
            None => return response,
        };

        let button_size = vec2(36.0, 28.0);
        let mut should_perform_new_scan = false;
        let mut should_collect_values = false;
        let mut should_start_scan = false;
        let mut should_add_new_scan_constraint = false;
        let mut remove_scan_constraint_index = 0;

        // Top row.
        toolbar_user_interface.allocate_ui(vec2(toolbar_user_interface.available_width(), top_row_height), |user_interface| {
            user_interface.with_layout(Layout::left_to_right(Align::Center), |user_interface| {
                // New scan.
                let button_new_scan = user_interface.add_sized(
                    button_size,
                    Button::new_from_theme(theme)
                        .background_color(Color32::TRANSPARENT)
                        .with_tooltip_text("New scan."),
                );
                IconDraw::draw(user_interface, button_new_scan.rect, &theme.icon_library.icon_handle_scan_new);

                if button_new_scan.clicked() {
                    should_perform_new_scan = true;
                }

                // Data type selector.
                user_interface.add_space(8.0);
                user_interface.add(DataTypeSelectorView::new(
                    self.app_context.clone(),
                    &mut element_scanner_view_data.selected_data_type,
                    "element_scanner_data_type_selector",
                ));

                // Collect values.
                let button_collect_values = user_interface.add_sized(
                    button_size,
                    Button::new_from_theme(theme)
                        .background_color(Color32::TRANSPARENT)
                        .with_tooltip_text("Collect values."),
                );
                IconDraw::draw(user_interface, button_collect_values.rect, &theme.icon_library.icon_handle_scan_collect_values);

                if button_collect_values.clicked() {
                    should_collect_values = true;
                }

                // Start scan.
                let button_start_scan = user_interface.add_sized(
                    button_size,
                    Button::new_from_theme(theme)
                        .background_color(Color32::TRANSPARENT)
                        .with_tooltip_text("Start scan."),
                );
                IconDraw::draw(user_interface, button_start_scan.rect, &theme.icon_library.icon_handle_navigation_right_arrow);

                if button_start_scan.clicked() {
                    should_start_scan = true;
                }
            });
        });

        let selected_data_type = &element_scanner_view_data.selected_data_type.clone();

        // Constraint rows.
        for index in 0..element_scanner_view_data.scan_values_and_constraints.len() {
            let scan_values_and_constraint = &mut element_scanner_view_data.scan_values_and_constraints[index];

            toolbar_user_interface.allocate_ui(vec2(toolbar_user_interface.available_width(), constraint_row_height), |user_interface| {
                user_interface.with_layout(Layout::left_to_right(Align::Center), |user_interface| {
                    // Scan compare type selector.
                    user_interface.add_space(8.0);
                    user_interface.add(ScanCompareTypeSelectorView::new(
                        self.app_context.clone(),
                        &mut scan_values_and_constraint.selected_scan_compare_type,
                        &scan_values_and_constraint.menu_id,
                    ));
                    // Scan value (primary).
                    match &scan_values_and_constraint.selected_scan_compare_type {
                        ScanCompareType::Relative(_) => {
                            // Nothing to display for relative scans.
                        }
                        _ => {
                            let data_type_ref = selected_data_type.clone();

                            user_interface.add_space(8.0);
                            user_interface.add(DataValueBoxView::new(
                                self.app_context.clone(),
                                &mut scan_values_and_constraint.current_scan_value,
                                &data_type_ref,
                                false,
                                true,
                                "Enter a scan value...",
                                &format!("data_value_box_scan_value_index_{}", index),
                            ));
                        }
                    }

                    if index == 0 {
                        let add_new_scan_constraint_button = user_interface.add_sized(
                            button_size,
                            Button::new_from_theme(theme)
                                .background_color(Color32::TRANSPARENT)
                                .with_tooltip_text("Add new scan constraint."),
                        );
                        IconDraw::draw(user_interface, add_new_scan_constraint_button.rect, &theme.icon_library.icon_handle_common_add);

                        if add_new_scan_constraint_button.clicked() {
                            should_add_new_scan_constraint = true;
                        }
                    } else {
                        let remove_scan_constraint_button = user_interface.add_sized(
                            button_size,
                            Button::new_from_theme(theme)
                                .background_color(Color32::TRANSPARENT)
                                .with_tooltip_text("Add new scan constraint."),
                        );
                        IconDraw::draw(
                            user_interface,
                            remove_scan_constraint_button.rect,
                            &theme.icon_library.icon_handle_common_delete,
                        );

                        if remove_scan_constraint_button.clicked() {
                            remove_scan_constraint_index = index;
                        }
                    }
                });
            });
        }

        drop(element_scanner_view_data);

        if should_perform_new_scan {
            ElementScannerViewData::reset_scan(self.element_scanner_view_data.clone(), self.app_context.engine_unprivileged_state.clone());
        } else if should_collect_values {
            ElementScannerViewData::collect_values(self.app_context.engine_unprivileged_state.clone());
        } else if should_start_scan {
            ElementScannerViewData::start_scan(self.element_scanner_view_data.clone(), self.app_context.engine_unprivileged_state.clone());
        } else if should_add_new_scan_constraint {
            ElementScannerViewData::add_constraint(self.element_scanner_view_data.clone());
        } else if remove_scan_constraint_index > 0 {
            ElementScannerViewData::remove_constraint(self.element_scanner_view_data.clone(), remove_scan_constraint_index);
        }

        response
    }
}
