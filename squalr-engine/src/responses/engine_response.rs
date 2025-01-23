use crate::responses::process::process_response::ProcessResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineResponse {
    Process(ProcessResponse),
}
