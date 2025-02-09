use crate::commands::command_handler::CommandHandler;
use crate::commands::process::handlers::process_command_close::handle_process_command_close;
use crate::commands::process::handlers::process_command_list::handle_process_command_list;
use crate::commands::process::handlers::process_command_open::handle_process_command_open;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
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
        #[structopt(short = "i", long)]
        fetch_icons: bool,
    },
    Close,
}

impl CommandHandler for ProcessCommand {
    fn handle(
        &self,
        uuid: Uuid,
    ) {
        match self {
            ProcessCommand::Open { pid, search_name, match_case } => {
                handle_process_command_open(*pid, search_name, *match_case, uuid);
            }
            ProcessCommand::List {
                require_windowed,
                search_name,
                match_case,
                limit,
                fetch_icons,
            } => {
                handle_process_command_list(*require_windowed, search_name, *match_case, *limit, *fetch_icons, uuid);
            }
            ProcessCommand::Close {} => {
                handle_process_command_close(uuid);
            }
        }
    }
}
