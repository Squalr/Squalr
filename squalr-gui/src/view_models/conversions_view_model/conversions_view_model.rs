use crate::ConversionsViewModelBindings;
use crate::MainWindowView;
use slint::ComponentHandle;
use slint::SharedString;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm_macros::create_view_bindings;
use squalr_engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_common::conversions::Conversions;
use std::sync::Arc;

pub struct ConversionsViewModel {
    _view_binding: ViewBinding<MainWindowView>,
    _engine_execution_context: Arc<EngineExecutionContext>,
}

impl ConversionsViewModel {
    pub fn new(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) -> Arc<Self> {
        let view = Arc::new(ConversionsViewModel {
            _view_binding: view_binding.clone(),
            _engine_execution_context: engine_execution_context.clone(),
        });

        create_view_bindings!(view_binding, {
            ConversionsViewModelBindings => {
                on_convert_hex_to_dec(data_value: SharedString) -> [] -> Self::on_convert_hex_to_dec,
                on_convert_dec_to_hex(data_value: SharedString) -> [] -> Self::on_convert_dec_to_hex,
            }
        });

        view
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
}
