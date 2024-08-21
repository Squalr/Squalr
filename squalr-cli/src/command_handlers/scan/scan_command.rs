use squalr_engine_common::dynamic_struct::field_value::FieldValue;
use squalr_engine_scanning::scanners::constraints::{scan_constraint::ScanFilterConstraint, scan_constraint_type::ScanConstraintType};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum ScanCommand {
    /// Collect values for the current scan if one exist, otherwise collect initial values.
    Collect,
    /// Collect values and scan in the same parallel thread pool.
    Hybrid {
        #[structopt(short = "v", long)]
        value_and_type: FieldValue,
        #[structopt(short = "c", long)]
        constraint_type: ScanConstraintType,
    },
    /// Starts a new scan with the provided data types / alignments
    New {
        #[structopt(short = "d", long)]
        filter_constraints: Vec<ScanFilterConstraint>,
    },
    /// Standard scan that operates on existing collected values.
    Manual {
        #[structopt(short = "v", long)]
        value_and_type: FieldValue,
        #[structopt(short = "c", long)]
        constraint_type: ScanConstraintType,
    },
}
