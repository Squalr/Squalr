use squalr_engine_common::structures::{data_values::data_value::DataValue, processes::process_info::OpenedProcessInfo};

pub trait IMemoryReader {
    fn read(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        data_value: &mut Box<dyn DataValue>,
    ) -> bool;
    /*
    fn read_struct(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        dynamic_struct: &mut DynamicStruct,
    ) -> bool;*/
    fn read_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &mut [u8],
    ) -> bool;
}
