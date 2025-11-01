use crate::{
    app_context::AppContext,
    ui::widgets::controls::combo_box::{combo_box_item_view::ComboBoxItemView, combo_box_view::ComboBoxView},
    views::process_selector::process_selector_view_data::ProcessSelectorViewData,
};
use eframe::egui::{Align, Layout, Response, Sense, Ui, UiBuilder, Widget};
use epaint::{CornerRadius, Rect, vec2};
use squalr_engine_api::{
    commands::{
        engine_command_request::EngineCommandRequest,
        process::{list::process_list_request::ProcessListRequest, open::process_open_request::ProcessOpenRequest},
    },
    dependency_injection::dependency::Dependency,
    engine::engine_execution_context::EngineExecutionContext,
    events::process::changed::process_changed_event::ProcessChangedEvent,
    structures::processes::opened_process_info::OpenedProcessInfo,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct MainShortcutBarView {
    app_context: Arc<AppContext>,
    process_selector_view_data: Dependency<ProcessSelectorViewData>,
}

impl MainShortcutBarView {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let process_selector_view_data = app_context
            .dependency_container
            .get_lazy::<ProcessSelectorViewData>();
        let instance = Self {
            app_context,
            process_selector_view_data,
        };

        instance.listen_for_process_change();

        instance
    }

    fn listen_for_process_change(&self) {
        let app_context = self.app_context.clone();
        let process_selector_view_data = self.process_selector_view_data.clone();
        let engine_execution_context = self.app_context.engine_execution_context.clone();

        engine_execution_context.listen_for_engine_event::<ProcessChangedEvent>(move |process_changed_event| {
            Self::update_cached_opened_process(
                app_context.clone(),
                process_selector_view_data.clone(),
                process_changed_event.process_info.clone(),
            );
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

    fn refresh_full_process_list(&self) {
        let list_all_processes_request = ProcessListRequest {
            require_windowed: false,
            search_name: None,
            match_case: false,
            limit: None,
            fetch_icons: true,
        };
        let engine_execution_context = self.app_context.engine_execution_context.clone();

        list_all_processes_request.send(&engine_execution_context, move |process_list_response| {
            // full_process_list_collection = process_list_response.processes
        });
    }

    fn refresh_windowed_process_list(
        engine_execution_context: Arc<EngineExecutionContext>,
        process_selector_view_data: Dependency<ProcessSelectorViewData>,
    ) {
        let list_windowed_processes_request = ProcessListRequest {
            require_windowed: true,
            search_name: None,
            match_case: false,
            limit: None,
            fetch_icons: true,
        };
        let engine_execution_context = engine_execution_context.clone();

        list_windowed_processes_request.send(&engine_execution_context, move |process_list_response| {
            // windowed_process_list_collection = process_list_response.processes
            let mut process_selector_view_data = match process_selector_view_data.write() {
                Ok(process_selector_view_data) => process_selector_view_data,
                Err(error) => {
                    log::error!("Failed to access process selector view data for updating windowed process list: {}", error);
                    return;
                }
            };

            process_selector_view_data.windowed_processes = process_list_response.processes;
        });
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
}

impl Widget for MainShortcutBarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (available_size_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), 32.0), Sense::empty());
        let theme = &self.app_context.theme;
        let combo_box_width = 192.0;
        let process_dropdown_list_width = 256.0;
        let process_selector_view_data = match self.process_selector_view_data.read() {
            Ok(process_selector_view_data) => process_selector_view_data,
            Err(_error) => {
                return response;
            }
        };

        // Draw background.
        user_interface
            .painter()
            .rect_filled(available_size_rectangle, CornerRadius::ZERO, theme.background_primary);

        let available_size_rectangle = Rect::from_min_size(available_size_rectangle.min, vec2(available_size_rectangle.width(), 32.0));
        let mut row_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(available_size_rectangle)
                .layout(Layout::left_to_right(Align::Center)),
        );

        row_user_interface.add_space(8.0);

        let mut refresh_windowed_processes = false;
        let mut process_to_open = None;

        let name_display = match &process_selector_view_data.opened_process {
            Some(opened_proces) => opened_proces.get_name(),
            None => "Select a process...",
        };

        let process_select_combo_box = ComboBoxView::new_from_theme(
            theme,
            self.app_context.clone(),
            name_display,
            process_selector_view_data.cached_icon.clone(),
            |user_interface: &mut Ui, should_close: &mut bool| {
                for windowed_process in &process_selector_view_data.windowed_processes {
                    if user_interface
                        .add(ComboBoxItemView::new(
                            self.app_context.clone(),
                            windowed_process.get_name(),
                            None,
                            process_dropdown_list_width,
                        ))
                        .clicked()
                    {
                        process_to_open = Some(windowed_process.get_process_id().as_u32());
                        *should_close = true;

                        return;
                    }
                }
            },
        )
        .width(combo_box_width);

        if row_user_interface.add(process_select_combo_box).clicked() {
            refresh_windowed_processes = true;
        }

        if refresh_windowed_processes {
            // Drop the read lock to free up the data for write lock access.
            drop(process_selector_view_data);

            // Refresh the process list on first click.
            // JIRA: Set an atomic flag maybe on the process view data such that we can show a loading indicator.
            Self::refresh_windowed_process_list(self.app_context.engine_execution_context.clone(), self.process_selector_view_data.clone());
        } else if let Some(process_id) = process_to_open {
            // Drop the read lock to free up the data for write lock access.
            drop(process_selector_view_data);

            Self::select_process(self.app_context.clone(), self.process_selector_view_data.clone(), process_id);
        }

        response
    }
}
