use arc_swap::ArcSwap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

pub struct WriteGuard<'lifetime, T: Clone + Send + Sync + 'static> {
    arc_swap: &'lifetime ArcSwap<T>,
    uncomitted_value_ref: Arc<T>,
    committed: bool,
}

impl<'lifetime, T: Clone + Send + Sync + 'static> WriteGuard<'lifetime, T> {
    pub fn new(arc_swap: &'lifetime ArcSwap<T>) -> Self {
        let arc = &(*arc_swap.load());
        let uncomitted_value = arc.clone();

        Self {
            arc_swap,
            uncomitted_value_ref: uncomitted_value,
            committed: false,
        }
    }

    /// Commit now (still commits on Drop unless you mark committed = true).
    pub fn commit(&mut self) {
        self.arc_swap.store(self.uncomitted_value_ref.clone());
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
        self.uncomitted_value_ref.as_ref()
    }
}

impl<'lifetime, T: Clone + Send + Sync + 'static> DerefMut for WriteGuard<'lifetime, T> {
    fn deref_mut(&mut self) -> &mut T {
        // Clones T only if Arc is shared.
        Arc::make_mut(&mut self.uncomitted_value_ref)
    }
}

impl<'lifetime, T: Clone + Send + Sync + 'static> Drop for WriteGuard<'lifetime, T> {
    fn drop(&mut self) {
        if !self.committed {
            self.arc_swap.store(self.uncomitted_value_ref.clone());
        }
    }
}
