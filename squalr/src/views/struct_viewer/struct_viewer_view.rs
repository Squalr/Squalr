use crate::views::struct_viewer::struct_viewer_entry_view::StructViewerEntryView;
use crate::views::struct_viewer::view_data::struct_viewer_frame_action::StructViewerFrameAction;
use crate::{app_context::AppContext, views::struct_viewer::view_data::struct_viewer_view_data::StructViewerViewData};
use eframe::egui::{Align, CursorIcon, Layout, Response, ScrollArea, Sense, Ui, Widget};
use epaint::{Rect, pos2};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use squalr_engine_api::structures::data_values::{anonymous_value_string::AnonymousValueString, container_type::ContainerType};
use squalr_engine_api::structures::structs::valued_struct_field::ValuedStructFieldData;
use std::sync::Arc;

#[derive(Clone)]
pub struct StructViewerView {
    app_context: Arc<AppContext>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
}

impl StructViewerView {
    pub const WINDOW_ID: &'static str = "window_struct_viewer";

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let struct_viewer_view_data = app_context
            .dependency_container
            .register(StructViewerViewData::new());

        Self {
            app_context,
            struct_viewer_view_data,
        }
    }
}

impl Widget for StructViewerView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        const ICON_COLUMN_WIDTH: f32 = 32.0;
        const BAR_THICKNESS: f32 = 4.0;
        const MINIMUM_COLUMN_PIXEL_WIDTH: f32 = 80.0;

        let theme = &self.app_context.theme;
        let mut frame_action = StructViewerFrameAction::None;

        let mut new_value_splitter_ratio: Option<f32> = None;

        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |mut user_interface| {
                let mut struct_viewer_view_data = match self.struct_viewer_view_data.write("Struct viewer view") {
                    Some(data) => data,
                    None => return,
                };
                let mut value_splitter_ratio = struct_viewer_view_data.value_splitter_ratio;
                let content_rect = user_interface.available_rect_before_wrap();
                let content_width = content_rect.width();
                let content_min_x = content_rect.min.x;

                if content_width <= 0.0 {
                    return;
                }

                if value_splitter_ratio <= 0.0 {
                    value_splitter_ratio = StructViewerViewData::DEFAULT_NAME_SPLITTER_RATIO;

                    new_value_splitter_ratio = Some(value_splitter_ratio);
                }

                let value_splitter_x = content_min_x + content_width * value_splitter_ratio;

                let splitter_min_y = content_rect.min.y;
                let splitter_max_y = content_rect.max.y;

                let value_splitter_rect = Rect::from_min_max(
                    pos2(value_splitter_x - BAR_THICKNESS * 0.5, splitter_min_y),
                    pos2(value_splitter_x + BAR_THICKNESS * 0.5, splitter_max_y),
                );

                // Rows
                ScrollArea::vertical()
                    .id_salt("struct_viewer")
                    .auto_shrink([false, false])
                    .show(&mut user_interface, |inner_ui| {
                        if let Some(struct_under_view) = struct_viewer_view_data.struct_under_view.as_ref() {
                            let struct_fields = struct_under_view.get_fields().to_vec();
                            let selected_field_name = struct_viewer_view_data.selected_field_name.as_ref().clone();
                            let field_display_values_map = struct_viewer_view_data.field_display_values.clone();

                            for (field_row_index, field) in struct_fields.into_iter().enumerate() {
                                let is_selected = selected_field_name.as_deref().unwrap_or_default() == field.get_name();
                                let validation_data_type_ref = field
                                    .get_data_value()
                                    .map(|data_value| data_value.get_data_type_ref());
                                let field_edit_value = struct_viewer_view_data
                                    .field_edit_values
                                    .get_mut(field.get_name());
                                let field_display_values = field_display_values_map
                                    .get(field.get_name())
                                    .map(Vec::as_slice);

                                inner_ui.add(StructViewerEntryView::new(
                                    self.app_context.clone(),
                                    &field,
                                    field_row_index,
                                    is_selected,
                                    &mut frame_action,
                                    field_edit_value,
                                    field_display_values,
                                    validation_data_type_ref,
                                    ICON_COLUMN_WIDTH + BAR_THICKNESS,
                                    value_splitter_x + BAR_THICKNESS,
                                ));
                            }
                        }
                    });

                // Draw non-resizable icon/name divider.
                let icon_divider_x = content_min_x + ICON_COLUMN_WIDTH;
                let icon_divider_rect = Rect::from_min_max(
                    pos2(icon_divider_x - BAR_THICKNESS * 0.5, splitter_min_y),
                    pos2(icon_divider_x + BAR_THICKNESS * 0.5, splitter_max_y),
                );

                user_interface
                    .painter()
                    .rect_filled(icon_divider_rect, 0.0, theme.background_control);

                // Draw the name/value divider.
                let value_splitter_response = user_interface
                    .interact(value_splitter_rect, user_interface.id().with("value_splitter"), Sense::drag())
                    .on_hover_cursor(CursorIcon::ResizeHorizontal);

                user_interface
                    .painter()
                    .rect_filled(value_splitter_rect, 0.0, theme.background_control);

                if value_splitter_response.dragged() {
                    let drag_delta = value_splitter_response.drag_delta();
                    let mut new_x = value_splitter_x + drag_delta.x;
                    let min_x = content_min_x + ICON_COLUMN_WIDTH + MINIMUM_COLUMN_PIXEL_WIDTH;
                    let max_x = content_min_x + content_width - MINIMUM_COLUMN_PIXEL_WIDTH;

                    new_x = new_x.clamp(min_x, max_x);
                    new_value_splitter_ratio = Some((new_x - content_min_x) / content_width);
                }
            })
            .response;

        // Commit splitter changes.
        if new_value_splitter_ratio.is_some() {
            if let Some(mut data) = self.struct_viewer_view_data.write("Struct viewer view") {
                if let Some(ratio) = new_value_splitter_ratio {
                    data.value_splitter_ratio = ratio;
                }
            }
        }

        match frame_action {
            StructViewerFrameAction::None => {}
            StructViewerFrameAction::SelectField(field_name) => {
                StructViewerViewData::set_selected_field(self.struct_viewer_view_data.clone(), field_name);
            }
            StructViewerFrameAction::EditValue(edited_field) => {
                if let Some(mut struct_viewer_view_data) = self.struct_viewer_view_data.write("Struct viewer edit value") {
                    if let Some(struct_under_view) = Arc::make_mut(&mut struct_viewer_view_data.struct_under_view).as_mut() {
                        if let Some(field_under_view) = struct_under_view.get_field_mut(edited_field.get_name()) {
                            field_under_view.set_field_data(edited_field.get_field_data().clone());
                        }
                    }

                    if let ValuedStructFieldData::Value(new_data_value) = edited_field.get_field_data() {
                        if let Some(edit_value) = struct_viewer_view_data
                            .field_edit_values
                            .get_mut(edited_field.get_name())
                        {
                            let symbol_registry = SymbolRegistry::get_instance();
                            let data_type_ref = new_data_value.get_data_type_ref();
                            let default_anonymous_value_string_format = symbol_registry.get_default_anonymous_value_string_format(data_type_ref);
                            let new_anonymous_value_string = symbol_registry
                                .anonymize_value(new_data_value, default_anonymous_value_string_format)
                                .unwrap_or_else(|error| {
                                    log::warn!("Failed to anonymize edited struct value: {}", error);
                                    AnonymousValueString::new(String::new(), default_anonymous_value_string_format, ContainerType::None)
                                });

                            *edit_value = new_anonymous_value_string;
                        }
                    }

                    if let Some(struct_field_modified_callback) = struct_viewer_view_data.struct_field_modified_callback.clone() {
                        struct_field_modified_callback(edited_field);
                    }
                }
            }
        }

        response
    }
}
