use slint::{ComponentHandle, ModelRc, Weak};
use std::sync::{Arc, Mutex};

/// Defines a collection (a vector) of data that automatically syncs to the UI.
/// This is done by using the given converter to convert data to a type that the UI recognizes,
/// and by retrieving the existing model (via a model_getter) so we can update it *in place*.
pub struct ViewPropertyBinding<T, U, V>
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

impl<T, U, V> ViewPropertyBinding<T, U, V>
where
    T: Clone + PartialEq + 'static,
    U: Send + 'static,
    V: 'static + ComponentHandle,
{
    /// Create a new ViewPropertyBinding with:
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
        ViewPropertyBinding {
            converter: Arc::new(converter),
            comparer: Arc::new(comparer),
            view_handle: Arc::new(Mutex::new(view_handle.clone())),
            model_setter: Arc::new(model_setter),
            model_getter: Arc::new(model_getter),
        }
    }

    pub fn update_from_source(
        &self,
        source_data: U,
    ) {
        let converter = self.converter.clone();
        let comparer = self.comparer.clone();
        let view_handle = self.view_handle.clone();
        let model_setter = self.model_setter.clone();
        let model_getter = self.model_getter.clone();

        let weak_handle = view_handle.lock().unwrap().clone();

        // Schedule a UI update via Slint’s event loop.
        let _ = weak_handle.upgrade_in_event_loop(move |handle| {
            // Convert the incoming data items into the UI type T.
            let converted: T = (converter)(source_data);

            // TODO idk
            // (model_setter)(&handle, ModelRc::new(converted));
        });
    }
}

impl<T, U, V> Clone for ViewPropertyBinding<T, U, V>
where
    T: Clone + 'static,
    U: Send + 'static,
    V: 'static + ComponentHandle,
{
    fn clone(&self) -> Self {
        ViewPropertyBinding {
            converter: self.converter.clone(),
            comparer: self.comparer.clone(),
            view_handle: self.view_handle.clone(),
            model_setter: self.model_setter.clone(),
            model_getter: self.model_getter.clone(),
        }
    }
}
