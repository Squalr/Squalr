use crate::structures::properties::property::Property;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PropertyCollection {
    properties: Vec<Property>,
}

impl PropertyCollection {
    pub fn new(properties: Vec<Property>) -> Self {
        Self { properties }
    }

    pub fn get_properties(&self) -> &Vec<Property> {
        &self.properties
    }

    pub fn combine_property_collections(property_collection: &Vec<PropertyCollection>) -> Vec<Property> {
        vec![]
    }
}
