use slint::{ModelRc, VecModel};
use std::rc::Rc;
use std::sync::Arc;

pub struct ViewModelCollection<T, U>
where
    T: Clone + 'static,
{
    converter: Arc<dyn Fn(U) -> T + Send + Sync>,
}

impl<T, U> ViewModelCollection<T, U>
where
    T: Clone + 'static,
{
    pub fn new(converter: impl Fn(U) -> T + Send + Sync + 'static) -> Self {
        ViewModelCollection {
            converter: Arc::new(converter),
        }
    }

    pub fn create_empty_model() -> ModelRc<T> {
        ModelRc::from(Rc::new(VecModel::from(vec![])))
    }

    pub fn converter(&self) -> Arc<dyn Fn(U) -> T + Send + Sync> {
        self.converter.clone()
    }
}

impl<T, U> Clone for ViewModelCollection<T, U>
where
    T: Clone + 'static,
{
    fn clone(&self) -> Self {
        ViewModelCollection {
            converter: self.converter.clone(),
        }
    }
}
