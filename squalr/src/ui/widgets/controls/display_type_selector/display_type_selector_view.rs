use crate::{
    app_context::AppContext,
    ui::{
        converters::anonymous_value_string_format_to_icon_converter::AnonymousValueStringFormatToIconConverter,
        widgets::controls::combo_box::{combo_box_item_view::ComboBoxItemView, combo_box_view::ComboBoxView},
    },
};
use eframe::egui::{Response, Ui, Widget};
use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use std::sync::Arc;

pub struct DisplayTypeSelectorView<'lifetime> {
    app_context: Arc<AppContext>,
    active_display_format: &'lifetime mut AnonymousValueStringFormat,
    available_display_formats: Vec<AnonymousValueStringFormat>,
    menu_id: &'lifetime str,
    disabled: bool,
    width: f32,
    height: f32,
    show_preview_text: bool,
}

impl<'lifetime> DisplayTypeSelectorView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        active_display_format: &'lifetime mut AnonymousValueStringFormat,
        available_display_formats: Vec<AnonymousValueStringFormat>,
        menu_id: &'lifetime str,
    ) -> Self {
        Self {
            app_context,
            active_display_format,
            available_display_formats: Self::normalize_available_display_formats(&available_display_formats),
            menu_id,
            disabled: false,
            width: 110.0,
            height: 28.0,
            show_preview_text: true,
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

    pub fn disabled(
        mut self,
        disabled: bool,
    ) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn hide_preview_text(mut self) -> Self {
        self.show_preview_text = false;
        self
    }

    fn display_format_label(anonymous_value_string_format: AnonymousValueStringFormat) -> &'static str {
        match anonymous_value_string_format {
            AnonymousValueStringFormat::Bool => "Boolean",
            AnonymousValueStringFormat::String => "String",
            AnonymousValueStringFormat::Binary => "Binary",
            AnonymousValueStringFormat::Decimal => "Decimal",
            AnonymousValueStringFormat::Hexadecimal => "Hex",
            AnonymousValueStringFormat::HexPattern => "Pattern",
            AnonymousValueStringFormat::Address => "Address",
            AnonymousValueStringFormat::DataTypeRef => "Data Type",
            AnonymousValueStringFormat::Enumeration => "Enum",
        }
    }

    fn display_format_sort_key(anonymous_value_string_format: AnonymousValueStringFormat) -> u8 {
        match anonymous_value_string_format {
            AnonymousValueStringFormat::Bool => 0,
            AnonymousValueStringFormat::String => 1,
            AnonymousValueStringFormat::Binary => 2,
            AnonymousValueStringFormat::Decimal => 3,
            AnonymousValueStringFormat::Hexadecimal => 4,
            AnonymousValueStringFormat::HexPattern => 5,
            AnonymousValueStringFormat::Address => 6,
            AnonymousValueStringFormat::DataTypeRef => 7,
            AnonymousValueStringFormat::Enumeration => 8,
        }
    }

    pub fn normalize_available_display_formats(available_display_formats: &[AnonymousValueStringFormat]) -> Vec<AnonymousValueStringFormat> {
        let mut normalized_available_display_formats = available_display_formats.to_vec();
        normalized_available_display_formats.sort_by_key(|anonymous_value_string_format| Self::display_format_sort_key(*anonymous_value_string_format));
        normalized_available_display_formats.dedup();

        normalized_available_display_formats
    }
}

impl<'lifetime> Widget for DisplayTypeSelectorView<'lifetime> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let combo_icon = Some(AnonymousValueStringFormatToIconConverter::convert(
            *self.active_display_format,
            &self.app_context.theme.icon_library,
        ));
        let combo_label = if self.show_preview_text {
            Self::display_format_label(*self.active_display_format).to_string()
        } else {
            String::new()
        };
        let available_display_formats = self.available_display_formats;
        let available_display_format_count = available_display_formats.len();
        let popup_display_formats = available_display_formats.clone();
        let app_context = self.app_context.clone();
        let active_display_format = self.active_display_format;

        user_interface.add(
            ComboBoxView::new(
                self.app_context,
                combo_label,
                self.menu_id,
                combo_icon,
                move |popup_user_interface: &mut Ui, should_close: &mut bool| {
                    for anonymous_value_string_format in &popup_display_formats {
                        let item_response = popup_user_interface.add(
                            ComboBoxItemView::new(
                                app_context.clone(),
                                Self::display_format_label(*anonymous_value_string_format),
                                Some(AnonymousValueStringFormatToIconConverter::convert(
                                    *anonymous_value_string_format,
                                    &app_context.theme.icon_library,
                                )),
                                110.0,
                            )
                            .width(110.0),
                        );

                        if item_response.clicked() {
                            *active_display_format = *anonymous_value_string_format;
                            *should_close = true;
                        }
                    }
                },
            )
            .width(self.width)
            .height(self.height)
            .disabled(self.disabled || available_display_format_count <= 1),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::DisplayTypeSelectorView;
    use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;

    #[test]
    fn normalize_available_display_formats_sorts_and_deduplicates() {
        assert_eq!(
            DisplayTypeSelectorView::normalize_available_display_formats(&[
                AnonymousValueStringFormat::Hexadecimal,
                AnonymousValueStringFormat::String,
                AnonymousValueStringFormat::Decimal,
                AnonymousValueStringFormat::Hexadecimal,
            ]),
            vec![
                AnonymousValueStringFormat::String,
                AnonymousValueStringFormat::Decimal,
                AnonymousValueStringFormat::Hexadecimal,
            ]
        );
    }
}
