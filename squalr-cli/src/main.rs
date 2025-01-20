mod logging;
mod runtime;

use crate::runtime::runtime::Runtime;
use std::io;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Create a runtime, which will either be an interactive cli or an ipc shell controlled by a parent process based on args.
    let mut runtime = Runtime::new(args);

    // Run the cli or ipc loop.
    let result = runtime.run();

    runtime.shutdown();
    result
}
