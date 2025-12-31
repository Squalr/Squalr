use arc_swap::ArcSwap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

pub struct WriteGuard<'lifetime, T: Clone + Send + Sync + 'static> {
    swap: &'lifetime ArcSwap<T>,
    value: Arc<T>,
    committed: bool,
}

impl<'lifetime, T: Clone + Send + Sync + 'static> WriteGuard<'lifetime, T> {
    pub fn new(swap: &'lifetime ArcSwap<T>) -> Self {
        // ArcSwap<T>::load() -> Guard<Arc<T>>
        let value = (*swap.load()).clone(); // clone the Arc<T>
        Self { swap, value, committed: false }
    }

    /// Commit now (still commits on Drop unless you mark committed=true)
    pub fn commit(&mut self) {
        self.swap.store(self.value.clone()); // clone Arc<T> (cheap)
        self.committed = true;
    }

    /// Prevent commit on Drop
    pub fn abort(&mut self) {
        self.committed = true;
    }
}

impl<'lifetime, T: Clone + Send + Sync + 'static> Deref for WriteGuard<'lifetime, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.value.as_ref()
    }
}

impl<'lifetime, T: Clone + Send + Sync + 'static> DerefMut for WriteGuard<'lifetime, T> {
    fn deref_mut(&mut self) -> &mut T {
        Arc::make_mut(&mut self.value) // clones T only if Arc is shared
    }
}

impl<'lifetime, T: Clone + Send + Sync + 'static> Drop for WriteGuard<'lifetime, T> {
    fn drop(&mut self) {
        if !self.committed {
            self.swap.store(self.value.clone());
        }
    }
}
