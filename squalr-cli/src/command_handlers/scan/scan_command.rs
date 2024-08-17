use squalr_engine_common::dynamic_struct::data_type::DataType;
use squalr_engine_common::dynamic_struct::data_value::DataValue;
use squalr_engine_scanning::scanners::constraints::scan_constraint_type::ScanConstraintType;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum ScanCommand {
    /// Scan for a specific value
    Manual {
        #[structopt(short = "v", long)]
        value: Option<DataValue>,
        #[structopt(short = "d", long)]
        data_type: DataType,
        #[structopt(short = "c", long)]
        constraint_type: ScanConstraintType,
    },
    Hybrid {
        #[structopt(short = "v", long)]
        value: Option<DataValue>,
        #[structopt(short = "d", long)]
        data_type: DataType,
        #[structopt(short = "c", long)]
        constraint_type: ScanConstraintType,
    },
    /// Collect values for the current scan if one exist, otherwise collect for a new scan
    Collect,
}
