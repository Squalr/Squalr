use crate::ui::theme::Theme;
use eframe::egui::{Context, Response, Sense, Ui, Widget};
use epaint::CornerRadius;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::{rc::Rc, sync::Arc};

#[derive(Clone)]
pub struct OutputView {
    _engine_execution_context: Arc<EngineExecutionContext>,
    _context: Context,
    theme: Rc<Theme>,
}

impl OutputView {
    pub fn new(
        engine_execution_context: Arc<EngineExecutionContext>,
        context: Context,
        theme: Rc<Theme>,
    ) -> Self {
        Self {
            _engine_execution_context: engine_execution_context,
            _context: context,
            theme,
        }
    }
}

impl Widget for OutputView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (available_size_rect, response) = user_interface.allocate_exact_size(user_interface.available_size(), Sense::empty());

        response
    }
}
