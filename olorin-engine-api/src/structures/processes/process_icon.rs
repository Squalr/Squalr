use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessIcon {
    bytes_rgba: Vec<u8>,
    width: u32,
    height: u32,
}

impl ProcessIcon {
    pub fn new(
        bytes_rgba: Vec<u8>,
        width: u32,
        height: u32,
    ) -> Self {
        Self { bytes_rgba, width, height }
    }

    pub fn get_bytes_rgba(&self) -> &Vec<u8> {
        &self.bytes_rgba
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }
}
