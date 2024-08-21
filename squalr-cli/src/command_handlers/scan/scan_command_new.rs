use crate::command_handlers::scan::ScanCommand;
use squalr_engine_common::values::{data_type::DataType, endian::Endian};
use squalr_engine::session_manager::SessionManager;
use squalr_engine_scanning::scanners::parameters::scan_filter_parameters::ScanFilterParameters;

pub fn handle_new_scan_command(
    cmd: &mut ScanCommand,
) {
    if let ScanCommand::New { scan_filter_parameters, scan_all_primitives } = cmd {
        let mut scan_filter_parameters = scan_filter_parameters.clone();

        if *scan_all_primitives {
            scan_filter_parameters = vec![
                ScanFilterParameters::new_with_value(None, DataType::U8()),
                ScanFilterParameters::new_with_value(None, DataType::U16(Endian::Little)),
                ScanFilterParameters::new_with_value(None, DataType::U32(Endian::Little)),
                ScanFilterParameters::new_with_value(None, DataType::U64(Endian::Little)),
                ScanFilterParameters::new_with_value(None, DataType::I8()),
                ScanFilterParameters::new_with_value(None, DataType::I16(Endian::Little)),
                ScanFilterParameters::new_with_value(None, DataType::I32(Endian::Little)),
                ScanFilterParameters::new_with_value(None, DataType::I64(Endian::Little)),
                ScanFilterParameters::new_with_value(None, DataType::F32(Endian::Little)),
                ScanFilterParameters::new_with_value(None, DataType::F64(Endian::Little)),
            ];
        }

        let session_manager_lock = SessionManager::get_instance();
        let process_info = {
            let session_manager = session_manager_lock.read().unwrap();
            session_manager.get_opened_process().cloned()
        };

        if let Some(process_info) = process_info {
            let session_manager = session_manager_lock.write().unwrap();
            let snapshot = session_manager.get_snapshot();
            let mut snapshot = snapshot.write().unwrap();

            snapshot.new_scan(&process_info, scan_filter_parameters);
        }
    }
}
