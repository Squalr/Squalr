use crate::{app_context::AppContext, ui::converters::data_type_to_icon_converter::DataTypeToIconConverter};
use eframe::egui::{Align, Id, Layout, Response, Ui, Widget};
use epaint::vec2;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use std::sync::Arc;

pub struct ElementScannerResultsOverlayView<'lifetime> {
    app_context: Arc<AppContext>,
    active_data_type: &'lifetime mut DataTypeRef,
    width: f32,
    height: f32,
}

impl<'lifetime> ElementScannerResultsOverlayView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        active_data_type: &'lifetime mut DataTypeRef,
    ) -> Self {
        Self {
            app_context,
            active_data_type,
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
}

impl<'lifetime> Widget for ElementScannerResultsOverlayView<'lifetime> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let icon_library = &theme.icon_library;
        let width = self.width;
        let height = self.height;

        let response = user_interface
            .allocate_ui_with_layout(vec2(width, height), Layout::top_down(Align::Min), |mut user_interface| {
                //
            })
            .response;

        response
    }
}
