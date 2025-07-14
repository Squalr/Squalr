use crate::dependency_injection::dependency_container::DependencyContainer;
use anyhow::Result;
use std::sync::{Arc, OnceLock};

pub struct Lazy<T: Send + Sync + 'static> {
    container: DependencyContainer,
    cell: OnceLock<Result<Arc<T>>>,
}

impl<T: Send + Sync + 'static> Lazy<T> {
    pub fn new(container: DependencyContainer) -> Self {
        Self {
            container,
            cell: OnceLock::new(),
        }
    }

    pub fn get(&self) -> Result<Arc<T>> {
        match self.cell.get_or_init(|| self.container.get_existing::<T>()) {
            Ok(arc) => Ok(arc.clone()),
            Err(error) => Err(anyhow::anyhow!(error.to_string())),
        }
    }
}
