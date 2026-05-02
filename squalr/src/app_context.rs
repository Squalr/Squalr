use crate::{models::docking::docking_manager::DockingManager, models::window_focus_manager::WindowFocusManager, ui::theme::Theme};
use eframe::egui::Context;
use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
use squalr_engine_session::engine_unprivileged_state::EngineUnprivilegedState;
use std::sync::{Arc, RwLock};

/// Contains commonly used state shared between most widgets.
#[derive(Clone)]
pub struct AppContext {
    pub context: Context,
    pub theme: Arc<Theme>,
    pub docking_manager: Arc<RwLock<DockingManager>>,
    pub window_focus_manager: Arc<WindowFocusManager>,
    pub engine_unprivileged_state: Arc<EngineUnprivilegedState>,

    /// Allows for registering and listening for dependencies.
    pub dependency_container: Arc<DependencyContainer>,
}

impl AppContext {
    pub fn new(
        context: Context,
        theme: Arc<Theme>,
        docking_manager: Arc<RwLock<DockingManager>>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) -> Self {
        let dependency_container = Arc::new(DependencyContainer::new());
        let window_focus_manager = Arc::new(WindowFocusManager::new());

        Self {
            context,
            theme,
            docking_manager,
            window_focus_manager,
            engine_unprivileged_state,
            dependency_container,
        }
    }
}
