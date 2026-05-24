use squalr_engine_api::structures::{
    data_values::data_value::DataValue, processes::opened_process_info::OpenedProcessInfo, structs::valued_struct::ValuedStruct,
};

pub trait MemoryReaderTrait {
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
        valued_struct: &mut ValuedStruct,
    ) -> bool;
    fn read_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &mut [u8],
    ) -> bool;
}
