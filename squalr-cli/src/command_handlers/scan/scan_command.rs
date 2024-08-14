use squalr_engine_common::dynamic_struct::field_value::FieldValue;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum ScanCommand {
    /// Scan for a specific value
    Value {
        #[structopt(short = "v", long)]
        value: FieldValue,
    },
    /// Collect values for the current scan if one exist, otherwise collect for a new scan
    Collect,
}
