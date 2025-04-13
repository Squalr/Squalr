use squalr_engine_api::structures::data_values::data_value::DataValue;
use std::{collections::HashMap, sync::RwLock};

pub struct SnapshotScanResultFreezeList {
    frozen_indicies: RwLock<HashMap<u64, DataValue>>,
}

/// Contains all indicies that the user has marked as frozen in the scan results list.
/// Frozen refers to wriiting a specified value to an address repeatedly within a timer, 'freezing' it to the original value.
impl SnapshotScanResultFreezeList {
    pub fn new() -> Self {
        Self {
            frozen_indicies: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_frozen_indicies(&self) -> &RwLock<HashMap<u64, DataValue>> {
        &self.frozen_indicies
    }

    pub fn is_address_frozen(
        &self,
        address: u64,
    ) -> bool {
        if let Ok(frozen_indicies) = self.frozen_indicies.read() {
            frozen_indicies.contains_key(&address)
        } else {
            false
        }
    }

    pub fn get_address_frozen_data_value(
        &self,
        address: u64,
    ) -> Option<DataValue> {
        if let Ok(frozen_indicies) = self.frozen_indicies.read() {
            if let Some(data_value) = frozen_indicies.get(&address) {
                Some(data_value.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn set_address_frozen(
        &self,
        address: u64,
        data_value: DataValue,
    ) {
        if let Ok(mut frozen_indicies) = self.frozen_indicies.write() {
            frozen_indicies.insert(address, data_value);
        }
    }

    pub fn set_address_unfrozen(
        &self,
        address: u64,
    ) {
        if let Ok(mut frozen_indicies) = self.frozen_indicies.write() {
            frozen_indicies.remove(&address);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut frozen_indicies) = self.frozen_indicies.write() {
            frozen_indicies.clear();
        }
    }
}
