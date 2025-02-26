use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

pub fn update_config_field<T: Serialize + DeserializeOwned>(
    config: &mut T,
    field: &str,
    value: &str,
) {
    let config_json = match serde_json::to_value(&*config) {
        Ok(json) => json,
        Err(err) => {
            log::error!("Failed to serialize config: {}", err);
            return;
        }
    };

    if let Some(config_map) = config_json.as_object() {
        if let Some(existing_value) = config_map.get(field) {
            let new_value = match existing_value {
                Value::Number(_) => match value.parse::<serde_json::Number>() {
                    Ok(parsed_value) => Value::Number(parsed_value),
                    Err(err) => {
                        log::error!("Failed to parse number: {}", err);
                        return;
                    }
                },
                Value::Bool(_) => match value.parse::<bool>() {
                    Ok(parsed_value) => Value::Bool(parsed_value),
                    Err(err) => {
                        log::error!("Failed to parse boolean: {}", err);
                        return;
                    }
                },
                Value::String(_) => Value::String(value.to_string()),
                _ => {
                    log::error!("Unsupported value type");
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
                Err(err) => {
                    log::error!("Failed to deserialize config: {}", err);
                }
            }
        } else {
            log::error!("Unknown setting name");
        }
    } else {
        log::error!("Failed to convert config to JSON map");
    }
}
