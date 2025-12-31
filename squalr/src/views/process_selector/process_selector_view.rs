use crate::{
    app_context::AppContext,
    views::process_selector::{
        process_entry_view::ProcessEntryView, process_selector_toolbar_view::ProcessSelectorToolbarView, process_selector_view_data::ProcessSelectorViewData,
    },
};
use eframe::egui::{Align, Direction, Layout, Response, ScrollArea, Spinner, Ui, Widget};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use std::sync::Arc;

#[derive(Clone)]
pub struct ProcessSelectorView {
    app_context: Arc<AppContext>,
    process_selector_view_data: Dependency<ProcessSelectorViewData>,
    process_selector_toolbar_view: ProcessSelectorToolbarView,
}

impl ProcessSelectorView {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let process_selector_view_data = app_context
            .dependency_container
            .register(ProcessSelectorViewData::new());
        let process_selector_toolbar_view = ProcessSelectorToolbarView::new(app_context.clone());

        Self {
            app_context,
            process_selector_view_data,
            process_selector_toolbar_view,
        }
    }
}

impl Widget for ProcessSelectorView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = self.app_context.theme.clone();
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |mut user_interface| {
                user_interface.add(self.process_selector_toolbar_view.clone());

                let process_selector_view_data = match self.process_selector_view_data.read() {
                    Ok(process_selector_view_data) => process_selector_view_data,
                    Err(_error) => {
                        return;
                    }
                };

                if !process_selector_view_data.is_awaiting_full_process_list {
                    ScrollArea::vertical()
                        .id_salt("process_selector")
                        .auto_shrink([false, false])
                        .show(&mut user_interface, |inner_user_interface| {
                            let mut selected_process = None;

                            for process in &process_selector_view_data.full_process_list {
                                let icon = match process.get_icon() {
                                    Some(icon) => process_selector_view_data.get_icon(&self.app_context, process.get_process_id_raw(), icon),
                                    None => None,
                                };

                                if inner_user_interface
                                    .add(ProcessEntryView::new(self.app_context.clone(), process.get_name(), icon))
                                    .double_clicked()
                                {
                                    selected_process = Some(Some(process.get_process_id_raw()));
                                }
                            }

                            if let Some(selected_process) = selected_process {
                                drop(process_selector_view_data);

                                ProcessSelectorViewData::select_process(self.process_selector_view_data.clone(), self.app_context.clone(), selected_process);
                            }
                        });
                } else {
                    user_interface.allocate_ui_with_layout(
                        user_interface.available_size(),
                        Layout::centered_and_justified(Direction::LeftToRight),
                        |user_interface| {
                            user_interface.add(Spinner::new().color(theme.foreground));
                        },
                    );
                }
            })
            .response;

        response
    }
}
