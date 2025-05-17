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

    pub fn combine_property_collections(property_collections: &Vec<PropertyCollection>) -> Vec<Property> {
        let mut merged_properties = vec![];

        for property_collection in property_collections {
            for property in property_collection.get_properties() {
                // JIRA: Only push if not already added.
                merged_properties.push(property.clone());
            }
        }

        merged_properties
    }
}
