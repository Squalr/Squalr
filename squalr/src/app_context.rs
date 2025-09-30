use crate::{models::docking::docking_manager::DockingManager, ui::theme::Theme};
use eframe::egui::Context;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::{
    rc::Rc,
    sync::{Arc, RwLock},
};

/// Contains commonly used state shared between most widgets.
#[derive(Clone)]
pub struct AppContext {
    pub context: Context,
    pub theme: Rc<Theme>,
    pub docking_manager: Rc<RwLock<DockingManager>>,
    pub engine_execution_context: Arc<EngineExecutionContext>,
}

impl AppContext {
    pub fn new(
        context: Context,
        theme: Rc<Theme>,
        docking_manager: Rc<RwLock<DockingManager>>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) -> Self {
        Self {
            context,
            theme,
            docking_manager,
            engine_execution_context,
        }
    }
}
