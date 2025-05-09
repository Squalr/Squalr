use crate::structures::property_viewer::{property::Property, property_handle::PropertyHandle};
use slab::Slab;

pub struct PropertyRegistry {
    registered_properties: Slab<Property>,
}

impl PropertyRegistry {
    pub fn new() -> Self {
        Self {
            registered_properties: Slab::new(),
        }
    }

    pub fn insert(
        &mut self,
        property: Property,
    ) -> PropertyHandle {
        PropertyHandle::new(self.registered_properties.insert(property))
    }

    pub fn get(
        &self,
        handle: &PropertyHandle,
    ) -> Option<&Property> {
        self.registered_properties.get(handle.get_id())
    }

    pub fn get_mut(
        &mut self,
        handle: &PropertyHandle,
    ) -> Option<&mut Property> {
        self.registered_properties.get_mut(handle.get_id())
    }

    pub fn remove(
        &mut self,
        handle: PropertyHandle,
    ) -> Option<Property> {
        self.registered_properties.try_remove(handle.get_id())
    }
}
