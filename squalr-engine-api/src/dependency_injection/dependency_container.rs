use crate::dependency_injection::dep_tuple::DepTuple;
use anyhow::{Result, anyhow};
use std::any::{Any, type_name};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

type Callback = Box<dyn FnOnce(DependencyContainer) + Send + Sync>;

#[derive(Clone)]
pub struct DependencyContainer {
    inner: Arc<RwLock<DependencyContainerInner>>,
}

impl DependencyContainer {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(DependencyContainerInner {
                services: HashMap::new(),
                pending_callbacks: Vec::new(),
            })),
        }
    }

    pub fn register<T>(
        &self,
        instance: Arc<T>,
    ) where
        T: Send + Sync + 'static,
    {
        let key = type_name::<T>().to_string();

        // Store ready callbacks.
        let ready_callbacks = match self.inner.write() {
            Ok(mut container) => {
                container.services.insert(key, instance);
                container.collect_ready_callbacks()
            }
            Err(error) => {
                log::error!("Error acquiring dependency register lock: {}", error);
                return;
            }
        };

        // Run the callbacks now that the write lock is dropped.
        for callback in ready_callbacks {
            callback(self.clone());
        }
    }

    pub fn get_existing<T: Send + Sync + 'static>(&self) -> Result<Arc<T>> {
        let key = type_name::<T>();

        match self.inner.read() {
            Ok(container) => container
                .services
                .get(key)
                .ok_or_else(|| anyhow!("Dependency not found: {}", key))?
                .clone()
                .downcast::<T>()
                .map_err(|_| anyhow!("Dependency type mismatch for type: {}", key)),
            Err(error) => Err(anyhow!("Failed to acquire lock when getting dependency instance: {}", error)),
        }
    }

    pub fn resolve_all<T, F>(
        &self,
        callback: F,
    ) where
        T: DepTuple + 'static,
        F: FnOnce(DependencyContainer, T) + Send + Sync + 'static,
    {
        let (missing, resolved) = { (T::missing_dependencies(self), T::resolve_from(self)) };

        if missing.is_empty() {
            match resolved {
                Ok(resolved) => callback(self.clone(), resolved),
                Err(error) => log::error!("Fatal error resolving dependency for immediate resolve: {}", error),
            }
        } else {
            let stored_callback = Box::new({
                move |container: DependencyContainer| match T::resolve_from(&container) {
                    Ok(resolved) => callback(container, resolved),
                    Err(error) => log::error!("Fatal error resolving dependency for stored resolve: {}", error),
                }
            });

            match self.inner.write() {
                Ok(mut container) => container.pending_callbacks.push((missing, stored_callback)),
                Err(error) => log::error!("Failed to acquire lock when resolving dependency instance: {}", error),
            }
        }
    }
}

struct DependencyContainerInner {
    services: HashMap<String, Arc<dyn Any + Send + Sync>>,
    pending_callbacks: Vec<(HashSet<String>, Callback)>,
}

impl DependencyContainerInner {
    fn collect_ready_callbacks(&mut self) -> Vec<Callback> {
        let mut ready_callbacks = vec![];
        let mut still_pending = vec![];

        for (waiting_on, callback) in self.pending_callbacks.drain(..) {
            let unmet: HashSet<_> = waiting_on
                .into_iter()
                .filter(|id| !self.services.contains_key(id))
                .collect();

            if unmet.is_empty() {
                ready_callbacks.push(callback);
            } else {
                still_pending.push((unmet, callback));
            }
        }

        self.pending_callbacks = still_pending;
        ready_callbacks
    }
}
