use slint::{ComponentHandle, Model, ModelRc, VecModel, Weak};
use std::sync::{Arc, Mutex};

/// Defines a collection (a vector) of data that automatically syncs to the UI.
/// This is done by using the given converter to convert data to a type that the UI recognizes,
/// and by retrieving the existing model (via a model_getter) so we can update it *in place*.
pub struct ViewModelCollectionBinding<T, U, V>
where
    T: Clone + 'static,
    U: Send + 'static,
    V: 'static + ComponentHandle,
{
    /// A function that converts your `U` into a `T` recognized by the UI.
    converter: Arc<dyn Fn(U) -> T + Send + Sync>,

    /// An optional custom comparer for T. If `None`, fall back to `==`.
    /// If provided, should return true if the two T items are "equal" in your sense of equality.
    comparer: Arc<dyn Fn(&T, &T) -> bool + Send + Sync>,

    /// The handle to the UI component (for scheduling updates).
    view_handle: Arc<Mutex<Weak<V>>>,

    /// A function that sets the model in the UI, if we create a new one.
    model_setter: Arc<dyn Fn(&V, ModelRc<T>) + Send + Sync>,

    /// A function that retrieves the current model from the UI (if any).
    model_getter: Arc<dyn Fn(&V) -> ModelRc<T> + Send + Sync>,
}

impl<T, U, V> ViewModelCollectionBinding<T, U, V>
where
    T: Clone + PartialEq + 'static,
    U: Send + 'static,
    V: 'static + ComponentHandle,
{
    /// Create a new ViewModelCollectionBinding with:
    /// - A converter: `U -> T`
    /// - A setter for storing the final `ModelRc<T>` (if we create a new one)
    /// - A getter for retrieving the existing `ModelRc<T>` (if any)
    /// - An optional comparer for T. If `None`, we default to using `==`.
    pub fn new(
        view_handle: &Weak<V>,
        model_setter: impl Fn(&V, ModelRc<T>) + Send + Sync + 'static,
        model_getter: impl Fn(&V) -> ModelRc<T> + Send + Sync + 'static,
        converter: impl Fn(U) -> T + Send + Sync + 'static,
        comparer: impl Fn(&T, &T) -> bool + Send + Sync + 'static,
    ) -> Self {
        ViewModelCollectionBinding {
            converter: Arc::new(converter),
            comparer: Arc::new(comparer),
            view_handle: Arc::new(Mutex::new(view_handle.clone())),
            model_setter: Arc::new(model_setter),
            model_getter: Arc::new(model_getter),
        }
    }

    /// Called whenever the source data changes, so we can update the UI model accordingly.
    /// Grab the existing model (if any) and edit it in place. If no model exists or types
    /// mismatch, create a new one and store it via the `model_setter` closure.
    pub fn update_from_source(
        &self,
        source_data: Vec<U>,
    ) {
        let converter = self.converter.clone();
        let comparer = self.comparer.clone();
        let view_handle = self.view_handle.clone();
        let model_setter = self.model_setter.clone();
        let model_getter = self.model_getter.clone();

        let weak_handle = view_handle.lock().unwrap().clone();

        // Schedule a UI update via Slint’s event loop.
        weak_handle
            .upgrade_in_event_loop(move |handle| {
                // Try to grab the existing model from the UI.
                let existing_model_rc = (model_getter)(&handle);

                let model_rc = if existing_model_rc
                    .as_any()
                    .downcast_ref::<VecModel<T>>()
                    .is_some()
                {
                    // We can re-use the existing VecModel
                    existing_model_rc
                } else {
                    // Downcast failed, create and set a new VecModel
                    let new_model = VecModel::from(vec![]);
                    let new_model_rc = ModelRc::new(new_model);
                    (model_setter)(&handle, new_model_rc.clone());
                    new_model_rc
                };

                // At this point, we are guaranteed to have a VecModel<T>.
                let vec_model = model_rc
                    .as_any()
                    .downcast_ref::<VecModel<T>>()
                    .expect("The model in the UI is not a VecModel<T>—type mismatch!");

                // Convert the incoming data items into the UI type T.
                let converted: Vec<T> = source_data.into_iter().map(|item| (converter)(item)).collect();

                // In-place update for as many entries that overlap.
                let mut index = 0;
                while index < converted.len() && index < vec_model.row_count() {
                    let current_row = vec_model.row_data(index).unwrap();
                    let new_row = &converted[index];
                    if !comparer(&current_row, new_row) {
                        vec_model.set_row_data(index, new_row.clone());
                    }
                    index += 1;
                }

                // If the new data has more items than the existing model.
                while index < converted.len() {
                    vec_model.push(converted[index].clone());
                    index += 1;
                }

                // If the new data has fewer items than the existing model.
                while vec_model.row_count() > converted.len() {
                    vec_model.remove(vec_model.row_count() - 1);
                }

                // IMPORTANT: We do *not* call (model_setter) here unless we created
                // a new `VecModel` above. The existing model is already "wired up"
                // and the in-place modifications have triggered the necessary UI updates.
            })
            .expect("Failed to schedule UI update");
    }
}

impl<T, U, V> Clone for ViewModelCollectionBinding<T, U, V>
where
    T: Clone + 'static,
    U: Send + 'static,
    V: 'static + ComponentHandle,
{
    fn clone(&self) -> Self {
        ViewModelCollectionBinding {
            converter: self.converter.clone(),
            comparer: self.comparer.clone(),
            view_handle: self.view_handle.clone(),
            model_setter: self.model_setter.clone(),
            model_getter: self.model_getter.clone(),
        }
    }
}
