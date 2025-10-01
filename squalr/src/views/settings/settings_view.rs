use crate::{app_context::AppContext, models::tab_menu::tab_menu_data::TabMenuData, ui::widgets::controls::tab_menu::tab_menu_view::TabMenuView};
use eframe::egui::{Response, Sense, Ui, Widget};
use std::{
    rc::Rc,
    sync::atomic::{AtomicI32, Ordering},
};

#[derive(Clone)]
pub struct SettingsView {
    app_context: Rc<AppContext>,
    tab_menu_data: TabMenuData,
}

impl SettingsView {
    pub fn new(app_context: Rc<AppContext>) -> Self {
        let tab_menu_data = TabMenuData {
            headers: vec!["General".to_string(), "Memory".to_string(), "Scan".to_string()].into(),
            active_tab_index: Rc::new(AtomicI32::new(0)),
        };

        Self { app_context, tab_menu_data }
    }
}

impl Widget for SettingsView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (available_size_rectangle, response) = user_interface.allocate_exact_size(user_interface.available_size(), Sense::empty());
        let theme = &self.app_context.theme;

        // Compose the menu bar over the painted available space rectangle.
        let tab_menu = TabMenuView::new(self.app_context.clone(), &self.tab_menu_data);

        user_interface.add(tab_menu);

        match self.tab_menu_data.active_tab_index.load(Ordering::Acquire) {
            1 => {
                // Memory
            }
            2 => {
                // Scan
            }
            _ => {
                // General
            }
        }

        response
    }
}
