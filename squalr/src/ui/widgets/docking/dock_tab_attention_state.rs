use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DockTabAttentionKind {
    Warning,
    Danger,
}

#[derive(Clone, Debug)]
pub struct DockTabAttentionState {
    attention_kind: DockTabAttentionKind,
    force_when_visible: bool,
    requested_at: Instant,
}

impl DockTabAttentionState {
    pub fn new(
        attention_kind: DockTabAttentionKind,
        force_when_visible: bool,
    ) -> Self {
        Self {
            attention_kind,
            force_when_visible,
            requested_at: Instant::now(),
        }
    }

    pub fn get_attention_kind(&self) -> DockTabAttentionKind {
        self.attention_kind
    }

    pub fn get_force_when_visible(&self) -> bool {
        self.force_when_visible
    }

    pub fn get_requested_at(&self) -> Instant {
        self.requested_at
    }
}
