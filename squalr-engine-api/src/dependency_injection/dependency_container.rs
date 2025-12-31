use crate::dependency_injection::dep_tuple::DepTuple;
use crate::dependency_injection::dependency::Dependency;
use anyhow::{Result, anyhow};
use arc_swap::ArcSwap;
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
        instance: T,
    ) -> Dependency<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let key = type_name::<T>().to_string();

        // Store ready callbacks.
        let ready_callbacks = match self.inner.write() {
            Ok(mut container) => {
                container
                    .services
                    .insert(key, Arc::new(ArcSwap::new(Arc::new(instance))));
                container.collect_ready_callbacks()
            }
            Err(error) => {
                log::error!("Error acquiring dependency register lock: {}", error);

                return self.get_dependency();
            }
        };

        // Run callbacks now that the write lock is dropped.
        for callback in ready_callbacks {
            callback(self.clone());
        }

        // Return a lazy `Dependency<T>`.
        self.get_dependency()
    }

    pub fn get_existing<T>(&self) -> Result<Arc<ArcSwap<T>>>
    where
        T: Send + Sync + 'static,
    {
        let key = type_name::<T>();
        let container = self
            .inner
            .read()
            .map_err(|error| anyhow!("Failed to lock container: {error}"))?;
        let svc = container
            .services
            .get(key)
            .ok_or_else(|| anyhow!("Dependency not found: {}", key))?;

        // Clone arc and downcast ArcSwap's inner type.
        let arc_any = Arc::clone(svc);
        Arc::downcast::<ArcSwap<T>>(arc_any).map_err(|_| anyhow!("Type mismatch for dependency {}", key))
    }

    pub fn get_dependency<T>(&self) -> Dependency<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        Dependency::new(self.clone())
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
