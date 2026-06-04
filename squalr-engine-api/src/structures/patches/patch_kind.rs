use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum PatchKind {
    #[default]
    Code,
    NoOperation,
    SoftwareBreakpoint,
    Generic,
}
