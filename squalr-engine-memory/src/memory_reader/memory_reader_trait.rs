use squalr_engine_common::structures::dynamic_struct::dynamic_struct::DynamicStruct;
use squalr_engine_common::structures::processes::process_info::OpenedProcessInfo;
use squalr_engine_common::structures::values::data_value::DataValue;

pub trait IMemoryReader {
    fn read(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        data_value: &mut DataValue,
    ) -> bool;
    fn read_struct(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        dynamic_struct: &mut DynamicStruct,
    ) -> bool;
    fn read_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &mut [u8],
    ) -> bool;
}
