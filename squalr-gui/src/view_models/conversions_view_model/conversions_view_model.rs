use crate::ConversionsViewModelBindings;
use crate::DisplayValueTypeView;
use crate::FloatingPointToleranceView;
use crate::MainWindowView;
use crate::MemoryAlignmentView;
use crate::MemoryReadModeView;
use crate::StringEncodingView;
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
                on_get_string_encoding_string(string_encoding: StringEncodingView) -> [] -> Self::on_get_string_encoding_string,
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
            Err(err) => {
                log::error!("Error converting data value for display type {}, error: {}", data_value_string, err);
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

    fn on_get_string_encoding_string(string_encoding: StringEncodingView) -> SharedString {
        match string_encoding {
            StringEncodingView::Utf8 => "UTF-8",
            StringEncodingView::Utf16 => "UTF-16",
            StringEncodingView::Utf16be => "UTF-16BE",
            StringEncodingView::Ascii => "ASCII",
            StringEncodingView::Big5 => "Big5",
            StringEncodingView::EucJp => "EUC-JP",
            StringEncodingView::EucKr => "EUC-KR",
            StringEncodingView::Gb180302022 => "GB18030-2022",
            StringEncodingView::Gbk => "GBK",
            StringEncodingView::Hz => "HZ",
            StringEncodingView::Iso2022Jp => "ISO-2022-JP",
            StringEncodingView::Iso88591 => "ISO-8859-1",
            StringEncodingView::Iso885910 => "ISO-8859-10",
            StringEncodingView::Iso885913 => "ISO-8859-13",
            StringEncodingView::Iso885914 => "ISO-8859-14",
            StringEncodingView::Iso885915 => "ISO-8859-15",
            StringEncodingView::Iso885916 => "ISO-8859-16",
            StringEncodingView::Iso88592 => "ISO-8859-2",
            StringEncodingView::Iso88593 => "ISO-8859-3",
            StringEncodingView::Iso88594 => "ISO-8859-4",
            StringEncodingView::Iso88595 => "ISO-8859-5",
            StringEncodingView::Iso88596 => "ISO-8859-6",
            StringEncodingView::Iso88597 => "ISO-8859-7",
            StringEncodingView::Iso88598 => "ISO-8859-8",
            StringEncodingView::Iso88598i => "ISO-8859-8-i",
            StringEncodingView::Koi8R => "KOI8-R",
            StringEncodingView::Koi8U => "KOI8-U",
            StringEncodingView::Macintosh => "Macintosh",
            StringEncodingView::MacCyrillic => "Mac-Cyrillic",
            StringEncodingView::Replacement => "Replacement",
            StringEncodingView::ShiftJis => "Shift-JIS",
            StringEncodingView::Windows1250 => "Windows-1250",
            StringEncodingView::Windows1251 => "Windows-1251",
            StringEncodingView::Windows1252 => "Windows-1252",
            StringEncodingView::Windows1253 => "Windows-1253",
            StringEncodingView::Windows1254 => "Windows-1254",
            StringEncodingView::Windows1255 => "Windows-1255",
            StringEncodingView::Windows1256 => "Windows-1256",
            StringEncodingView::Windows1257 => "Windows-1257",
            StringEncodingView::Windows1258 => "Windows-1258",
            StringEncodingView::Windows874 => "Windows-874",
            StringEncodingView::XMacCyrillic => "X-Mac-Cyrillic",
            StringEncodingView::XUserDefined => "X-User-Defined",
        }
        .into()
    }
}
