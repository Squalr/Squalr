use slint::Model;
use slint::{ComponentHandle, ModelRc, VecModel, Weak};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub struct ViewModelCollection<T, U, V>
where
    T: Clone + PartialEq + 'static,
    U: Send + 'static,
    V: 'static + ComponentHandle,
{
    converter: Arc<dyn Fn(U) -> T + Send + Sync>,
    view_handle: Arc<Mutex<Weak<V>>>,
    model_setter: Arc<dyn Fn(&V, ModelRc<T>) + Send + Sync>,
}

impl<T, U, V> ViewModelCollection<T, U, V>
where
    T: Clone + PartialEq + 'static,
    U: Send + 'static,
    V: 'static + ComponentHandle,
{
    pub fn new(
        view_handle: &Weak<V>,
        converter: impl Fn(U) -> T + Send + Sync + 'static,
        model_setter: impl Fn(&V, ModelRc<T>) + Send + Sync + 'static,
    ) -> Self {
        ViewModelCollection {
            converter: Arc::new(converter),
            view_handle: Arc::new(Mutex::new(view_handle.clone())),
            model_setter: Arc::new(model_setter),
        }
    }

    pub fn update_from_source(
        &self,
        source_data: Vec<U>,
    ) {
        let converter = self.converter.clone();
        let view_handle = self.view_handle.clone();
        let model_setter = self.model_setter.clone();
        let weak_handle = view_handle.lock().unwrap().clone();

        weak_handle
            .upgrade_in_event_loop(move |handle| {
                let converted: Vec<T> = source_data.into_iter().map(|item| (converter)(item)).collect();
                let model = VecModel::from(vec![]);
                let mut has_changes = false;

                // Get the current state.
                let current_row_count = model.row_count();

                // Update existing entries and add new ones.
                for (index, new_entry) in converted.iter().enumerate() {
                    if index < current_row_count {
                        // Check if we need to update.
                        if let Some(current) = model.row_data(index) {
                            if current != *new_entry {
                                model.set_row_data(index, new_entry.clone());
                                has_changes = true;
                            }
                        }
                    } else {
                        // Add new entry.
                        model.push(new_entry.clone());
                        has_changes = true;
                    }
                }

                // Remove excess entries if new data is shorter.
                while model.row_count() > converted.len() {
                    model.remove(model.row_count() - 1);
                    has_changes = true;
                }

                // Only update the UI if something actually changed.
                if has_changes {
                    let model_rc = ModelRc::from(Rc::new(model));
                    (model_setter)(&handle, model_rc);
                }
            })
            .expect("Failed to schedule UI update");
    }
}

impl<T, U, V> Clone for ViewModelCollection<T, U, V>
where
    T: Clone + PartialEq + 'static,
    U: Send + 'static,
    V: 'static + ComponentHandle,
{
    fn clone(&self) -> Self {
        ViewModelCollection {
            converter: self.converter.clone(),
            view_handle: self.view_handle.clone(),
            model_setter: self.model_setter.clone(),
        }
    }
}
