use eframe::egui::Context;
use std::sync::RwLock;

/// Tracks which docked window owns window-level keyboard shortcuts.
pub struct WindowFocusManager {
    focused_window_identifier: RwLock<Option<String>>,
}

impl WindowFocusManager {
    pub fn new() -> Self {
        Self {
            focused_window_identifier: RwLock::new(None),
        }
    }

    /// Sets the focused docked window.
    pub fn focus_window(
        &self,
        window_identifier: &str,
    ) {
        match self.focused_window_identifier.write() {
            Ok(mut focused_window_identifier) => {
                *focused_window_identifier = Some(window_identifier.to_string());
            }
            Err(error) => {
                log::error!("Failed to acquire window focus write lock: {}.", error);
            }
        }
    }

    /// Gets the currently focused docked window identifier.
    pub fn get_focused_window_identifier(&self) -> Option<String> {
        match self.focused_window_identifier.read() {
            Ok(focused_window_identifier) => focused_window_identifier.clone(),
            Err(error) => {
                log::error!("Failed to acquire window focus read lock: {}.", error);
                None
            }
        }
    }

    /// Returns true when the specified window is focused.
    pub fn is_window_focused(
        &self,
        window_identifier: &str,
    ) -> bool {
        self.get_focused_window_identifier()
            .as_deref()
            .is_some_and(|focused_window_identifier| focused_window_identifier == window_identifier)
    }

    /// Returns true when a window may handle pane-level keyboard shortcuts.
    pub fn can_window_handle_shortcuts(
        &self,
        context: &Context,
        window_identifier: &str,
    ) -> bool {
        self.is_window_focused(window_identifier) && !context.wants_keyboard_input()
    }
}

impl Default for WindowFocusManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::WindowFocusManager;
    use eframe::egui::Context;

    #[test]
    fn focus_window_replaces_previous_focus_owner() {
        let window_focus_manager = WindowFocusManager::new();

        window_focus_manager.focus_window("window_project_explorer");
        assert!(window_focus_manager.is_window_focused("window_project_explorer"));

        window_focus_manager.focus_window("window_symbol_explorer");
        assert!(!window_focus_manager.is_window_focused("window_project_explorer"));
        assert!(window_focus_manager.is_window_focused("window_symbol_explorer"));
    }

    #[test]
    fn can_window_handle_shortcuts_requires_focus() {
        let context = Context::default();
        let window_focus_manager = WindowFocusManager::new();

        assert!(!window_focus_manager.can_window_handle_shortcuts(&context, "window_project_explorer"));

        window_focus_manager.focus_window("window_project_explorer");

        assert!(window_focus_manager.can_window_handle_shortcuts(&context, "window_project_explorer"));
        assert!(!window_focus_manager.can_window_handle_shortcuts(&context, "window_symbol_explorer"));
    }
}
