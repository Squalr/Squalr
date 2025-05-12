use crate::dependency_injection::dep_tuple::DepTuple;
use anyhow::{Result, anyhow};
use std::any::{Any, TypeId};
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
        let type_id = TypeId::of::<T>();

        match self.inner.write() {
            Ok(mut container) => {
                container.services.insert(type_id, instance);
                container.process_pending_callbacks(self.clone());
            }
            Err(err) => log::error!("Error acquiring dependency register lock: {}", err),
        }
    }

    pub fn get_existing<T: Send + Sync + 'static>(&self) -> Result<Arc<T>> {
        let guard = self.inner.read().unwrap();
        guard
            .services
            .get(&TypeId::of::<T>())
            .ok_or_else(|| anyhow!("Dependency not found: {}", std::any::type_name::<T>()))?
            .clone()
            .downcast::<T>()
            .map_err(|_| anyhow!("Dependency type mismatch for type: {}", std::any::type_name::<T>()))
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
            if let Ok(resolved) = resolved {
                callback(self.clone(), resolved);
            }
        } else {
            let cb = Box::new({
                move |container: DependencyContainer| match T::resolve_from(&container) {
                    Ok(resolved) => callback(container, resolved),
                    Err(err) => log::error!("Fatal error resolving internal dependency: {}", err),
                }
            });

            let mut guard = self.inner.write().unwrap();
            guard.pending_callbacks.push((missing, cb));
        }
    }
}

struct DependencyContainerInner {
    pub services: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    pending_callbacks: Vec<(HashSet<TypeId>, Callback)>,
}

impl DependencyContainerInner {
    fn process_pending_callbacks(
        &mut self,
        container: DependencyContainer,
    ) {
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

        for cb in ready_callbacks {
            cb(container.clone());
        }
    }
}
