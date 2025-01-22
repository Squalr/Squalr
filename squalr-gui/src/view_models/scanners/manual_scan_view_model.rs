use crate::DataTypeView;
use crate::MainWindowView;
use crate::ManualScanViewModelBindings;
use crate::ScanConstraintTypeView;
use crate::ValueCollectorViewModelBindings;
use slint::ComponentHandle;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm_macros::create_view_bindings;
use squalr_engine::squalr_engine::SqualrEngine;
use squalr_engine_common::values::data_type::DataType;
use squalr_engine_common::values::endian::Endian;
use squalr_engine_scanning::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use squalr_engine_scanning::scanners::value_collector::ValueCollector;
use std::thread;

pub struct ManualScanViewModel {
    _view_binding: ViewBinding<MainWindowView>,
}

impl ManualScanViewModel {
    pub fn new(view_binding: ViewBinding<MainWindowView>) -> Self {
        let view: ManualScanViewModel = ManualScanViewModel {
            _view_binding: view_binding.clone(),
        };

        create_view_bindings!(view_binding, {
            ManualScanViewModelBindings => {
                on_new_scan() -> [view_binding] -> Self::on_new_scan,
                on_start_scan() -> [view_binding] -> Self::on_start_scan,
                on_set_scan_constraint(scan_constraint: ScanConstraintTypeView) -> [view_binding] -> Self::on_set_scan_constraint,
            },
            ValueCollectorViewModelBindings => {
                on_collect_values() -> [] -> Self::on_collect_values,
            },
        });

        view
    }

    fn on_new_scan(view_binding: ViewBinding<MainWindowView>) {
        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            // TODO: Push this into a converter perhaps, although gets tricky with args
            let scan_filter_parameters = vec![match main_window_view
                .global::<ManualScanViewModelBindings>()
                .get_active_data_type()
            {
                DataTypeView::I8 => ScanFilterParameters::new_with_value(None, DataType::I8()),
                DataTypeView::U8 => ScanFilterParameters::new_with_value(None, DataType::U8()),
                DataTypeView::I16 => ScanFilterParameters::new_with_value(None, DataType::I16(Endian::Little)),
                DataTypeView::I16be => ScanFilterParameters::new_with_value(None, DataType::I16(Endian::Big)),
                DataTypeView::U16 => ScanFilterParameters::new_with_value(None, DataType::U16(Endian::Little)),
                DataTypeView::U16be => ScanFilterParameters::new_with_value(None, DataType::U16(Endian::Big)),
                DataTypeView::I32 => ScanFilterParameters::new_with_value(None, DataType::I32(Endian::Little)),
                DataTypeView::I32be => ScanFilterParameters::new_with_value(None, DataType::I32(Endian::Big)),
                DataTypeView::U32 => ScanFilterParameters::new_with_value(None, DataType::U32(Endian::Little)),
                DataTypeView::U32be => ScanFilterParameters::new_with_value(None, DataType::U32(Endian::Big)),
                DataTypeView::I64 => ScanFilterParameters::new_with_value(None, DataType::I64(Endian::Little)),
                DataTypeView::I64be => ScanFilterParameters::new_with_value(None, DataType::I64(Endian::Big)),
                DataTypeView::U64 => ScanFilterParameters::new_with_value(None, DataType::U64(Endian::Little)),
                DataTypeView::U64be => ScanFilterParameters::new_with_value(None, DataType::U64(Endian::Big)),
                DataTypeView::F32 => ScanFilterParameters::new_with_value(None, DataType::F32(Endian::Little)),
                DataTypeView::F32be => ScanFilterParameters::new_with_value(None, DataType::F32(Endian::Big)),
                DataTypeView::F64 => ScanFilterParameters::new_with_value(None, DataType::F64(Endian::Little)),
                DataTypeView::F64be => ScanFilterParameters::new_with_value(None, DataType::F64(Endian::Big)),
                DataTypeView::Aob => ScanFilterParameters::new_with_value(None, DataType::Bytes(0)), // TODO
                DataTypeView::Str => ScanFilterParameters::new_with_value(None, DataType::Bytes(0)), // TODO
            }];

            thread::spawn(move || {
                if let Some(process_info) = SqualrEngine::get_opened_process() {
                    let snapshot = SqualrEngine::get_snapshot();
                    let mut snapshot = snapshot.write().unwrap();

                    snapshot.new_scan(&process_info, scan_filter_parameters);
                }
            });
        });
    }

    fn on_start_scan(view_binding: ViewBinding<MainWindowView>) {
        //
    }

    fn on_collect_values() {
        if let Some(process_info) = SqualrEngine::get_opened_process() {
            let snapshot = SqualrEngine::get_snapshot();
            let _task = ValueCollector::collect_values(process_info.clone(), snapshot, None, true);
        }
    }

    fn on_set_scan_constraint(
        view_binding: ViewBinding<MainWindowView>,
        scan_constraint: ScanConstraintTypeView,
    ) {
        //
    }
}
