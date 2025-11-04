use crate::app_context::AppContext;
use crate::ui::widgets::controls::combo_box::combo_box_view::ComboBoxView;
use crate::ui::widgets::controls::scan_constraint_selector::scan_compare_type_item_view::ScanCompareTypeItemView;
use eframe::egui::{Align, Id, Layout, Response, RichText, Sense, Ui, UiBuilder, Widget};
use epaint::{CornerRadius, vec2};
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use std::sync::Arc;

/// A widget that allows selecting from a set of data types.
pub struct ScanCompareTypeSelectorView {
    app_context: Arc<AppContext>,
    active_scan_compare_type: ScanCompareType,
}

impl ScanCompareTypeSelectorView {
    pub fn new(
        app_context: Arc<AppContext>,
        active_scan_compare_type: ScanCompareType,
    ) -> Self {
        Self {
            app_context,
            active_scan_compare_type,
        }
    }

    pub fn close(
        &self,
        user_interface: &mut Ui,
    ) {
        let popup_id = Id::new(("data_type_selector_popup", user_interface.id().value()));

        user_interface.memory_mut(|memory| {
            memory.data.insert_temp(popup_id, false);
        });
    }

    fn create_header(
        &self,
        user_interface: &mut Ui,
        label: &str,
        width: f32,
    ) {
        let (allocated_size_rectangle, _response) = user_interface.allocate_exact_size(vec2(width, 32.0), Sense::empty());
        let theme = &self.app_context.theme;

        // Background highlight if this is the actively dragged window.
        let background = theme.background_panel;
        user_interface
            .painter()
            .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, background);

        // Child UI for layouting contents.
        let builder = UiBuilder::new()
            .max_rect(allocated_size_rectangle)
            .layout(Layout::left_to_right(Align::Center));
        let mut child_user_interface = user_interface.new_child(builder);

        child_user_interface.set_clip_rect(allocated_size_rectangle);

        // Title text.
        child_user_interface.add_space(8.0);

        child_user_interface.label(
            RichText::new(label)
                .color(theme.foreground)
                .font(theme.font_library.font_noto_sans.font_window_title.clone()),
        );
    }
}

impl Widget for ScanCompareTypeSelectorView {
    fn ui(
        self,
        ui: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let element_width_left = 160.0;
        let element_width_right = 204.0;
        let total_row_width = element_width_left + element_width_right;

        // Build the combo box widget first
        let combo_box = ComboBoxView::new(self.app_context.clone(), "", None, |popup_user_interface: &mut Ui, should_close: &mut bool| {
            popup_user_interface.vertical(|user_interface| {
                self.create_header(user_interface, "Relative", total_row_width);

                user_interface.horizontal(|user_interface| {
                    if user_interface
                        .add(ScanCompareTypeItemView::new(
                            self.app_context.clone(),
                            "Changed",
                            Some(theme.icon_library.icon_handle_scan_relative_changed.clone()),
                            element_width_left,
                        ))
                        .clicked()
                    {
                        *should_close = true;
                    };

                    if user_interface
                        .add(ScanCompareTypeItemView::new(
                            self.app_context.clone(),
                            "Unchanged",
                            Some(theme.icon_library.icon_handle_scan_relative_unchanged.clone()),
                            element_width_right,
                        ))
                        .clicked()
                    {
                        *should_close = true;
                    };
                });

                user_interface.horizontal(|user_interface| {
                    if user_interface
                        .add(ScanCompareTypeItemView::new(
                            self.app_context.clone(),
                            "Increased",
                            Some(theme.icon_library.icon_handle_scan_relative_increased.clone()),
                            element_width_left,
                        ))
                        .clicked()
                    {
                        *should_close = true;
                    };

                    if user_interface
                        .add(ScanCompareTypeItemView::new(
                            self.app_context.clone(),
                            "Decreased",
                            Some(theme.icon_library.icon_handle_scan_relative_decreased.clone()),
                            element_width_right,
                        ))
                        .clicked()
                    {
                        *should_close = true;
                    };
                });

                self.create_header(user_interface, "Immediate", total_row_width);

                user_interface.horizontal(|user_interface| {
                    if user_interface
                        .add(ScanCompareTypeItemView::new(
                            self.app_context.clone(),
                            "Equal",
                            Some(theme.icon_library.icon_handle_scan_immediate_equal.clone()),
                            element_width_left,
                        ))
                        .clicked()
                    {
                        *should_close = true;
                    };

                    if user_interface
                        .add(ScanCompareTypeItemView::new(
                            self.app_context.clone(),
                            "Not Equal",
                            Some(theme.icon_library.icon_handle_scan_immediate_not_equal.clone()),
                            element_width_right,
                        ))
                        .clicked()
                    {
                        *should_close = true;
                    };
                });
                user_interface.horizontal(|user_interface| {
                    if user_interface
                        .add(ScanCompareTypeItemView::new(
                            self.app_context.clone(),
                            "Greater Than",
                            Some(
                                theme
                                    .icon_library
                                    .icon_handle_scan_immediate_greater_than
                                    .clone(),
                            ),
                            element_width_left,
                        ))
                        .clicked()
                    {
                        *should_close = true;
                    };

                    if user_interface
                        .add(ScanCompareTypeItemView::new(
                            self.app_context.clone(),
                            "Greater Than or Equal to",
                            Some(
                                theme
                                    .icon_library
                                    .icon_handle_scan_immediate_greater_than_or_equal
                                    .clone(),
                            ),
                            element_width_right,
                        ))
                        .clicked()
                    {
                        *should_close = true;
                    };
                });
                user_interface.horizontal(|user_interface| {
                    if user_interface
                        .add(ScanCompareTypeItemView::new(
                            self.app_context.clone(),
                            "Less Than",
                            Some(theme.icon_library.icon_handle_scan_immediate_less_than.clone()),
                            element_width_left,
                        ))
                        .clicked()
                    {
                        *should_close = true;
                    };

                    if user_interface
                        .add(ScanCompareTypeItemView::new(
                            self.app_context.clone(),
                            "Less Than or Equal to",
                            Some(
                                theme
                                    .icon_library
                                    .icon_handle_scan_immediate_less_than_or_equal
                                    .clone(),
                            ),
                            element_width_right,
                        ))
                        .clicked()
                    {
                        *should_close = true;
                    };
                });

                self.create_header(user_interface, "Delta", total_row_width);

                user_interface.horizontal(|user_interface| {
                    if user_interface
                        .add(ScanCompareTypeItemView::new(
                            self.app_context.clone(),
                            "Increased by x",
                            Some(theme.icon_library.icon_handle_scan_delta_increased_by_x.clone()),
                            element_width_left,
                        ))
                        .clicked()
                    {
                        *should_close = true;
                    };

                    if user_interface
                        .add(ScanCompareTypeItemView::new(
                            self.app_context.clone(),
                            "Decreased by x",
                            Some(theme.icon_library.icon_handle_scan_delta_decreased_by_x.clone()),
                            element_width_right,
                        ))
                        .clicked()
                    {
                        *should_close = true;
                    };
                });

                user_interface.horizontal(|user_interface| {
                    if user_interface
                        .add(ScanCompareTypeItemView::new(
                            self.app_context.clone(),
                            "Multiplied by x",
                            Some(
                                theme
                                    .icon_library
                                    .icon_handle_scan_delta_multiplied_by_x
                                    .clone(),
                            ),
                            element_width_left,
                        ))
                        .clicked()
                    {
                        *should_close = true;
                    };

                    if user_interface
                        .add(ScanCompareTypeItemView::new(
                            self.app_context.clone(),
                            "Divided by x",
                            Some(theme.icon_library.icon_handle_scan_delta_divided_by_x.clone()),
                            element_width_right,
                        ))
                        .clicked()
                    {
                        *should_close = true;
                    };
                });

                user_interface.horizontal(|user_interface| {
                    if user_interface
                        .add(ScanCompareTypeItemView::new(
                            self.app_context.clone(),
                            "Modulo by x",
                            Some(theme.icon_library.icon_handle_scan_delta_modulo_by_x.clone()),
                            element_width_left,
                        ))
                        .clicked()
                    {
                        *should_close = true;
                    };
                });

                self.create_header(user_interface, "Binary", total_row_width);

                user_interface.horizontal(|user_interface| {
                    if user_interface
                        .add(ScanCompareTypeItemView::new(
                            self.app_context.clone(),
                            "Shifted left by x",
                            Some(
                                theme
                                    .icon_library
                                    .icon_handle_scan_delta_shift_left_by_x
                                    .clone(),
                            ),
                            element_width_left,
                        ))
                        .clicked()
                    {
                        *should_close = true;
                    };

                    if user_interface
                        .add(ScanCompareTypeItemView::new(
                            self.app_context.clone(),
                            "Shifted right by x",
                            Some(
                                theme
                                    .icon_library
                                    .icon_handle_scan_delta_shift_right_by_x
                                    .clone(),
                            ),
                            element_width_right,
                        ))
                        .clicked()
                    {
                        *should_close = true;
                    };
                });

                user_interface.horizontal(|user_interface| {
                    if user_interface
                        .add(ScanCompareTypeItemView::new(
                            self.app_context.clone(),
                            "Logical AND'd by x",
                            Some(
                                theme
                                    .icon_library
                                    .icon_handle_scan_delta_logical_and_by_x
                                    .clone(),
                            ),
                            element_width_left,
                        ))
                        .clicked()
                    {
                        *should_close = true;
                    };

                    if user_interface
                        .add(ScanCompareTypeItemView::new(
                            self.app_context.clone(),
                            "Logical OR'd by x",
                            Some(
                                theme
                                    .icon_library
                                    .icon_handle_scan_delta_logical_or_by_x
                                    .clone(),
                            ),
                            element_width_right,
                        ))
                        .clicked()
                    {
                        *should_close = true;
                    };
                });

                user_interface.horizontal(|user_interface| {
                    if user_interface
                        .add(ScanCompareTypeItemView::new(
                            self.app_context.clone(),
                            "Logical XOR'd by x",
                            Some(
                                theme
                                    .icon_library
                                    .icon_handle_scan_delta_logical_xor_by_x
                                    .clone(),
                            ),
                            element_width_left,
                        ))
                        .clicked()
                    {
                        *should_close = true;
                    };
                });
            });
        })
        .width(128.0)
        .height(28.0);

        // Add the combo box to the layout
        ui.add(combo_box)
    }
}
