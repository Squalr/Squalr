use squalr_engine_common::values::anonymous_value::AnonymousValue;
use squalr_engine_scanning::scanners::constraints::scan_constraint_type::ScanConstraintType;
use squalr_engine_scanning::scanners::constraints::scan_filter_constraint::ScanFilterConstraint;
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
        constraint_type: ScanConstraintType,
    },
    /// Starts a new scan with the provided data types / alignments
    New {
        #[structopt(short = "d", long, use_delimiter = true)]
        filter_constraints: Vec<ScanFilterConstraint>,
    },
    /// Standard scan that operates on existing collected values.
    Manual {
        #[structopt(short = "v", long)]
        scan_value: Option<AnonymousValue>,
        #[structopt(short = "c", long)]
        constraint_type: ScanConstraintType,
    },
}
