use crate as api;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug)]
pub(crate) enum CommandLineProcessCommand {
    Open {
        #[structopt(flatten)]
        process_open_request: CommandLineProcessOpenRequest,
    },
    List {
        #[structopt(flatten)]
        process_list_request: CommandLineProcessListRequest,
    },
    Close {
        #[structopt(flatten)]
        process_close_request: CommandLineProcessCloseRequest,
    },
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineProcessOpenRequest {
    #[structopt(short = "p", long)]
    pub process_id: Option<u32>,
    #[structopt(short = "n", long)]
    pub search_name: Option<String>,
    #[structopt(short = "m", long)]
    pub match_case: bool,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineProcessListRequest {
    #[structopt(short = "w", long)]
    pub require_windowed: bool,
    #[structopt(short = "n", long)]
    pub search_name: Option<String>,
    #[structopt(short = "m", long)]
    pub match_case: bool,
    #[structopt(short = "l", long)]
    pub limit: Option<u64>,
    #[structopt(short = "i", long)]
    pub fetch_icons: bool,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineProcessCloseRequest {}

impl From<CommandLineProcessCommand> for api::commands::process::process_command::ProcessCommand {
    fn from(command: CommandLineProcessCommand) -> Self {
        match command {
            CommandLineProcessCommand::Open { process_open_request } => Self::Open {
                process_open_request: process_open_request.into(),
            },
            CommandLineProcessCommand::List { process_list_request } => Self::List {
                process_list_request: process_list_request.into(),
            },
            CommandLineProcessCommand::Close { process_close_request } => Self::Close {
                process_close_request: process_close_request.into(),
            },
        }
    }
}

impl From<CommandLineProcessOpenRequest> for api::commands::process::open::process_open_request::ProcessOpenRequest {
    fn from(request: CommandLineProcessOpenRequest) -> Self {
        Self {
            process_id: request.process_id,
            search_name: request.search_name,
            match_case: request.match_case,
        }
    }
}

impl From<CommandLineProcessListRequest> for api::commands::process::list::process_list_request::ProcessListRequest {
    fn from(request: CommandLineProcessListRequest) -> Self {
        Self {
            require_windowed: request.require_windowed,
            search_name: request.search_name,
            match_case: request.match_case,
            limit: request.limit,
            fetch_icons: request.fetch_icons,
        }
    }
}

impl From<CommandLineProcessCloseRequest> for api::commands::process::close::process_close_request::ProcessCloseRequest {
    fn from(_: CommandLineProcessCloseRequest) -> Self {
        Self {}
    }
}
