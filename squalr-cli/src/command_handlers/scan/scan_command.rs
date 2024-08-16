use squalr_engine_common::dynamic_struct::field_value::FieldValue;
use squalr_engine_scanning::scanners::constraints::scan_constraint_type::ScanConstraintType;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum ScanCommand {
    /// Scan for a specific value
    Value {
        #[structopt(short = "v", long)]
        value: Option<FieldValue>,
        #[structopt(short = "c", long)]
        constraint_type: ScanConstraintType,
        #[structopt(short = "d", long)]
        delta_value: Option<FieldValue>,
    },
    /// Collect values for the current scan if one exist, otherwise collect for a new scan
    Collect,
}
