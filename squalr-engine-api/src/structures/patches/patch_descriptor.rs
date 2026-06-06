use crate::structures::{memory::normalized_region::NormalizedRegion, patches::patch_kind::PatchKind};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PatchDescriptor {
    patch_id: String,
    module_name: String,
    region: NormalizedRegion,
    original_bytes: Vec<u8>,
    patched_bytes: Vec<u8>,
    kind: PatchKind,
    label: Option<String>,
    is_active: bool,
}

impl PatchDescriptor {
    pub fn new(
        patch_id: impl Into<String>,
        module_name: impl Into<String>,
        region: NormalizedRegion,
        original_bytes: Vec<u8>,
        patched_bytes: Vec<u8>,
        kind: PatchKind,
        label: Option<String>,
        is_active: bool,
    ) -> Self {
        Self {
            patch_id: patch_id.into(),
            module_name: module_name.into(),
            region,
            original_bytes,
            patched_bytes,
            kind,
            label,
            is_active,
        }
    }

    pub fn get_patch_id(&self) -> &str {
        &self.patch_id
    }

    pub fn get_module_name(&self) -> &str {
        &self.module_name
    }

    pub fn get_region(&self) -> &NormalizedRegion {
        &self.region
    }

    pub fn get_original_bytes(&self) -> &[u8] {
        &self.original_bytes
    }

    pub fn get_patched_bytes(&self) -> &[u8] {
        &self.patched_bytes
    }

    pub fn get_kind(&self) -> PatchKind {
        self.kind
    }

    pub fn get_label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    pub fn get_is_active(&self) -> bool {
        self.is_active
    }

    pub fn set_is_active(
        &mut self,
        is_active: bool,
    ) {
        self.is_active = is_active;
    }
}
