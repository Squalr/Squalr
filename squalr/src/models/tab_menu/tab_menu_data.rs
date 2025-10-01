use smallvec::SmallVec;
use std::{rc::Rc, sync::atomic::AtomicI32};

#[derive(Clone)]
pub struct TabMenuData {
    pub headers: SmallVec<[String; 16]>,
    // Atomic for mutability.
    pub active_tab_index: Rc<AtomicI32>,
}
