use slint::{ComponentHandle, Weak};
use std::sync::{Arc, Mutex};

pub trait ViewModel {
    fn create_view_bindings(&self);
}

pub struct ViewModelBase<T: 'static + ComponentHandle> {
    view_handle: Arc<Mutex<Weak<T>>>,
}

impl<T: 'static + ComponentHandle> ViewModelBase<T> {
    pub fn new(view_handle: Weak<T>) -> Self {
        Self {
            view_handle: Arc::new(Mutex::new(view_handle)),
        }
    }

    pub fn execute_on_ui_thread<F>(
        &self,
        f: F,
    ) where
        F: FnOnce(&T, ViewModelBase<T>) + Send + 'static,
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

    pub fn get_view_handle(&self) -> &Arc<Mutex<Weak<T>>> {
        return &self.view_handle;
    }
}

impl<T: 'static + ComponentHandle> Clone for ViewModelBase<T> {
    fn clone(&self) -> Self {
        Self {
            view_handle: self.view_handle.clone(),
        }
    }
}
