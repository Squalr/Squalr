use crate::structures::memory::pointer::Pointer;
use std::collections::HashMap;

pub struct FreezeListRegistry {
    frozen_pointers: HashMap<Pointer, Vec<u8>>,
}

/// Contains all indicies that the user has marked as frozen in the scan results list.
/// Frozen refers to wriiting a specified value to an address repeatedly within a timer, 'freezing' it to the original value.
impl FreezeListRegistry {
    pub fn new() -> Self {
        Self {
            frozen_pointers: HashMap::new(),
        }
    }

    pub fn get_frozen_pointers(&self) -> &HashMap<Pointer, Vec<u8>> {
        &self.frozen_pointers
    }

    pub fn is_address_frozen(
        &self,
        pointer: &Pointer,
    ) -> bool {
        self.frozen_pointers.contains_key(pointer)
    }

    pub fn get_address_frozen_bytes(
        &self,
        pointer: &Pointer,
    ) -> Option<&Vec<u8>> {
        if let Some(data_value) = self.frozen_pointers.get(pointer) {
            Some(data_value)
        } else {
            None
        }
    }

    pub fn set_address_frozen(
        &mut self,
        pointer: Pointer,
        data_value: Vec<u8>,
    ) {
        self.frozen_pointers.insert(pointer, data_value);
    }

    pub fn set_address_unfrozen(
        &mut self,
        pointer: &Pointer,
    ) {
        self.frozen_pointers.remove(pointer);
    }

    // JIRA: This function need sto be able to clear by source. We need to be be able to register by source.
    // We need to be able to also freeze complex types like pointers.
    pub fn clear(&mut self) {
        self.frozen_pointers.clear();
    }
}
