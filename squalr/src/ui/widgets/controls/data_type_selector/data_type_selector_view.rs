use crate::ui::converters::data_type_to_string_converter::DataTypeToStringConverter;
use crate::ui::widgets::controls::combo_box::combo_box_view::ComboBoxView;
use crate::ui::widgets::controls::data_type_selector::data_type_item_view::DataTypeItemView;
use crate::{app_context::AppContext, ui::converters::data_type_to_icon_converter::DataTypeToIconConverter};
use eframe::egui::{Id, Response, Ui, Widget};
use squalr_engine_api::structures::data_types::{
    built_in_types::{
        f32::data_type_f32::DataTypeF32, f32be::data_type_f32be::DataTypeF32be, f64::data_type_f64::DataTypeF64, f64be::data_type_f64be::DataTypeF64be,
        i8::data_type_i8::DataTypeI8, i16::data_type_i16::DataTypeI16, i16be::data_type_i16be::DataTypeI16be, i32::data_type_i32::DataTypeI32,
        i32be::data_type_i32be::DataTypeI32be, i64::data_type_i64::DataTypeI64, i64be::data_type_i64be::DataTypeI64be, u8::data_type_u8::DataTypeU8,
        u16::data_type_u16::DataTypeU16, u16be::data_type_u16be::DataTypeU16be, u32::data_type_u32::DataTypeU32, u32be::data_type_u32be::DataTypeU32be,
        u64::data_type_u64::DataTypeU64, u64be::data_type_u64be::DataTypeU64be,
    },
    data_type_ref::DataTypeRef,
};
use std::sync::Arc;

/// A widget that allows selecting from a set of data types.
pub struct DataTypeSelectorView<'lifetime> {
    app_context: Arc<AppContext>,
    active_data_type: &'lifetime mut DataTypeRef,
    menu_id: &'lifetime str,
    width: f32,
    height: f32,
}

impl<'lifetime> DataTypeSelectorView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        active_data_type: &'lifetime mut DataTypeRef,
        menu_id: &'lifetime str,
    ) -> Self {
        Self {
            app_context,
            active_data_type,
            menu_id,
            width: 160.0,
            height: 28.0,
        }
    }

    pub fn width(
        mut self,
        width: f32,
    ) -> Self {
        self.width = width;
        self
    }

    pub fn height(
        mut self,
        height: f32,
    ) -> Self {
        self.height = height;
        self
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

impl<'lifetime> Widget for DataTypeSelectorView<'lifetime> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let icon_library = &theme.icon_library;
        let width = self.width;
        let height = self.height;
        let element_width = 104.0;
        let data_type_id = self.active_data_type.get_data_type_id();
        let icon = DataTypeToIconConverter::convert_data_type_to_icon(data_type_id, icon_library);

        let combo_box = ComboBoxView::new(
            self.app_context.clone(),
            DataTypeToStringConverter::convert_data_type_to_string(data_type_id),
            self.menu_id,
            Some(icon),
            |popup_user_interface: &mut Ui, should_close: &mut bool| {
                popup_user_interface.vertical(|user_interface| {
                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                DataTypeToStringConverter::convert_data_type_to_string(DataTypeU8::get_data_type_id()),
                                Some(DataTypeToIconConverter::convert_data_type_to_icon(DataTypeU8::get_data_type_id(), icon_library)),
                                element_width,
                            ))
                            .clicked()
                        {
                            *self.active_data_type = DataTypeRef::new(DataTypeU8::get_data_type_id());
                            *should_close = true;
                        };

                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                DataTypeToStringConverter::convert_data_type_to_string(DataTypeI8::get_data_type_id()),
                                Some(DataTypeToIconConverter::convert_data_type_to_icon(DataTypeI8::get_data_type_id(), icon_library)),
                                element_width,
                            ))
                            .clicked()
                        {
                            *self.active_data_type = DataTypeRef::new(DataTypeI8::get_data_type_id());
                            *should_close = true;
                        };
                    });

                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                DataTypeToStringConverter::convert_data_type_to_string(DataTypeI16::get_data_type_id()),
                                Some(DataTypeToIconConverter::convert_data_type_to_icon(
                                    DataTypeI16::get_data_type_id(),
                                    icon_library,
                                )),
                                element_width,
                            ))
                            .clicked()
                        {
                            *self.active_data_type = DataTypeRef::new(DataTypeI16::get_data_type_id());
                            *should_close = true;
                        };

                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                DataTypeToStringConverter::convert_data_type_to_string(DataTypeI16be::get_data_type_id()),
                                Some(DataTypeToIconConverter::convert_data_type_to_icon(
                                    DataTypeI16be::get_data_type_id(),
                                    icon_library,
                                )),
                                element_width,
                            ))
                            .clicked()
                        {
                            *self.active_data_type = DataTypeRef::new(DataTypeI16be::get_data_type_id());
                            *should_close = true;
                        };
                    });

                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                DataTypeToStringConverter::convert_data_type_to_string(DataTypeI32::get_data_type_id()),
                                Some(DataTypeToIconConverter::convert_data_type_to_icon(
                                    DataTypeI32::get_data_type_id(),
                                    icon_library,
                                )),
                                element_width,
                            ))
                            .clicked()
                        {
                            *self.active_data_type = DataTypeRef::new(DataTypeI32::get_data_type_id());
                            *should_close = true;
                        };

                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                DataTypeToStringConverter::convert_data_type_to_string(DataTypeI32be::get_data_type_id()),
                                Some(DataTypeToIconConverter::convert_data_type_to_icon(
                                    DataTypeI32be::get_data_type_id(),
                                    icon_library,
                                )),
                                element_width,
                            ))
                            .clicked()
                        {
                            *self.active_data_type = DataTypeRef::new(DataTypeI32be::get_data_type_id());
                            *should_close = true;
                        };
                    });

                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                DataTypeToStringConverter::convert_data_type_to_string(DataTypeI64::get_data_type_id()),
                                Some(DataTypeToIconConverter::convert_data_type_to_icon(
                                    DataTypeI64::get_data_type_id(),
                                    icon_library,
                                )),
                                element_width,
                            ))
                            .clicked()
                        {
                            *self.active_data_type = DataTypeRef::new(DataTypeI64::get_data_type_id());
                            *should_close = true;
                        };

                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                DataTypeToStringConverter::convert_data_type_to_string(DataTypeI64be::get_data_type_id()),
                                Some(DataTypeToIconConverter::convert_data_type_to_icon(
                                    DataTypeI64be::get_data_type_id(),
                                    icon_library,
                                )),
                                element_width,
                            ))
                            .clicked()
                        {
                            *self.active_data_type = DataTypeRef::new(DataTypeI64be::get_data_type_id());
                            *should_close = true;
                        };
                    });

                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                DataTypeToStringConverter::convert_data_type_to_string(DataTypeU16::get_data_type_id()),
                                Some(DataTypeToIconConverter::convert_data_type_to_icon(
                                    DataTypeU16::get_data_type_id(),
                                    icon_library,
                                )),
                                element_width,
                            ))
                            .clicked()
                        {
                            *self.active_data_type = DataTypeRef::new(DataTypeU16::get_data_type_id());
                            *should_close = true;
                        };

                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                DataTypeToStringConverter::convert_data_type_to_string(DataTypeU16be::get_data_type_id()),
                                Some(DataTypeToIconConverter::convert_data_type_to_icon(
                                    DataTypeU16be::get_data_type_id(),
                                    icon_library,
                                )),
                                element_width,
                            ))
                            .clicked()
                        {
                            *self.active_data_type = DataTypeRef::new(DataTypeU16be::get_data_type_id());
                            *should_close = true;
                        };
                    });

                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                DataTypeToStringConverter::convert_data_type_to_string(DataTypeU32::get_data_type_id()),
                                Some(DataTypeToIconConverter::convert_data_type_to_icon(
                                    DataTypeU32::get_data_type_id(),
                                    icon_library,
                                )),
                                element_width,
                            ))
                            .clicked()
                        {
                            *self.active_data_type = DataTypeRef::new(DataTypeU32::get_data_type_id());
                            *should_close = true;
                        };

                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                DataTypeToStringConverter::convert_data_type_to_string(DataTypeU32be::get_data_type_id()),
                                Some(DataTypeToIconConverter::convert_data_type_to_icon(
                                    DataTypeU32be::get_data_type_id(),
                                    icon_library,
                                )),
                                element_width,
                            ))
                            .clicked()
                        {
                            *self.active_data_type = DataTypeRef::new(DataTypeU32be::get_data_type_id());
                            *should_close = true;
                        };
                    });

                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                DataTypeToStringConverter::convert_data_type_to_string(DataTypeU64::get_data_type_id()),
                                Some(DataTypeToIconConverter::convert_data_type_to_icon(
                                    DataTypeU64::get_data_type_id(),
                                    icon_library,
                                )),
                                element_width,
                            ))
                            .clicked()
                        {
                            *self.active_data_type = DataTypeRef::new(DataTypeU64::get_data_type_id());
                            *should_close = true;
                        };

                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                DataTypeToStringConverter::convert_data_type_to_string(DataTypeU64be::get_data_type_id()),
                                Some(DataTypeToIconConverter::convert_data_type_to_icon(
                                    DataTypeU64be::get_data_type_id(),
                                    icon_library,
                                )),
                                element_width,
                            ))
                            .clicked()
                        {
                            *self.active_data_type = DataTypeRef::new(DataTypeU64be::get_data_type_id());
                            *should_close = true;
                        };
                    });

                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                DataTypeToStringConverter::convert_data_type_to_string(DataTypeF32::get_data_type_id()),
                                Some(DataTypeToIconConverter::convert_data_type_to_icon(
                                    DataTypeF32::get_data_type_id(),
                                    icon_library,
                                )),
                                element_width,
                            ))
                            .clicked()
                        {
                            *self.active_data_type = DataTypeRef::new(DataTypeF32::get_data_type_id());
                            *should_close = true;
                        };

                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                DataTypeToStringConverter::convert_data_type_to_string(DataTypeF32be::get_data_type_id()),
                                Some(DataTypeToIconConverter::convert_data_type_to_icon(
                                    DataTypeF32be::get_data_type_id(),
                                    icon_library,
                                )),
                                element_width,
                            ))
                            .clicked()
                        {
                            *self.active_data_type = DataTypeRef::new(DataTypeF32be::get_data_type_id());
                            *should_close = true;
                        };
                    });

                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                DataTypeToStringConverter::convert_data_type_to_string(DataTypeF64::get_data_type_id()),
                                Some(DataTypeToIconConverter::convert_data_type_to_icon(
                                    DataTypeF64::get_data_type_id(),
                                    icon_library,
                                )),
                                element_width,
                            ))
                            .clicked()
                        {
                            *self.active_data_type = DataTypeRef::new(DataTypeF64::get_data_type_id());
                            *should_close = true;
                        };

                        if user_interface
                            .add(DataTypeItemView::new(
                                self.app_context.clone(),
                                DataTypeToStringConverter::convert_data_type_to_string(DataTypeF64be::get_data_type_id()),
                                Some(DataTypeToIconConverter::convert_data_type_to_icon(
                                    DataTypeF64be::get_data_type_id(),
                                    icon_library,
                                )),
                                element_width,
                            ))
                            .clicked()
                        {
                            *self.active_data_type = DataTypeRef::new(DataTypeF64be::get_data_type_id());
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
        .width(width)
        .height(height);

        // Add the combo box to the layout
        user_interface.add(combo_box)
    }
}
