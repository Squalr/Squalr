use crate::{
    app_context::AppContext,
    views::process_selector::{
        process_entry_view::ProcessEntryView, process_selector_toolbar_view::ProcessSelectorToolbarView, process_selector_view_data::ProcessSelectorViewData,
    },
};
use eframe::egui::{Align, Layout, Response, ScrollArea, Ui, Widget};
use squalr_engine_api::{
    commands::{engine_command_request::EngineCommandRequest, process::open::process_open_request::ProcessOpenRequest},
    dependency_injection::dependency::Dependency,
    structures::processes::opened_process_info::OpenedProcessInfo,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct ProcessSelectorView {
    app_context: Arc<AppContext>,
    process_selector_view_data: Dependency<ProcessSelectorViewData>,
}

impl ProcessSelectorView {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let process_selector_view_data = app_context
            .dependency_container
            .register(ProcessSelectorViewData::new());

        Self {
            app_context,
            process_selector_view_data,
        }
    }

    fn select_process(
        app_context: Arc<AppContext>,
        process_selector_view_data: Dependency<ProcessSelectorViewData>,
        process_id: u32,
    ) {
        let engine_execution_context = app_context.engine_execution_context.clone();
        let process_open_request = ProcessOpenRequest {
            process_id: Some(process_id),
            search_name: None,
            match_case: false,
        };

        process_open_request.send(&engine_execution_context, move |process_open_response| {
            Self::update_cached_opened_process(app_context, process_selector_view_data, process_open_response.opened_process_info)
        });
    }

    fn update_cached_opened_process(
        app_context: Arc<AppContext>,
        process_selector_view_data: Dependency<ProcessSelectorViewData>,
        process_info: Option<OpenedProcessInfo>,
    ) {
        let mut process_selector_view_data = match process_selector_view_data.write() {
            Ok(process_selector_view_data) => process_selector_view_data,
            Err(_error) => return,
        };

        process_selector_view_data.set_opened_process(&app_context.context, process_info);
    }
}

impl Widget for ProcessSelectorView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |mut user_interface| {
                user_interface.add(ProcessSelectorToolbarView::new(self.app_context.clone()));

                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(&mut user_interface, |inner_user_interface| {
                        let process_selector_view_data = match self.process_selector_view_data.read() {
                            Ok(process_selector_view_data) => process_selector_view_data,
                            Err(_error) => {
                                return;
                            }
                        };

                        let mut selected_process = None;

                        for windowed_process in &process_selector_view_data.full_process_list {
                            let icon = match windowed_process.get_icon() {
                                Some(icon) => {
                                    process_selector_view_data.get_or_create_icon(&self.app_context.context, windowed_process.get_process_id_raw(), icon)
                                }
                                None => None,
                            };

                            if inner_user_interface
                                .add(ProcessEntryView::new(self.app_context.clone(), windowed_process.get_name(), icon))
                                .double_clicked()
                            {
                                selected_process = Some(windowed_process.get_process_id_raw());
                            }
                        }

                        if let Some(selected_process) = selected_process {
                            drop(process_selector_view_data);

                            Self::select_process(self.app_context.clone(), self.process_selector_view_data.clone(), selected_process);
                        }
                    });
            })
            .response;

        response
    }
}
