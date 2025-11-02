use crate::ui::widgets::docking::dockable_window::DockableWindow;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct DockRootViewData {
    // JIRA: Maybe make this a hashmap of id to window for faster lookups (ie sibling tab ids -> window name).
    pub windows: Arc<RwLock<Vec<Box<dyn DockableWindow>>>>,
}

impl DockRootViewData {
    pub fn new() -> Self {
        Self {
            windows: Arc::new(RwLock::new(Vec::new())),
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
}
