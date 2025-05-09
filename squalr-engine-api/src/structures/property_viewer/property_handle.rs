use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PropertyHandle {
    id: usize,
}

impl PropertyHandle {
    pub fn new(id: usize) -> Self {
        Self { id }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }
}
