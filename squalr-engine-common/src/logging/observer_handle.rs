use std::sync::Arc;
use std::hash::{Hash, Hasher};
use crate::logging::logger_observer::ILoggerObserver;

pub struct ObserverHandle(Arc<dyn ILoggerObserver + Send + Sync>);

impl PartialEq for ObserverHandle {
    fn eq(
        &self,
        other: &Self
    ) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ObserverHandle {}

impl Hash for ObserverHandle {
    fn hash<H: Hasher>(
        &self,
        state: &mut H
    ) {
        let ptr: *const dyn ILoggerObserver = &*self.0;
        let thin_ptr: *const () = ptr as *const ();
        thin_ptr.hash(state);
    }
}

impl ObserverHandle {
    pub fn new(
        observer: Arc<dyn ILoggerObserver + Send + Sync>
    ) -> Self {
        ObserverHandle(observer)
    }

    pub fn get(
        &self
    ) -> &Arc<dyn ILoggerObserver + Send + Sync> {
        &self.0
    }
}
