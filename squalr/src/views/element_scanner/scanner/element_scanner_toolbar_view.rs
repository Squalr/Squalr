use crate::{
    app_context::AppContext,
    ui::{
        draw::icon_draw::IconDraw,
        widgets::controls::{
            button::Button,
            combo_box::{combo_box_item_view::ComboBoxItemView, combo_box_view::ComboBoxView},
            data_type_selector::data_type_selector_view::DataTypeSelectorView,
            data_value_box::data_value_box_view::DataValueBoxView,
            scan_constraint_selector::scan_compare_type_selector_view::ScanCompareTypeSelectorView,
        },
    },
    views::element_scanner::scanner::view_data::element_scanner_view_data::{ElementScannerContainerMode, ElementScannerViewData},
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
    const DEFAULT_TOP_ROW_HEIGHT: f32 = 34.0;
    const DEFAULT_CONSTRAINT_ROW_HEIGHT: f32 = 34.0;
    const INSTRUCTION_CONSTRAINT_ROW_HEIGHT: f32 = 74.0;

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
        Self::DEFAULT_TOP_ROW_HEIGHT
    }

    pub fn get_constraint_row_height(&self) -> f32 {
        self.element_scanner_view_data
            .read("Element scanner toolbar view get constraint row height")
            .map(|element_scanner_view_data| {
                if ElementScannerViewData::is_instruction_sequence_data_type(
                    element_scanner_view_data
                        .data_type_selection
                        .visible_data_type(),
                ) {
                    Self::INSTRUCTION_CONSTRAINT_ROW_HEIGHT
                } else {
                    Self::DEFAULT_CONSTRAINT_ROW_HEIGHT
                }
            })
            .unwrap_or(Self::DEFAULT_CONSTRAINT_ROW_HEIGHT)
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

        let (allocated_size_rectangle, response) =
            user_interface.allocate_exact_size(vec2(user_interface.available_width().max(1.0), total_height), Sense::hover());

        // Background.
        user_interface
            .painter()
            .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.background_primary);

        // Create a child UI constrained to the title bar.
        let builder = UiBuilder::new()
            .max_rect(allocated_size_rectangle)
            .layout(Layout::top_down(Align::Min));

        let mut toolbar_user_interface = user_interface.new_child(builder);
        let available_data_types = self
            .app_context
            .engine_unprivileged_state
            .get_registered_data_type_refs();

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
        let mut selected_container_mode: Option<ElementScannerContainerMode> = None;
        let is_data_type_selection_disabled = element_scanner_view_data.view_state.has_active_scan();

        let visible_data_type_ref = element_scanner_view_data
            .data_type_selection
            .visible_data_type()
            .clone();
        let is_instruction_sequence_data_type = ElementScannerViewData::is_instruction_sequence_data_type(&visible_data_type_ref);
        let effective_container_mode =
            ElementScannerViewData::resolve_container_mode_for_data_type(&visible_data_type_ref, element_scanner_view_data.container_mode);
        element_scanner_view_data.container_mode = effective_container_mode;
        element_scanner_view_data.active_display_format = self
            .app_context
            .engine_unprivileged_state
            .resolve_supported_anonymous_value_string_format(&visible_data_type_ref, element_scanner_view_data.active_display_format);

        for scan_value_and_constraint in &mut element_scanner_view_data.scan_values_and_constraints {
            self.app_context
                .engine_unprivileged_state
                .normalize_anonymous_value_string_format(&visible_data_type_ref, &mut scan_value_and_constraint.current_scan_value);
        }

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
                user_interface.add(
                    DataTypeSelectorView::new(
                        self.app_context.clone(),
                        &mut element_scanner_view_data.data_type_selection,
                        "element_scanner_data_type_selector",
                    )
                    .enforce_format_compatibility()
                    .disabled(is_data_type_selection_disabled)
                    .available_data_types(available_data_types.clone()),
                );

                // Container type selector.
                user_interface.add_space(8.0);
                user_interface.add(
                    ComboBoxView::new(
                        self.app_context.clone(),
                        ElementScannerViewData::get_container_mode_label(&visible_data_type_ref, effective_container_mode),
                        "element_scanner_container_mode",
                        None,
                        |popup_user_interface: &mut Ui, should_close: &mut bool| {
                            for mode in ElementScannerContainerMode::ALL {
                                let item_response = popup_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), mode.label(), None, 100.0));

                                if item_response.clicked() {
                                    selected_container_mode = Some(*mode);
                                    *should_close = true;
                                }
                            }
                        },
                    )
                    .width(100.0)
                    .height(28.0)
                    .disabled(is_instruction_sequence_data_type),
                );

                if let Some(new_mode) = selected_container_mode {
                    element_scanner_view_data.container_mode = new_mode;
                }

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

        let selected_data_type = visible_data_type_ref;
        let selected_container_mode = effective_container_mode;
        let scan_value_placeholder = if is_instruction_sequence_data_type {
            "Enter instructions. Use Shift+Enter or Enter for new lines."
        } else {
            "Enter a value..."
        };

        // Constraint rows.
        for index in 0..element_scanner_view_data.scan_values_and_constraints.len() {
            let scan_values_and_constraint = &mut element_scanner_view_data.scan_values_and_constraints[index];
            ElementScannerViewData::apply_container_mode_to_constraint_value(selected_container_mode, &mut scan_values_and_constraint.current_scan_value);

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
                            user_interface.add_space(8.0);
                            user_interface.add(
                                DataValueBoxView::new(
                                    self.app_context.clone(),
                                    &mut scan_values_and_constraint.current_scan_value,
                                    &selected_data_type,
                                    false,
                                    true,
                                    scan_value_placeholder,
                                    &format!("data_value_box_scan_value_index_{}", index),
                                )
                                .validation_scan_compare_type(scan_values_and_constraint.selected_scan_compare_type)
                                .height(if is_instruction_sequence_data_type {
                                    constraint_row_height - 6.0
                                } else {
                                    28.0
                                })
                                .multiline(is_instruction_sequence_data_type)
                                .multiline_rows(if is_instruction_sequence_data_type { 3 } else { 1 }),
                            );
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
                            &theme.icon_library.icon_handle_common_remove,
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
            ElementScannerViewData::collect_values(self.element_scanner_view_data.clone(), self.app_context.engine_unprivileged_state.clone());
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
