use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum ProcessCommand {
    Open {
        #[structopt(short = "p", long)]
        pid: Option<u32>,
        #[structopt(short = "n", long)]
        search_name: Option<String>,
        #[structopt(short = "m", long)]
        match_case: bool,
    },
    List {
        #[structopt(short = "w", long)]
        require_windowed: bool,
        #[structopt(short = "n", long)]
        search_name: Option<String>,
        #[structopt(short = "m", long)]
        match_case: bool,
        #[structopt(short = "l", long)]
        limit: Option<u64>,
    },
    Close,
}
