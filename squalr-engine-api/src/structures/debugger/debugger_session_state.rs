use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum DebuggerSessionState {
    #[default]
    Detached,
    Attaching,
    Attached,
    Paused,
    Running,
    Exited,
}
