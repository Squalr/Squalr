use squalr_engine_api::commands::settings::general::general_settings_response::GeneralSettingsResponse;
use squalr_engine_api::commands::settings::memory::memory_settings_response::MemorySettingsResponse;
use squalr_engine_api::commands::settings::scan::scan_settings_response::ScanSettingsResponse;
use squalr_engine_api::commands::settings::settings_response::SettingsResponse;

pub fn handle_settings_response(cmd: SettingsResponse) {
    match cmd {
        SettingsResponse::General { general_settings_response } => match general_settings_response {
            GeneralSettingsResponse::List {
                general_settings_list_response,
            } => match general_settings_list_response.general_settings {
                Ok(settings) => log::info!("General settings: {:?}", settings),
                Err(error) => log::error!("Failed to list general settings: {}", error),
            },
            GeneralSettingsResponse::Set { .. } => {
                log::info!("General settings updated.");
            }
        },
        SettingsResponse::Memory { memory_settings_response } => match memory_settings_response {
            MemorySettingsResponse::List { memory_settings_list_response } => match memory_settings_list_response.memory_settings {
                Ok(settings) => log::info!("Memory settings: {:?}", settings),
                Err(error) => log::error!("Failed to list memory settings: {}", error),
            },
            MemorySettingsResponse::Set { .. } => {
                log::info!("Memory settings updated.");
            }
        },
        SettingsResponse::Scan { scan_settings_response } => match scan_settings_response {
            ScanSettingsResponse::List { scan_settings_list_response } => match scan_settings_list_response.scan_settings {
                Ok(settings) => log::info!("Scan settings: {:?}", settings),
                Err(error) => log::error!("Failed to list scan settings: {}", error),
            },
            ScanSettingsResponse::Set { .. } => {
                log::info!("Scan settings updated.");
            }
        },
    }
}
