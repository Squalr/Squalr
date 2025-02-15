use crate::commands::results::list::results_list_request::ResultsListRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum ResultsCommand {
    /// Collect values and scan in the same parallel thread pool.
    List {
        #[structopt(flatten)]
        results_list_request: ResultsListRequest,
    },
}
