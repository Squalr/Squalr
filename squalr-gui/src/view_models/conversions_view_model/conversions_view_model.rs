use crate::ConversionsViewModelBindings;
use crate::DisplayValueTypeView;
use crate::FloatingPointToleranceView;
use crate::MainWindowView;
use crate::MemoryAlignmentView;
use crate::MemoryReadModeView;
use crate::converters::display_value_type_converter::DisplayValueTypeConverter;
use slint::ComponentHandle;
use slint::SharedString;
use slint_mvvm::convert_from_view_data::ConvertFromViewData;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm_macros::create_view_bindings;
use squalr_engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::conversions::conversions::Conversions;
use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
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
                on_convert_to_data_type(data_value: SharedString, from_display_value_type: DisplayValueTypeView, to_display_value_type: DisplayValueTypeView) -> [] -> Self::on_convert_to_data_type,
                on_get_memory_alignment_string(memory_alignment: MemoryAlignmentView) -> [] -> Self::on_get_memory_alignment_string,
                on_get_memory_read_mode_string(memory_read_mode: MemoryReadModeView) -> [] -> Self::on_get_memory_read_mode_string,
                on_get_floating_point_tolerance_string(floating_point_tolerance: FloatingPointToleranceView) -> [] -> Self::on_get_floating_point_tolerance_string,
            }
        });

        dependency_container.register::<ConversionsViewModel>(view_model);
    }

    fn on_convert_to_data_type(
        data_value: SharedString,
        from_display_value_type: DisplayValueTypeView,
        to_display_value_type: DisplayValueTypeView,
    ) -> SharedString {
        let data_value_string = data_value.to_string();
        let from_display_value_type = DisplayValueTypeConverter {}.convert_from_view_data(&from_display_value_type);
        let to_display_value_type = DisplayValueTypeConverter {}.convert_from_view_data(&to_display_value_type);

        match Conversions::convert_data_value(&data_value_string, from_display_value_type, to_display_value_type) {
            Ok(converted_data_value) => converted_data_value.into(),
            Err(error) => {
                log::error!("Error converting data value for display type {}, error: {}", data_value_string, error);
                data_value
            }
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
