use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::anonymous_value_string::AnonymousValueString;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, Debug, Default, StructOpt, Serialize, Deserialize, PartialEq)]
pub struct PointerScanTargetRequest {
    #[structopt(long = "target-address")]
    pub target_address: Option<AnonymousValueString>,
    #[structopt(long = "target-value")]
    pub target_value: Option<AnonymousValueString>,
    #[structopt(long = "target-data-type")]
    pub target_data_type_ref: Option<DataTypeRef>,
}

impl PointerScanTargetRequest {
    pub fn address(target_address: AnonymousValueString) -> Self {
        Self {
            target_address: Some(target_address),
            target_value: None,
            target_data_type_ref: None,
        }
    }

    pub fn value(
        target_value: AnonymousValueString,
        target_data_type_ref: DataTypeRef,
    ) -> Self {
        Self {
            target_address: None,
            target_value: Some(target_value),
            target_data_type_ref: Some(target_data_type_ref),
        }
    }
}
