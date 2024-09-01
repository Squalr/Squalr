use crate::command_handlers::settings::settings_command::SettingsCommand;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use squalr_engine_common::logging::{log_level::LogLevel, logger::Logger};
use squalr_engine_memory::memory_settings::MemorySettings;
use squalr_engine_scanning::scan_settings::ScanSettings;

pub fn handle_settings_set(cmd: &SettingsCommand) {
    if let SettingsCommand::Set { setting_command } = cmd {
        // Parse the setting command
        let (domain_and_setting, new_value) = match setting_command.split_once('=') {
            Some(parts) => parts,
            None => {
                Logger::get_instance().log(LogLevel::Error, "Invalid command format. Expected format: domain.setting=value", None);
                return;
            }
        };

        let (domain, setting_name) = match domain_and_setting.split_once('.') {
            Some(parts) => parts,
            None => {
                Logger::get_instance().log(LogLevel::Error, "Invalid setting format. Expected format: domain.setting", None);
                return;
            }
        };

        Logger::get_instance().log(LogLevel::Info, &format!("Setting {}.{}={}", domain, setting_name, new_value), None);

        // Dispatch to the appropriate domain handler
        match domain {
            "memory" => {
                let mut memory_config = MemorySettings::get_instance()
                    .get_full_config()
                    .write()
                    .unwrap();
                update_config_field(&mut *memory_config, setting_name, new_value);
            }
            "scan" => {
                let mut scan_config = ScanSettings::get_instance().get_full_config().write().unwrap();
                update_config_field(&mut *scan_config, setting_name, new_value);
            }
            _ => {
                Logger::get_instance().log(LogLevel::Error, "Unknown domain", None);
                return;
            }
        }
    }
}

fn update_config_field<T: Serialize + DeserializeOwned>(
    config: &mut T,
    field: &str,
    value: &str,
) {
    let config_json = match serde_json::to_value(&*config) {
        Ok(json) => json,
        Err(e) => {
            Logger::get_instance().log(LogLevel::Error, &format!("Failed to serialize config: {}", e), None);
            return;
        }
    };

    if let Some(config_map) = config_json.as_object() {
        if let Some(existing_value) = config_map.get(field) {
            let new_value = match existing_value {
                Value::Number(_) => match value.parse::<serde_json::Number>() {
                    Ok(parsed_value) => Value::Number(parsed_value),
                    Err(_) => {
                        Logger::get_instance().log(LogLevel::Error, "Failed to parse number", None);
                        return;
                    }
                },
                Value::Bool(_) => match value.parse::<bool>() {
                    Ok(parsed_value) => Value::Bool(parsed_value),
                    Err(_) => {
                        Logger::get_instance().log(LogLevel::Error, "Failed to parse boolean", None);
                        return;
                    }
                },
                Value::String(_) => Value::String(value.to_string()),
                _ => {
                    Logger::get_instance().log(LogLevel::Error, "Unsupported value type", None);
                    return;
                }
            };

            let mut updated_json = config_json.clone();
            updated_json
                .as_object_mut()
                .unwrap()
                .insert(field.to_string(), new_value);

            match serde_json::from_value(updated_json) {
                Ok(updated_config) => *config = updated_config,
                Err(e) => {
                    Logger::get_instance().log(LogLevel::Error, &format!("Failed to deserialize config: {}", e), None);
                }
            }
        } else {
            Logger::get_instance().log(LogLevel::Error, "Unknown setting name", None);
        }
    } else {
        Logger::get_instance().log(LogLevel::Error, "Failed to convert config to JSON map", None);
    }
}
