use crate::ui::widgets::controls::button::Button;
use crate::ui::widgets::controls::combo_box::combo_box_view::ComboBoxView;
use crate::ui::widgets::controls::data_type_selector::data_type_item_view::DataTypeItemView;
use crate::ui::widgets::controls::state_layer::StateLayer;
use crate::{app_context::AppContext, ui::draw::icon_draw::IconDraw};
use eframe::egui::{Align, Area, Frame, Id, Key, Layout, Order, Response, Sense, Ui, Widget};
use epaint::{Color32, CornerRadius, Rect, TextureHandle, Vec2, pos2, vec2};
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use std::sync::Arc;

/// A widget that allows selecting from a set of data types.
pub struct DataTypeSelectorView {
    app_context: Arc<AppContext>,
    active_data_type: DataTypeRef,
}

impl DataTypeSelectorView {
    pub fn new(
        app_context: Arc<AppContext>,
        active_data_type: DataTypeRef,
    ) -> Self {
        Self { app_context, active_data_type }
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
}

impl Widget for DataTypeSelectorView {
    fn ui(
        self,
        ui: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let element_width = 92.0;

        // Build the combo box widget first
        let combo_box = ComboBoxView::new(
            self.app_context.clone(),
            self.active_data_type.get_data_type_id(),
            None,
            |popup_ui: &mut Ui, should_close: &mut bool| {
                popup_ui.vertical(|user_interface| {
                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                "u8",
                                Some(theme.icon_library.icon_handle_data_type_purple_blocks_1.clone()),
                                element_width,
                            ))
                            .clicked()
                        {
                            *should_close = true;
                        };

                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                "i8",
                                Some(theme.icon_library.icon_handle_data_type_blue_blocks_1.clone()),
                                element_width,
                            ))
                            .clicked()
                        {
                            *should_close = true;
                        };
                    });
                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                "i16",
                                Some(theme.icon_library.icon_handle_data_type_blue_blocks_2.clone()),
                                element_width,
                            ))
                            .clicked()
                        {
                            *should_close = true;
                        };

                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                "i16 (BE)",
                                Some(
                                    theme
                                        .icon_library
                                        .icon_handle_data_type_blue_blocks_reverse_2
                                        .clone(),
                                ),
                                element_width,
                            ))
                            .clicked()
                        {
                            *should_close = true;
                        };
                    });
                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                "i32",
                                Some(theme.icon_library.icon_handle_data_type_blue_blocks_4.clone()),
                                element_width,
                            ))
                            .clicked()
                        {
                            *should_close = true;
                        };

                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                "i32 (BE)",
                                Some(
                                    theme
                                        .icon_library
                                        .icon_handle_data_type_blue_blocks_reverse_4
                                        .clone(),
                                ),
                                element_width,
                            ))
                            .clicked()
                        {
                            *should_close = true;
                        };
                    });
                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                "i64",
                                Some(theme.icon_library.icon_handle_data_type_blue_blocks_8.clone()),
                                element_width,
                            ))
                            .clicked()
                        {
                            *should_close = true;
                        };

                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                "i64 (BE)",
                                Some(
                                    theme
                                        .icon_library
                                        .icon_handle_data_type_blue_blocks_reverse_8
                                        .clone(),
                                ),
                                element_width,
                            ))
                            .clicked()
                        {
                            *should_close = true;
                        };
                    });
                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                "u16",
                                Some(theme.icon_library.icon_handle_data_type_purple_blocks_2.clone()),
                                element_width,
                            ))
                            .clicked()
                        {
                            *should_close = true;
                        };

                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                "u16 (BE)",
                                Some(
                                    theme
                                        .icon_library
                                        .icon_handle_data_type_purple_blocks_reverse_2
                                        .clone(),
                                ),
                                element_width,
                            ))
                            .clicked()
                        {
                            *should_close = true;
                        };
                    });
                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                "u32",
                                Some(theme.icon_library.icon_handle_data_type_purple_blocks_4.clone()),
                                element_width,
                            ))
                            .clicked()
                        {
                            *should_close = true;
                        };

                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                "u32 (BE)",
                                Some(
                                    theme
                                        .icon_library
                                        .icon_handle_data_type_purple_blocks_reverse_4
                                        .clone(),
                                ),
                                element_width,
                            ))
                            .clicked()
                        {
                            *should_close = true;
                        };
                    });
                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                "u64",
                                Some(theme.icon_library.icon_handle_data_type_purple_blocks_8.clone()),
                                element_width,
                            ))
                            .clicked()
                        {
                            *should_close = true;
                        };

                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                "u64 (BE)",
                                Some(
                                    theme
                                        .icon_library
                                        .icon_handle_data_type_purple_blocks_reverse_8
                                        .clone(),
                                ),
                                element_width,
                            ))
                            .clicked()
                        {
                            *should_close = true;
                        };
                    });
                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                "f32",
                                Some(theme.icon_library.icon_handle_data_type_orange_blocks_4.clone()),
                                element_width,
                            ))
                            .clicked()
                        {
                            *should_close = true;
                        };

                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                "f32 (BE)",
                                Some(
                                    theme
                                        .icon_library
                                        .icon_handle_data_type_orange_blocks_reverse_4
                                        .clone(),
                                ),
                                element_width,
                            ))
                            .clicked()
                        {
                            *should_close = true;
                        };
                    });
                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                "f64",
                                Some(theme.icon_library.icon_handle_data_type_orange_blocks_8.clone()),
                                element_width,
                            ))
                            .clicked()
                        {
                            *should_close = true;
                        };

                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                "f64 (BE)",
                                Some(
                                    theme
                                        .icon_library
                                        .icon_handle_data_type_orange_blocks_reverse_8
                                        .clone(),
                                ),
                                element_width,
                            ))
                            .clicked()
                        {
                            *should_close = true;
                        };
                    });
                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                "String...",
                                Some(theme.icon_library.icon_handle_data_type_string.clone()),
                                element_width,
                            ))
                            .clicked()
                        {
                            *should_close = true;
                        };

                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                "Custom...",
                                Some(
                                    theme
                                        .icon_library
                                        .icon_handle_data_type_purple_blocks_array
                                        .clone(),
                                ),
                                element_width,
                            ))
                            .clicked()
                        {
                            *should_close = true;
                        };
                    });
                });
            },
        )
        .width(128.0)
        .height(28.0);

        // Add the combo box to the layout
        ui.add(combo_box)
    }
}
