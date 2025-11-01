use crate::{models::docking::docking_manager::DockingManager, ui::theme::Theme};
use eframe::egui::Context;
use squalr_engine_api::{dependency_injection::dependency_container::DependencyContainer, engine::engine_execution_context::EngineExecutionContext};
use std::sync::{Arc, RwLock};

/// Contains commonly used state shared between most widgets.
#[derive(Clone)]
pub struct AppContext {
    pub context: Context,
    pub theme: Arc<Theme>,
    pub docking_manager: Arc<RwLock<DockingManager>>,
    pub engine_execution_context: Arc<EngineExecutionContext>,

    /// Allows for registering and listening for dependencies.
    pub dependency_container: Arc<DependencyContainer>,
}

impl AppContext {
    pub fn new(
        context: Context,
        theme: Arc<Theme>,
        docking_manager: Arc<RwLock<DockingManager>>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) -> Self {
        let dependency_container = Arc::new(DependencyContainer::new());

        Self {
            context,
            theme,
            docking_manager,
            engine_execution_context,
            dependency_container,
        }
    }
}
