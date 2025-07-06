use slint::{ComponentHandle, Weak};
use std::sync::{Arc, Mutex};

pub struct ViewBinding<T: 'static + ComponentHandle> {
    view_handle: Mutex<Weak<T>>,
}

impl<T: 'static + ComponentHandle> ViewBinding<T> {
    pub fn new(view_handle: Weak<T>) -> Self {
        Self {
            view_handle: Mutex::new(view_handle),
        }
    }

    /// Gets the handle to the view being bound.
    pub fn get_view_handle(&self) -> &Mutex<Weak<T>> {
        &self.view_handle
    }

    /// Executes a function on the UI thread, while also capturing the window view and this view binding as variables.
    /// If we are already on the UI thread, the callback is executed immediately.
    pub fn execute_on_ui_thread<F>(
        self: &Arc<Self>,
        callback: F,
    ) where
        F: FnOnce(&T, Arc<ViewBinding<T>>) + Send + 'static,
    {
        // Attempt to lock the Arc<Mutex<Weak<T>>>.
        let Ok(handle_guard) = self.view_handle.lock() else {
            log::error!("Failed to acquire view handle lock");
            return;
        };

        let handle = handle_guard;
        let view_binding = self.clone();

        // Try to upgrade immediately (as if we're on the UI thread).
        match handle.upgrade() {
            Some(view) => {
                // Success: call immediately
                callback(&view, view_binding);
            }
            None => {
                // If the immediate upgrade fails, schedule in the event loop
                if let Err(error) = handle.upgrade_in_event_loop(move |view| callback(&view, view_binding)) {
                    log::error!("Failed to upgrade view in event loop: {}", error);
                }
            }
        }
    }
}
