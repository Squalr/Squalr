use crate::ui::widgets::docking::dockable_window::DockableWindow;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Clone)]
pub struct DockRootViewData {
    // JIRA: Maybe make this a hashmap of id to window for faster lookups (ie sibling tab ids -> window name).
    pub windows: Arc<RwLock<Vec<Box<dyn DockableWindow>>>>,
    pub maximized_window_identifier: Arc<RwLock<Option<String>>>,
    pub tab_strip_scroll_offsets: Arc<RwLock<HashMap<String, f32>>>,
}

impl DockRootViewData {
    pub fn new() -> Self {
        Self {
            windows: Arc::new(RwLock::new(Vec::new())),
            maximized_window_identifier: Arc::new(RwLock::new(None)),
            tab_strip_scroll_offsets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn set_windows(
        &self,
        new_windows: Vec<Box<dyn DockableWindow>>,
    ) {
        match self.windows.write() {
            Ok(mut windows) => {
                *windows = new_windows;
            }
            Err(error) => {
                log::error!("Failed to acquire windows lock: {}", error);
            }
        }
    }

    pub fn get_maximized_window_identifier(&self) -> Option<String> {
        match self.maximized_window_identifier.read() {
            Ok(maximized_window_identifier) => maximized_window_identifier.clone(),
            Err(error) => {
                log::error!("Failed to acquire maximized window lock: {}", error);

                None
            }
        }
    }

    pub fn set_maximized_window_identifier(
        &self,
        maximized_window_identifier: Option<String>,
    ) {
        match self.maximized_window_identifier.write() {
            Ok(mut active_maximized_window_identifier) => {
                *active_maximized_window_identifier = maximized_window_identifier;
            }
            Err(error) => {
                log::error!("Failed to acquire maximized window lock: {}", error);
            }
        }
    }

    pub fn toggle_maximized_window_identifier(
        &self,
        window_identifier: &str,
    ) {
        match self.maximized_window_identifier.write() {
            Ok(mut active_maximized_window_identifier) => {
                if active_maximized_window_identifier.as_deref() == Some(window_identifier) {
                    *active_maximized_window_identifier = None;
                } else {
                    *active_maximized_window_identifier = Some(window_identifier.to_string());
                }
            }
            Err(error) => {
                log::error!("Failed to acquire maximized window lock: {}", error);
            }
        }
    }

    pub fn is_window_maximized(
        &self,
        window_identifier: &str,
    ) -> bool {
        self.get_maximized_window_identifier().as_deref() == Some(window_identifier)
    }

    pub fn get_tab_strip_scroll_offset(
        &self,
        tab_group_key: &str,
    ) -> f32 {
        match self.tab_strip_scroll_offsets.read() {
            Ok(tab_strip_scroll_offsets) => tab_strip_scroll_offsets
                .get(tab_group_key)
                .copied()
                .unwrap_or(0.0),
            Err(error) => {
                log::error!("Failed to acquire tab strip scroll offsets for read: {}.", error);

                0.0
            }
        }
    }

    pub fn set_tab_strip_scroll_offset(
        &self,
        tab_group_key: String,
        scroll_offset: f32,
    ) {
        match self.tab_strip_scroll_offsets.write() {
            Ok(mut tab_strip_scroll_offsets) => {
                if scroll_offset <= 0.0 {
                    tab_strip_scroll_offsets.remove(&tab_group_key);
                } else {
                    tab_strip_scroll_offsets.insert(tab_group_key, scroll_offset);
                }
            }
            Err(error) => {
                log::error!("Failed to acquire tab strip scroll offsets for write: {}.", error);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DockRootViewData;

    #[test]
    fn is_window_maximized_matches_active_identifier() {
        let dock_root_view_data = DockRootViewData::new();
        dock_root_view_data.set_maximized_window_identifier(Some(String::from("memory_viewer")));

        assert!(dock_root_view_data.is_window_maximized("memory_viewer"));
        assert!(!dock_root_view_data.is_window_maximized("project_explorer"));
    }

    #[test]
    fn tab_strip_scroll_offset_defaults_to_zero_and_round_trips() {
        let dock_root_view_data = DockRootViewData::new();

        assert_eq!(dock_root_view_data.get_tab_strip_scroll_offset("project|memory"), 0.0);

        dock_root_view_data.set_tab_strip_scroll_offset(String::from("project|memory"), 48.0);
        assert_eq!(dock_root_view_data.get_tab_strip_scroll_offset("project|memory"), 48.0);

        dock_root_view_data.set_tab_strip_scroll_offset(String::from("project|memory"), 0.0);
        assert_eq!(dock_root_view_data.get_tab_strip_scroll_offset("project|memory"), 0.0);
    }
}
