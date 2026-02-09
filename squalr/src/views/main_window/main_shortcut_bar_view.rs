use crate::{
    app_context::AppContext,
    ui::widgets::controls::combo_box::{combo_box_item_view::ComboBoxItemView, combo_box_view::ComboBoxView},
    views::process_selector::view_data::process_selector_view_data::ProcessSelectorViewData,
};
use eframe::egui::{Align, Direction, Layout, Response, Sense, Spinner, Ui, UiBuilder, Widget};
use epaint::{CornerRadius, Rect, vec2};
use squalr_engine_api::{dependency_injection::dependency::Dependency, events::process::changed::process_changed_event::ProcessChangedEvent};
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
            .get_dependency::<ProcessSelectorViewData>();
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
        let engine_unprivileged_state = self.app_context.engine_unprivileged_state.clone();

        engine_unprivileged_state.listen_for_engine_event::<ProcessChangedEvent>(move |process_changed_event| {
            ProcessSelectorViewData::set_opened_process_info(process_selector_view_data.clone(), &app_context, process_changed_event.process_info.clone());
        });
    }
}

impl Widget for MainShortcutBarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), 32.0), Sense::empty());
        let theme = &self.app_context.theme;
        let combo_box_width = 224.0;
        let process_dropdown_list_width = 256.0;
        let process_selector_view_data = match self.process_selector_view_data.read("Main shortcut bar view") {
            Some(process_selector_view_data) => process_selector_view_data,
            None => return response,
        };

        // Draw background.
        user_interface
            .painter()
            .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.background_primary);

        let allocated_size_rectangle = Rect::from_min_size(allocated_size_rectangle.min, vec2(allocated_size_rectangle.width(), 32.0));
        let mut row_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(allocated_size_rectangle)
                .layout(Layout::left_to_right(Align::Center)),
        );

        row_user_interface.add_space(4.0);

        let mut refresh_windowed_processes = false;
        let mut process_to_open = None;

        let name_display = match &process_selector_view_data.opened_process {
            Some(opened_proces) => opened_proces.get_name(),
            None => {
                if process_selector_view_data.is_opening_process {
                    "Opening process..."
                } else {
                    "Select a process..."
                }
            }
        };

        let process_select_combo_box = ComboBoxView::new(
            self.app_context.clone(),
            name_display,
            "main_shortcut_bar_process_select",
            process_selector_view_data.cached_icon.clone(),
            |user_interface: &mut Ui, should_close: &mut bool| {
                if process_selector_view_data.opened_process.is_some() {
                    if user_interface
                        .add(ComboBoxItemView::new(
                            self.app_context.clone(),
                            "--- Detach from current process ---",
                            None,
                            process_dropdown_list_width,
                        ))
                        .clicked()
                    {
                        process_to_open = Some(None);
                        *should_close = true;

                        return;
                    }
                }

                if !process_selector_view_data.is_awaiting_windowed_process_list {
                    for windowed_process in &process_selector_view_data.windowed_process_list {
                        let icon = match windowed_process.get_icon() {
                            Some(icon) => process_selector_view_data.get_icon(&self.app_context, windowed_process.get_process_id_raw(), icon),
                            None => None,
                        };

                        if user_interface
                            .add(ComboBoxItemView::new(
                                self.app_context.clone(),
                                windowed_process.get_name(),
                                icon,
                                process_dropdown_list_width,
                            ))
                            .clicked()
                        {
                            process_to_open = Some(Some(windowed_process.get_process_id()));
                            *should_close = true;

                            return;
                        }
                    }
                } else {
                    user_interface.allocate_ui_with_layout(
                        vec2(user_interface.available_width(), 32.0),
                        Layout::centered_and_justified(Direction::LeftToRight),
                        |user_interface| {
                            user_interface.add(Spinner::new().color(theme.foreground));
                        },
                    );
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
            // JIRA: Set an atomic flag maybe on the process view data such that we can show a loading indicator?
            // Could throw in an artificial delay to simulate how this would behave over a network (GUI -> network -> shell).
            ProcessSelectorViewData::refresh_windowed_process_list(self.process_selector_view_data.clone(), self.app_context.clone());
        } else if let Some(process_id) = process_to_open {
            // Drop the read lock to free up the data for write lock access.
            drop(process_selector_view_data);

            ProcessSelectorViewData::select_process(self.process_selector_view_data.clone(), self.app_context.clone(), process_id);
        }

        response
    }
}
