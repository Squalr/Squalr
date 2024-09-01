use crate::logging::log_level::LogLevel;
use crate::logging::logger::Logger;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

pub fn update_config_field<T: Serialize + DeserializeOwned>(
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
