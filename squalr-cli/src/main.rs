mod logging;
mod runtime;

use crate::logging::cli_log_listener::CliLogListener;
use crate::runtime::runtime::Runtime;
use squalr_engine::squalr_engine::SqualrEngine;
use squalr_engine_common::logging::logger::Logger;

fn main() {
    // Hook into engine logging for the cli to display.
    let cli_log_listener = CliLogListener::new();
    Logger::get_instance().subscribe(cli_log_listener);

    SqualrEngine::initialize(false);

    let mut runtime = Runtime::new(std::env::args().collect());

    runtime.run_loop();
    runtime.shutdown();
}
