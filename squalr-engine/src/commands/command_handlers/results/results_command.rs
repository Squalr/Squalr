use serde::{Deserialize, Serialize};
use squalr_engine_common::values::data_type::DataType;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Serialize, Deserialize)]
pub enum ResultsCommand {
    /// Collect values and scan in the same parallel thread pool.
    List {
        #[structopt(short = "p", long)]
        page: u64,

        #[structopt(short = "d", long)]
        data_type: DataType,
    },
}
