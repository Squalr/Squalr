use squalr_engine_common::values::anonymous_value::AnonymousValue;
use squalr_engine_scanning::scanners::parameters::scan_compare_type::ScanCompareType;
use squalr_engine_scanning::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum ScanCommand {
    /// Collect values for the current scan if one exist, otherwise collect initial values.
    Collect,
    /// Collect values and scan in the same parallel thread pool.
    Hybrid {
        #[structopt(short = "v", long)]
        scan_value: Option<AnonymousValue>,
        #[structopt(short = "c", long)]
        compare_type: ScanCompareType,
    },
    /// Starts a new scan with the provided data types / alignments
    New {
        #[structopt(short = "d", long, use_delimiter = true)]
        scan_filter_parameters: Vec<ScanFilterParameters>,
        #[structopt(short = "a", long)]
        scan_all_primitives: bool,
    },
    /// Standard scan that operates on existing collected values.
    Manual {
        #[structopt(short = "v", long)]
        scan_value: Option<AnonymousValue>,
        #[structopt(short = "c", long)]
        compare_type: ScanCompareType,
    },
}
