use crate::commands::engine_response::EngineResponse;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::results::results_response::ResultsResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResultsListResponse {}

impl TypedEngineResponse for ResultsListResponse {
    fn to_engine_response(&self) -> EngineResponse {
        EngineResponse::Results(ResultsResponse::List {
            results_list_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Results(ResultsResponse::List { results_list_response }) = response {
            Ok(results_list_response)
        } else {
            Err(response)
        }
    }
}
