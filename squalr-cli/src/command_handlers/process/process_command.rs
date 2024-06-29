use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum ProcessCommand {
    /// Open a process
    Open {
        #[structopt(short = "o", long)]
        pid: u32,
    },
    /// List running processes
    List {
        #[structopt(short = "w", long)]
        windowed: bool,
        #[structopt(short = "t", long)]
        search_term: Option<String>,
        #[structopt(short = "m", long)]
        match_case: bool,
        #[structopt(short = "x", long)]
        system_processes: bool,
        #[structopt(short = "l", long)]
        limit: Option<usize>,
    },
    /// Close a process
    Close {
        #[structopt(short = "c", long)]
        pid: u32,
    },
    // Add other process commands here
}
