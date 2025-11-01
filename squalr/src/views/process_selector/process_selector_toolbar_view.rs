use crate::{
    app_context::AppContext,
    ui::{draw::icon_draw::IconDraw, widgets::controls::button::Button},
    views::process_selector::process_selector_view_data::ProcessSelectorViewData,
};
use eframe::egui::{Align, Layout, Response, Sense, Ui, UiBuilder, Widget};
use epaint::{Color32, CornerRadius, vec2};
use squalr_engine_api::{
    commands::{engine_command_request::EngineCommandRequest, process::list::process_list_request::ProcessListRequest},
    dependency_injection::dependency::Dependency,
    engine::engine_execution_context::EngineExecutionContext,
    events::process::changed::process_changed_event::ProcessChangedEvent,
    structures::processes::opened_process_info::OpenedProcessInfo,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct ProcessSelectorToolbarView {
    app_context: Arc<AppContext>,
    process_selector_view_data: Dependency<ProcessSelectorViewData>,
}

impl ProcessSelectorToolbarView {
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

    fn refresh_full_process_list(
        engine_execution_context: Arc<EngineExecutionContext>,
        process_selector_view_data: Dependency<ProcessSelectorViewData>,
    ) {
        let list_windowed_processes_request = ProcessListRequest {
            require_windowed: false,
            search_name: None,
            match_case: false,
            limit: None,
            fetch_icons: true,
        };
        let engine_execution_context = engine_execution_context.clone();

        list_windowed_processes_request.send(&engine_execution_context, move |process_list_response| {
            let mut process_selector_view_data = match process_selector_view_data.write() {
                Ok(process_selector_view_data) => process_selector_view_data,
                Err(error) => {
                    log::error!("Failed to access process selector view data for updating windowed process list: {}", error);
                    return;
                }
            };

            process_selector_view_data.set_full_process_list(process_list_response.processes);
        });
    }
}

impl Widget for ProcessSelectorToolbarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let height = 28.0;
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), height), Sense::empty());
        let theme = &self.app_context.theme;

        user_interface
            .painter()
            .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.background_primary);

        // Create a child ui constrained to the title bar.
        let builder = UiBuilder::new()
            .max_rect(allocated_size_rectangle)
            .layout(Layout::left_to_right(Align::Center));
        let mut toolbar_user_interface = user_interface.new_child(builder);

        toolbar_user_interface.with_layout(Layout::left_to_right(Align::Center), |user_interface| {
            let button_size = vec2(36.0, 28.0);

            // Refresh.
            let refresh = user_interface.add_sized(button_size, Button::new_from_theme(&theme).background_color(Color32::TRANSPARENT));
            IconDraw::draw(user_interface, refresh.rect, &theme.icon_library.icon_navigation_refresh);

            if refresh.clicked() {
                Self::refresh_full_process_list(self.app_context.engine_execution_context.clone(), self.process_selector_view_data.clone());
            }
        });

        response
    }
}
