use crate::mvvm::view_model_collection::ViewModelCollection;
use slint::{ComponentHandle, ModelRc, Weak};
use std::sync::{Arc, Mutex};

pub trait ViewModel {
    fn create_view_bindings(&self);
}

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

    /// Helper function to easily create a view model collection for this view binding.
    pub fn create_collection<SourceItem, TargetItem>(
        &self,
        converter: impl Fn(SourceItem) -> TargetItem + Send + Sync + 'static,
        model_setter: impl Fn(&T, ModelRc<TargetItem>) + Send + Sync + 'static,
    ) -> ViewModelCollection<TargetItem, SourceItem, T>
    where
        TargetItem: Clone + PartialEq + 'static,
        SourceItem: Send + 'static,
    {
        // Lock once here, so the caller doesn't have to.
        // It would be nice to handle this unwrap(), but it is not clear how to best handle this without destroying our API.
        let locked_handle = self.view_handle.lock().unwrap().clone();
        ViewModelCollection::new(&locked_handle, converter, model_setter)
    }
}

impl<T: 'static + ComponentHandle> Clone for ViewBinding<T> {
    fn clone(&self) -> Self {
        Self {
            view_handle: self.view_handle.clone(),
        }
    }
}
