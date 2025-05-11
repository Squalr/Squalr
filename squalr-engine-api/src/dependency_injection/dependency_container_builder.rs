use crate::dependency_injection::dependency_container::DependencyContainer;
use std::any::Any;
use std::any::TypeId;
use std::sync::Arc;

type Factory = Box<dyn Fn(&DependencyContainer) -> anyhow::Result<Arc<dyn Any + Send + Sync>>>;

pub struct DependencyContainerBuilder {
    factories: Vec<(TypeId, String, Factory)>,
}

impl DependencyContainerBuilder {
    pub fn new() -> Self {
        Self { factories: Vec::new() }
    }

    pub fn register<T, F>(
        &mut self,
        factory: F,
    ) where
        T: Send + Sync + 'static,
        F: Fn(&DependencyContainer) -> anyhow::Result<Arc<T>> + 'static,
    {
        let type_id = TypeId::of::<T>();
        let type_name = std::any::type_name::<T>().to_string();

        self.factories.push((
            type_id,
            type_name,
            Box::new(move |container| {
                let val = factory(container)?;
                Ok(val as Arc<dyn Any + Send + Sync>)
            }),
        ));
    }

    pub fn build(&self) -> anyhow::Result<DependencyContainer> {
        let mut container = DependencyContainer::new();

        for (type_id, type_name, factory) in &self.factories {
            match factory(&container) {
                Ok(instance) => {
                    container.register(*type_id, instance);
                }
                Err(err) => {
                    log::error!("Failed to create instance for type `{}`: {}", type_name, err);
                }
            }
        }

        Ok(container)
    }
}
