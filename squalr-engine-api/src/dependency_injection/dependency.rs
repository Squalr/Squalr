use crate::dependency_injection::dependency_container::DependencyContainer;
use crate::dependency_injection::write_guard::WriteGuard;
use anyhow::Result;
use anyhow::anyhow;
use arc_swap::ArcSwap;
use arc_swap::Guard;
use std::sync::{Arc, OnceLock};

/// A clone-safe wrapper for injected and lock-free dependencies. Requires clonable types to achieve lock-free.
pub struct Dependency<T: Clone + Send + Sync + 'static> {
    container: DependencyContainer,
    instance: Arc<OnceLock<Result<Arc<ArcSwap<T>>>>>,
}

impl<T: Clone + Send + Sync + 'static> Clone for Dependency<T> {
    fn clone(&self) -> Self {
        Self {
            container: self.container.clone(),
            instance: self.instance.clone(),
        }
    }
}

impl<T: Clone + Send + Sync + 'static> Dependency<T> {
    pub fn new(container: DependencyContainer) -> Self {
        Self {
            container,
            instance: Arc::new(OnceLock::new()),
        }
    }

    /// Get the Arc<RwLock<T>> that lives inside OnceLock
    /// This reference lives as long as &self, so guard lifetimes work.
    fn get_shared_lock(&self) -> Result<&ArcSwap<T>> {
        let slot = self.instance.get_or_init(|| self.container.get_existing::<T>());

        match slot {
            Ok(shared_lock) => Ok(shared_lock),
            Err(error) => Err(anyhow!(error.to_string())),
        }
    }

    /// Acquire a read guard.
    pub fn read(
        &self,
        error_context: &'static str,
    ) -> Option<Guard<Arc<T>>> {
        match self.get_shared_lock() {
            Ok(shared_lock) => Some(shared_lock.load()),
            Err(error) => {
                log::error!("Failed to acquire read on dependency: {}, context: {}", error, error_context);
                None
            }
        }
    }

    /// Acquire a write guard.
    pub fn write(
        &self,
        error_context: &'static str,
    ) -> Option<WriteGuard<'_, T>> {
        match self.get_shared_lock() {
            Ok(shared_lock) => Some(WriteGuard::new(shared_lock)),
            Err(error) => {
                log::error!("Failed to acquire write on dependency: {}, context: {}", error, error_context);
                None
            }
        }
    }
}
