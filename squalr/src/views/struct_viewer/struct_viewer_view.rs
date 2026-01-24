use crate::ui::converters::data_type_to_icon_converter::DataTypeToIconConverter;
use crate::views::struct_viewer::struct_viewer_entry_view::StructViewerEntryView;
use crate::{app_context::AppContext, views::struct_viewer::view_data::struct_viewer_view_data::StructViewerViewData};
use eframe::egui::{Align, Layout, Response, ScrollArea, Ui, Widget};
use squalr_engine_api::dependency_injection::dependency::Dependency;
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
        let theme = self.app_context.theme.clone();
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |mut user_interface| {
                let struct_viewer_view_data = match self.struct_viewer_view_data.read("Process selector view") {
                    Some(struct_viewer_view_data) => struct_viewer_view_data,
                    None => return,
                };

                ScrollArea::vertical()
                    .id_salt("process_selector")
                    .auto_shrink([false, false])
                    .show(&mut user_interface, |inner_user_interface| {
                        let mut selected_field = None;

                        if let Some(struct_under_view) = struct_viewer_view_data.struct_under_view.as_ref() {
                            for field in struct_under_view.get_fields() {
                                let icon = DataTypeToIconConverter::convert_data_type_to_icon(field.get_icon_id(), &theme.icon_library);

                                if inner_user_interface
                                    .add(StructViewerEntryView::new(self.app_context.clone(), field.get_name(), Some(icon)))
                                    .double_clicked()
                                {
                                    selected_field = Some(Some(field.get_name()));
                                }
                            }

                            if let Some(selected_field) = selected_field {
                                // drop(process_selector_view_data);

                                // ProcessSelectorViewData::select_process(self.process_selector_view_data.clone(), self.app_context.clone(), selected_field);
                            }
                        }
                    });
            })
            .response;

        response
    }
}
