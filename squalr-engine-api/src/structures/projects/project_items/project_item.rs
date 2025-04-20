use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectItem {
    name: String,
    description: String,

    #[serde(skip_serializing)]
    is_activated: bool,
}
