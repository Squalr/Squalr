use crate::{
    app_context::AppContext,
    ui::{
        draw::icon_draw::IconDraw,
        widgets::controls::{button::Button, checkbox::Checkbox},
    },
    views::process_selector::view_data::process_selector_view_data::ProcessSelectorViewData,
};
use eframe::egui::{Align, Layout, Response, Sense, Ui, UiBuilder, Widget};
use epaint::{Color32, CornerRadius, vec2};
use squalr_engine_api::dependency_injection::dependency::Dependency;
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
            .get_dependency::<ProcessSelectorViewData>();
        let instance = Self {
            app_context,
            process_selector_view_data,
        };

        instance
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
            let show_windowed_processes_only = self
                .process_selector_view_data
                .read("Process selector toolbar view")
                .map(|process_selector_view_data| process_selector_view_data.show_windowed_processes_only)
                .unwrap_or(false);

            // Refresh.
            let button_refresh = user_interface.add_sized(button_size, Button::new_from_theme(&theme).background_color(Color32::TRANSPARENT));
            IconDraw::draw(user_interface, button_refresh.rect, &theme.icon_library.icon_handle_navigation_refresh);

            if button_refresh.clicked() {
                ProcessSelectorViewData::refresh_active_process_list(self.process_selector_view_data.clone(), self.app_context.clone());
            }

            let button_windowed_only = user_interface.add(Checkbox::new_from_theme(theme).with_check_state_bool(show_windowed_processes_only));
            user_interface.label("Windowed");

            if button_windowed_only.clicked() {
                ProcessSelectorViewData::set_show_windowed_processes_only(
                    self.process_selector_view_data.clone(),
                    self.app_context.clone(),
                    !show_windowed_processes_only,
                );
            }
        });

        response
    }
}
