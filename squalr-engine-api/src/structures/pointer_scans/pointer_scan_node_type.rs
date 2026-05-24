use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum PointerScanNodeType {
    #[default]
    Heap,
    Static,
}
