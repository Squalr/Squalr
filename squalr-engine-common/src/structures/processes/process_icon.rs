use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessIcon {
    pub bytes_rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}
