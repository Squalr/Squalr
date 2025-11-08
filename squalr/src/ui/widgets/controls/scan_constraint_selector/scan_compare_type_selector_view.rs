use crate::ui::widgets::controls::combo_box::combo_box_view::ComboBoxView;
use crate::ui::widgets::controls::scan_constraint_selector::scan_compare_type_item_view::ScanCompareTypeItemView;
use crate::{app_context::AppContext, ui::converters::scan_compare_type_to_icon_converter::ScanCompareTypeToIconConverter};
use eframe::egui::{Align, Id, Layout, Response, RichText, Sense, Ui, UiBuilder, Widget};
use epaint::{CornerRadius, vec2};
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_delta::ScanCompareTypeDelta;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_relative::ScanCompareTypeRelative;
use std::sync::Arc;

/// A widget that allows selecting from a set of data types.
pub struct ScanCompareTypeSelectorView<'lifetime> {
    app_context: Arc<AppContext>,
    active_scan_compare_type: &'lifetime mut ScanCompareType,
}

impl<'lifetime> ScanCompareTypeSelectorView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        active_scan_compare_type: &'lifetime mut ScanCompareType,
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

impl<'lifetime> Widget for ScanCompareTypeSelectorView<'lifetime> {
    fn ui(
        self,
        ui: &mut Ui,
    ) -> Response {
        let app_context = self.app_context.clone();
        let theme = &app_context.theme;
        let icon_library = &theme.icon_library;
        let element_width_left = 160.0;
        let element_width_right = 204.0;
        let total_row_width = element_width_left + element_width_right;
        let selected_icon = ScanCompareTypeToIconConverter::convert_scan_compare_type_to_icon(&self.active_scan_compare_type, icon_library);

        let combo_box = ComboBoxView::new(
            self.app_context.clone(),
            "",
            Some(selected_icon),
            |popup_user_interface: &mut Ui, should_close: &mut bool| {
                popup_user_interface.vertical(|user_interface| {
                    self.create_header(user_interface, "Relative", total_row_width);

                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(ScanCompareTypeItemView::new(
                                self.app_context.clone(),
                                "Changed",
                                Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_relative_to_icon(
                                    &ScanCompareTypeRelative::Changed,
                                    icon_library,
                                )),
                                element_width_left,
                            ))
                            .clicked()
                        {
                            *self.active_scan_compare_type = ScanCompareType::Relative(ScanCompareTypeRelative::Changed);
                            *should_close = true;
                        };

                        if user_interface
                            .add(ScanCompareTypeItemView::new(
                                self.app_context.clone(),
                                "Unchanged",
                                Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_relative_to_icon(
                                    &ScanCompareTypeRelative::Unchanged,
                                    icon_library,
                                )),
                                element_width_right,
                            ))
                            .clicked()
                        {
                            *self.active_scan_compare_type = ScanCompareType::Relative(ScanCompareTypeRelative::Unchanged);
                            *should_close = true;
                        };
                    });

                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(ScanCompareTypeItemView::new(
                                self.app_context.clone(),
                                "Increased",
                                Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_relative_to_icon(
                                    &ScanCompareTypeRelative::Increased,
                                    icon_library,
                                )),
                                element_width_left,
                            ))
                            .clicked()
                        {
                            *self.active_scan_compare_type = ScanCompareType::Relative(ScanCompareTypeRelative::Increased);
                            *should_close = true;
                        };

                        if user_interface
                            .add(ScanCompareTypeItemView::new(
                                self.app_context.clone(),
                                "Decreased",
                                Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_relative_to_icon(
                                    &ScanCompareTypeRelative::Decreased,
                                    icon_library,
                                )),
                                element_width_right,
                            ))
                            .clicked()
                        {
                            *self.active_scan_compare_type = ScanCompareType::Relative(ScanCompareTypeRelative::Decreased);
                            *should_close = true;
                        };
                    });

                    self.create_header(user_interface, "Immediate", total_row_width);

                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(ScanCompareTypeItemView::new(
                                self.app_context.clone(),
                                "Equal",
                                Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_immediate_to_icon(
                                    &ScanCompareTypeImmediate::Equal,
                                    icon_library,
                                )),
                                element_width_left,
                            ))
                            .clicked()
                        {
                            *self.active_scan_compare_type = ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal);
                            *should_close = true;
                        };

                        if user_interface
                            .add(ScanCompareTypeItemView::new(
                                self.app_context.clone(),
                                "Not Equal",
                                Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_immediate_to_icon(
                                    &ScanCompareTypeImmediate::NotEqual,
                                    icon_library,
                                )),
                                element_width_right,
                            ))
                            .clicked()
                        {
                            *self.active_scan_compare_type = ScanCompareType::Immediate(ScanCompareTypeImmediate::NotEqual);
                            *should_close = true;
                        };
                    });
                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(ScanCompareTypeItemView::new(
                                self.app_context.clone(),
                                "Greater Than",
                                Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_immediate_to_icon(
                                    &ScanCompareTypeImmediate::GreaterThan,
                                    icon_library,
                                )),
                                element_width_left,
                            ))
                            .clicked()
                        {
                            *self.active_scan_compare_type = ScanCompareType::Immediate(ScanCompareTypeImmediate::GreaterThan);
                            *should_close = true;
                        };

                        if user_interface
                            .add(ScanCompareTypeItemView::new(
                                self.app_context.clone(),
                                "Greater Than or Equal to",
                                Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_immediate_to_icon(
                                    &ScanCompareTypeImmediate::GreaterThanOrEqual,
                                    icon_library,
                                )),
                                element_width_right,
                            ))
                            .clicked()
                        {
                            *self.active_scan_compare_type = ScanCompareType::Immediate(ScanCompareTypeImmediate::GreaterThanOrEqual);
                            *should_close = true;
                        };
                    });
                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(ScanCompareTypeItemView::new(
                                self.app_context.clone(),
                                "Less Than",
                                Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_immediate_to_icon(
                                    &ScanCompareTypeImmediate::LessThan,
                                    icon_library,
                                )),
                                element_width_left,
                            ))
                            .clicked()
                        {
                            *self.active_scan_compare_type = ScanCompareType::Immediate(ScanCompareTypeImmediate::LessThan);
                            *should_close = true;
                        };

                        if user_interface
                            .add(ScanCompareTypeItemView::new(
                                self.app_context.clone(),
                                "Less Than or Equal to",
                                Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_immediate_to_icon(
                                    &ScanCompareTypeImmediate::LessThanOrEqual,
                                    icon_library,
                                )),
                                element_width_right,
                            ))
                            .clicked()
                        {
                            *self.active_scan_compare_type = ScanCompareType::Immediate(ScanCompareTypeImmediate::LessThanOrEqual);
                            *should_close = true;
                        };
                    });

                    self.create_header(user_interface, "Delta", total_row_width);

                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(ScanCompareTypeItemView::new(
                                self.app_context.clone(),
                                "Increased by x",
                                Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_delta_to_icon(
                                    &ScanCompareTypeDelta::IncreasedByX,
                                    icon_library,
                                )),
                                element_width_left,
                            ))
                            .clicked()
                        {
                            *self.active_scan_compare_type = ScanCompareType::Delta(ScanCompareTypeDelta::IncreasedByX);
                            *should_close = true;
                        };

                        if user_interface
                            .add(ScanCompareTypeItemView::new(
                                self.app_context.clone(),
                                "Decreased by x",
                                Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_delta_to_icon(
                                    &ScanCompareTypeDelta::DecreasedByX,
                                    icon_library,
                                )),
                                element_width_right,
                            ))
                            .clicked()
                        {
                            *self.active_scan_compare_type = ScanCompareType::Delta(ScanCompareTypeDelta::DecreasedByX);
                            *should_close = true;
                        };
                    });

                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(ScanCompareTypeItemView::new(
                                self.app_context.clone(),
                                "Multiplied by x",
                                Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_delta_to_icon(
                                    &ScanCompareTypeDelta::MultipliedByX,
                                    icon_library,
                                )),
                                element_width_left,
                            ))
                            .clicked()
                        {
                            *self.active_scan_compare_type = ScanCompareType::Delta(ScanCompareTypeDelta::MultipliedByX);
                            *should_close = true;
                        };

                        if user_interface
                            .add(ScanCompareTypeItemView::new(
                                self.app_context.clone(),
                                "Divided by x",
                                Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_delta_to_icon(
                                    &ScanCompareTypeDelta::DividedByX,
                                    icon_library,
                                )),
                                element_width_right,
                            ))
                            .clicked()
                        {
                            *self.active_scan_compare_type = ScanCompareType::Delta(ScanCompareTypeDelta::DividedByX);
                            *should_close = true;
                        };
                    });

                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(ScanCompareTypeItemView::new(
                                self.app_context.clone(),
                                "Modulo by x",
                                Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_delta_to_icon(
                                    &ScanCompareTypeDelta::ModuloByX,
                                    icon_library,
                                )),
                                element_width_left,
                            ))
                            .clicked()
                        {
                            *self.active_scan_compare_type = ScanCompareType::Delta(ScanCompareTypeDelta::ModuloByX);
                            *should_close = true;
                        };
                    });

                    self.create_header(user_interface, "Binary", total_row_width);

                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(ScanCompareTypeItemView::new(
                                self.app_context.clone(),
                                "Shifted left by x",
                                Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_delta_to_icon(
                                    &ScanCompareTypeDelta::ShiftLeftByX,
                                    icon_library,
                                )),
                                element_width_left,
                            ))
                            .clicked()
                        {
                            *self.active_scan_compare_type = ScanCompareType::Delta(ScanCompareTypeDelta::ShiftLeftByX);
                            *should_close = true;
                        };

                        if user_interface
                            .add(ScanCompareTypeItemView::new(
                                self.app_context.clone(),
                                "Shifted right by x",
                                Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_delta_to_icon(
                                    &ScanCompareTypeDelta::ShiftRightByX,
                                    icon_library,
                                )),
                                element_width_right,
                            ))
                            .clicked()
                        {
                            *self.active_scan_compare_type = ScanCompareType::Delta(ScanCompareTypeDelta::ShiftRightByX);
                            *should_close = true;
                        };
                    });

                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(ScanCompareTypeItemView::new(
                                self.app_context.clone(),
                                "Logical AND'd by x",
                                Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_delta_to_icon(
                                    &ScanCompareTypeDelta::LogicalAndByX,
                                    icon_library,
                                )),
                                element_width_left,
                            ))
                            .clicked()
                        {
                            *self.active_scan_compare_type = ScanCompareType::Delta(ScanCompareTypeDelta::LogicalAndByX);
                            *should_close = true;
                        };

                        if user_interface
                            .add(ScanCompareTypeItemView::new(
                                self.app_context.clone(),
                                "Logical OR'd by x",
                                Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_delta_to_icon(
                                    &ScanCompareTypeDelta::LogicalOrByX,
                                    icon_library,
                                )),
                                element_width_right,
                            ))
                            .clicked()
                        {
                            *self.active_scan_compare_type = ScanCompareType::Delta(ScanCompareTypeDelta::LogicalOrByX);
                            *should_close = true;
                        };
                    });

                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(ScanCompareTypeItemView::new(
                                self.app_context.clone(),
                                "Logical XOR'd by x",
                                Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_delta_to_icon(
                                    &ScanCompareTypeDelta::LogicalXorByX,
                                    icon_library,
                                )),
                                element_width_left,
                            ))
                            .clicked()
                        {
                            *self.active_scan_compare_type = ScanCompareType::Delta(ScanCompareTypeDelta::LogicalXorByX);
                            *should_close = true;
                        };
                    });
                });
            },
        )
        .width(68.0)
        .height(28.0);

        // Add the combo box to the layout
        ui.add(combo_box)
    }
}
