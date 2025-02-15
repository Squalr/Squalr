use crate::commands::results::list::results_list_response::ResultsListResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ResultsResponse {
    List { results_list_response: ResultsListResponse },
}
