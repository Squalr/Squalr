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
        view_handle: Weak<V>,
        converter: impl Fn(U) -> T + Send + Sync + 'static,
        model_setter: impl Fn(&V, ModelRc<T>) + Send + Sync + 'static,
    ) -> Self {
        ViewModelCollection {
            converter: Arc::new(converter),
            view_handle: Arc::new(Mutex::new(view_handle)),
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

        // Safely get the weak handle
        let weak_handle = view_handle.lock().unwrap().clone();

        weak_handle
            .upgrade_in_event_loop(move |handle| {
                // Do the conversion on the UI thread
                let converted: Vec<T> = source_data.into_iter().map(|item| (converter)(item)).collect();

                let current_model = Rc::new(VecModel::from(vec![]));

                for item in converted {
                    current_model.push(item);
                }

                (model_setter)(&handle, ModelRc::from(current_model));
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
