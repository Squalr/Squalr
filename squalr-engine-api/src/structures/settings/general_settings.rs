use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::fmt;

#[derive(Copy, Clone, Deserialize, Serialize)]
pub struct GeneralSettings {
    pub debug_engine_request_delay_ms: u64,
}

impl fmt::Debug for GeneralSettings {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match to_string_pretty(&self) {
            Ok(json) => write!(formatter, "Settings for scan: {}", json),
            Err(_) => write!(formatter, "Scan config {{ could not serialize to JSON }}"),
        }
    }
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            debug_engine_request_delay_ms: 0,
        }
    }
}
