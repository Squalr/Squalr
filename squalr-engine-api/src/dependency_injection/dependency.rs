use crate::dependency_injection::dependency_container::DependencyContainer;
use anyhow::Result;
use anyhow::anyhow;
use std::sync::{Arc, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// A clone-safe wrapper for injected and read-write locked dependencies.
pub struct Dependency<T: Send + Sync + 'static> {
    container: DependencyContainer,
    instance: Arc<OnceLock<Result<Arc<RwLock<T>>>>>,
}

impl<T: Send + Sync + 'static> Clone for Dependency<T> {
    fn clone(&self) -> Self {
        Self {
            container: self.container.clone(),
            instance: self.instance.clone(),
        }
    }
}

impl<T: Send + Sync + 'static> Dependency<T> {
    pub fn new(container: DependencyContainer) -> Self {
        Self {
            container,
            instance: Arc::new(OnceLock::new()),
        }
    }

    /// Get the Arc<RwLock<T>> that lives inside OnceLock
    /// This reference lives as long as &self, so guard lifetimes work.
    fn get_shared_lock(&self) -> Result<&Arc<RwLock<T>>> {
        let slot = self.instance.get_or_init(|| self.container.get_existing::<T>());

        match slot {
            Ok(shared_lock) => Ok(shared_lock),
            Err(error) => Err(anyhow!(error.to_string())),
        }
    }

    /// Acquire a read guard
    pub fn read(&self) -> Result<RwLockReadGuard<'_, T>> {
        let shared_lock = self.get_shared_lock()?;

        shared_lock
            .read()
            .map_err(|error| anyhow!("RwLock read poisoned: {error}"))
    }

    /// Acquire a write guard
    pub fn write(&self) -> Result<RwLockWriteGuard<'_, T>> {
        let shared_lock = self.get_shared_lock()?;

        shared_lock
            .write()
            .map_err(|error| anyhow!("RwLock write poisoned: {error}"))
    }
}
