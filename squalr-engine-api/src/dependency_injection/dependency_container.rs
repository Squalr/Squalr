use anyhow::{Result, anyhow};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

type Factory = Box<dyn Fn(&DependencyContainer) -> Result<Arc<dyn Any + Send + Sync>>>;

pub struct DependencyContainer {
    services: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    factories: HashMap<TypeId, (String, Factory)>,
    built: bool,
}

impl DependencyContainer {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            factories: HashMap::new(),
            built: false,
        }
    }

    pub fn register<T, F>(
        &mut self,
        factory: F,
    ) where
        T: Send + Sync + 'static,
        F: Fn(&DependencyContainer) -> Result<Arc<T>> + 'static,
    {
        if self.built {
            log::error!("Attempted to register a dependency after the container has been built!")
        }

        let type_id = TypeId::of::<T>();
        let type_name = std::any::type_name::<T>().to_string();

        self.factories.insert(
            type_id,
            (
                type_name,
                Box::new(move |container| {
                    let val = factory(container)?;
                    Ok(val as Arc<dyn Any + Send + Sync>)
                }),
            ),
        );
    }

    pub fn build(&mut self) -> Result<()> {
        if self.built {
            return Err(anyhow!("Dependency container has already been built."));
        }

        for (type_id, (type_name, factory)) in &self.factories {
            match factory(self) {
                Ok(instance) => {
                    self.services.insert(*type_id, instance);
                }
                Err(err) => {
                    log::error!("Failed to create instance for type `{}`: {}", type_name, err);
                }
            }
        }

        // Clear factories to free memory, as these are no longer required.
        self.factories.clear();
        self.built = true;

        Ok(())
    }

    pub fn resolve<T: Send + Sync + 'static>(&self) -> Result<Arc<T>> {
        self.services
            .get(&TypeId::of::<T>())
            .ok_or_else(|| anyhow!("Dependency not found: {}", std::any::type_name::<T>()))?
            .clone()
            .downcast::<T>()
            .map_err(|_| anyhow!("Dependency type mismatch for type: {}", std::any::type_name::<T>()))
    }
}
