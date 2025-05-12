use crate::ConversionsViewModelBindings;
use crate::FloatingPointToleranceView;
use crate::MainWindowView;
use crate::MemoryAlignmentView;
use crate::MemoryReadModeView;
use slint::ComponentHandle;
use slint::SharedString;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm_macros::create_view_bindings;
use squalr_engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
use squalr_engine_common::conversions::Conversions;
use std::sync::Arc;

pub struct ConversionsViewModel {
    _view_binding: Arc<ViewBinding<MainWindowView>>,
    _engine_execution_context: Arc<EngineExecutionContext>,
}

impl ConversionsViewModel {
    pub fn register(dependency_container: &DependencyContainer) {
        dependency_container.resolve_all(Self::on_dependencies_resolved);
    }

    fn on_dependencies_resolved(
        dependency_container: DependencyContainer,
        (view_binding, engine_execution_context): (Arc<ViewBinding<MainWindowView>>, Arc<EngineExecutionContext>),
    ) {
        let view_model = Arc::new(ConversionsViewModel {
            _view_binding: view_binding.clone(),
            _engine_execution_context: engine_execution_context,
        });

        create_view_bindings!(view_binding, {
            ConversionsViewModelBindings => {
                on_convert_hex_to_dec(data_value: SharedString) -> [] -> Self::on_convert_hex_to_dec,
                on_convert_dec_to_hex(data_value: SharedString) -> [] -> Self::on_convert_dec_to_hex,
                on_get_memory_alignment_string(memory_alignment: MemoryAlignmentView) -> [] -> Self::on_get_memory_alignment_string,
                on_get_memory_read_mode_string(memory_read_mode: MemoryReadModeView) -> [] -> Self::on_get_memory_read_mode_string,
                on_get_floating_point_tolerance_string(floating_point_tolerance: FloatingPointToleranceView) -> [] -> Self::on_get_floating_point_tolerance_string,
            }
        });

        dependency_container.register::<ConversionsViewModel>(view_model);
    }

    fn on_convert_hex_to_dec(data_value: SharedString) -> SharedString {
        if let Ok(new_value) = Conversions::hex_to_dec(&data_value) {
            SharedString::from(new_value)
        } else {
            data_value
        }
    }

    fn on_convert_dec_to_hex(data_value: SharedString) -> SharedString {
        if let Ok(new_value) = Conversions::dec_to_hex(&data_value, false) {
            SharedString::from(new_value)
        } else {
            data_value
        }
    }

    fn on_get_memory_alignment_string(memory_alignment: MemoryAlignmentView) -> SharedString {
        match memory_alignment {
            MemoryAlignmentView::Alignment1 => "1-byte aligned".into(),
            MemoryAlignmentView::Alignment2 => "2-byte aligned".into(),
            MemoryAlignmentView::Alignment4 => "4-byte aligned".into(),
            MemoryAlignmentView::Alignment8 => "8-byte aligned".into(),
        }
    }

    fn on_get_memory_read_mode_string(memory_read_mode: MemoryReadModeView) -> SharedString {
        match memory_read_mode {
            MemoryReadModeView::Skip => "Skip reading".into(),
            MemoryReadModeView::ReadInterleavedWithScan => "Interleaved (fast scans)".into(),
            MemoryReadModeView::ReadBeforeScan => "Prior (fast collection)".into(),
        }
    }

    fn on_get_floating_point_tolerance_string(floating_point_tolerance: FloatingPointToleranceView) -> SharedString {
        match floating_point_tolerance {
            FloatingPointToleranceView::Tolerance10e1 => "0.1".into(),
            FloatingPointToleranceView::Tolerance10e2 => "0.01".into(),
            FloatingPointToleranceView::Tolerance10e3 => "0.001".into(),
            FloatingPointToleranceView::Tolerance10e4 => "0.0001".into(),
            FloatingPointToleranceView::Tolerance10e5 => "0.00001".into(),
            FloatingPointToleranceView::ToleranceEpsilon => "Epsilon".into(),
        }
    }
}
