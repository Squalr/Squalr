use anyhow::anyhow;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

pub struct DependencyContainer {
    services: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl DependencyContainer {
    pub fn new() -> Self {
        Self { services: HashMap::new() }
    }

    pub fn register(
        &mut self,
        type_id: TypeId,
        service: Arc<dyn Any + Send + Sync>,
    ) {
        self.services.insert(type_id, service);
    }

    pub fn resolve<T: Send + Sync + 'static>(&self) -> anyhow::Result<Arc<T>> {
        self.services
            .get(&TypeId::of::<T>())
            .ok_or_else(|| anyhow!("Dependency not found: {}", std::any::type_name::<T>()))?
            .clone()
            .downcast::<T>()
            .map_err(|_| anyhow!("Dependency type mismatch for type: {}", std::any::type_name::<T>()))
    }
}
