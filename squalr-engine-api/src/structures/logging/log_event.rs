use log::Level;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEvent {
    pub message: String,
    pub level: Level,
}
