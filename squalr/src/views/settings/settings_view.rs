use crate::{
    app_context::AppContext,
    models::tab_menu::tab_menu_data::TabMenuData,
    ui::widgets::controls::tab_menu::tab_menu_view::TabMenuView,
    views::settings::{settings_tab_memory_view::SettingsTabMemoryView, settings_tab_scan_view::SettingsTabScanView},
};
use eframe::egui::{Align, Layout, Response, Ui, Widget};
use std::{
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicI32, Ordering},
    },
};

#[derive(Clone)]
pub struct SettingsView {
    app_context: Arc<AppContext>,
    tab_menu_data: TabMenuData,
    settings_tab_memory_view: Rc<SettingsTabMemoryView>,
    settings_tab_scan_view: Rc<SettingsTabScanView>,
}

impl SettingsView {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let tab_menu_data = TabMenuData {
            headers: vec!["General".to_string(), "Memory".to_string(), "Scan".to_string()].into(),
            active_tab_index: Rc::new(AtomicI32::new(1)),
        };
        let settings_tab_memory_view = Rc::new(SettingsTabMemoryView::new(app_context.clone()));
        let settings_tab_scan_view = Rc::new(SettingsTabScanView::new(app_context.clone()));

        Self {
            app_context,
            tab_menu_data,
            settings_tab_memory_view,
            settings_tab_scan_view,
        }
    }
}

impl Widget for SettingsView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                // Compose the menu bar over the painted available space rectangle.
                let tab_menu = TabMenuView::new(self.app_context.clone(), &self.tab_menu_data);

                user_interface.add(tab_menu);

                match self.tab_menu_data.active_tab_index.load(Ordering::Acquire) {
                    1 => {
                        user_interface.add(self.settings_tab_memory_view.as_ref().clone());
                    }
                    2 => {
                        user_interface.add(self.settings_tab_scan_view.as_ref().clone());
                    }
                    _ => {
                        // General settings.
                    }
                }
            })
            .response;

        response
    }
}
