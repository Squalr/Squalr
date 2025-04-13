use dashmap::DashMap;
use squalr_engine_api::structures::data_values::data_value::DataValue;

pub struct SnapshotScanResultFreezeList {
    frozen_indicies: DashMap<u64, DataValue>,
}

/// Contains all indicies that the user has marked as frozen in the scan results list.
/// Frozen refers to wriiting a specified value to an address repeatedly within a timer, 'freezing' it to the original value.
impl SnapshotScanResultFreezeList {
    pub fn new() -> Self {
        Self {
            frozen_indicies: DashMap::new(),
        }
    }

    pub fn get_frozen_indicies(&self) -> &DashMap<u64, DataValue> {
        &self.frozen_indicies
    }

    pub fn is_address_frozen(
        &self,
        address: u64,
    ) -> bool {
        self.frozen_indicies.contains_key(&address)
    }

    pub fn get_address_frozen_data_value(
        &self,
        address: u64,
    ) -> Option<DataValue> {
        if let Some(data_value) = self.frozen_indicies.get(&address) {
            Some(data_value.clone())
        } else {
            None
        }
    }

    pub fn set_address_frozen(
        &self,
        address: u64,
        data_value: DataValue,
    ) {
        self.frozen_indicies.insert(address, data_value);
    }

    pub fn set_address_unfrozen(
        &self,
        address: u64,
    ) {
        self.frozen_indicies.remove(&address);
    }

    pub fn clear(&self) {
        self.frozen_indicies.clear();
    }
}
