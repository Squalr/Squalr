use crate::commands::command_handler::CommandHandler;
use crate::commands::scan::handlers::scan_command_hybrid::handle_hybrid_scan_command;
use crate::commands::scan::handlers::scan_command_manual::handle_manual_scan_command;
use crate::commands::scan::handlers::scan_command_new::handle_new_scan_command;
use crate::commands::scan::handlers::scan_command_value_collector::handle_value_collector_command;
use serde::{Deserialize, Serialize};
use squalr_engine_common::values::anonymous_value::AnonymousValue;
use squalr_engine_scanning::scanners::parameters::scan_compare_type::ScanCompareType;
use squalr_engine_scanning::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
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

impl CommandHandler for ScanCommand {
    fn handle(
        &self,
        uuid: Uuid,
    ) {
        match self {
            ScanCommand::Collect {} => {
                handle_value_collector_command(uuid);
            }
            ScanCommand::Hybrid { scan_value, compare_type } => {
                handle_hybrid_scan_command(scan_value, compare_type, uuid);
            }
            ScanCommand::New {
                scan_filter_parameters,
                scan_all_primitives,
            } => {
                handle_new_scan_command(scan_filter_parameters, *scan_all_primitives, uuid);
            }
            ScanCommand::Manual { scan_value, compare_type } => {
                handle_manual_scan_command(scan_value, compare_type, uuid);
            }
        }
    }
}
