use slint::{ComponentHandle, Weak};
use std::sync::{Arc, Mutex};

pub struct ViewBinding<T: 'static + ComponentHandle> {
    view_handle: Arc<Mutex<Weak<T>>>,
}

impl<T: 'static + ComponentHandle> ViewBinding<T> {
    pub fn new(view_handle: Weak<T>) -> Self {
        Self {
            view_handle: Arc::new(Mutex::new(view_handle)),
        }
    }

    /// Gets the handle to the view being bound.
    pub fn get_view_handle(&self) -> &Arc<Mutex<Weak<T>>> {
        return &self.view_handle;
    }

    /// Executes a function on the UI thread, while also capturing the window view and this view binding as variables.
    pub fn execute_on_ui_thread<F>(
        &self,
        f: F,
    ) where
        F: FnOnce(&T, ViewBinding<T>) + Send + 'static,
    {
        if let Err(e) = self.view_handle.lock() {
            log::error!("Failed to acquire view handle lock: {}", e);
            return;
        }

        let handle = self.view_handle.lock().unwrap();
        let view_model = self.clone();
        if let Err(e) = handle.upgrade_in_event_loop(move |view| f(&view, view_model)) {
            log::error!("Failed to upgrade view in event loop: {}", e);
        }
    }
}

impl<T: 'static + ComponentHandle> Clone for ViewBinding<T> {
    fn clone(&self) -> Self {
        Self {
            view_handle: self.view_handle.clone(),
        }
    }
}
