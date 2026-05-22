use crate as api;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug)]
pub(crate) enum CommandLineTrackableTasksCommand {
    List {
        #[structopt(flatten)]
        trackable_tasks_list_request: CommandLineTrackableTasksListRequest,
    },
    Cancel {
        #[structopt(flatten)]
        trackable_tasks_cancel_request: CommandLineTrackableTasksCancelRequest,
    },
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineTrackableTasksListRequest {}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineTrackableTasksCancelRequest {
    #[structopt(short = "t", long)]
    pub task_id: String,
}

impl From<CommandLineTrackableTasksCommand> for api::commands::trackable_tasks::trackable_tasks_command::TrackableTasksCommand {
    fn from(command: CommandLineTrackableTasksCommand) -> Self {
        match command {
            CommandLineTrackableTasksCommand::List { trackable_tasks_list_request } => Self::List {
                trackable_tasks_list_request: trackable_tasks_list_request.into(),
            },
            CommandLineTrackableTasksCommand::Cancel {
                trackable_tasks_cancel_request,
            } => Self::Cancel {
                trackable_tasks_cancel_request: trackable_tasks_cancel_request.into(),
            },
        }
    }
}

impl From<CommandLineTrackableTasksListRequest> for api::commands::trackable_tasks::list::trackable_tasks_list_request::TrackableTasksListRequest {
    fn from(_: CommandLineTrackableTasksListRequest) -> Self {
        Self {}
    }
}

impl From<CommandLineTrackableTasksCancelRequest> for api::commands::trackable_tasks::cancel::trackable_tasks_cancel_request::TrackableTasksCancelRequest {
    fn from(request: CommandLineTrackableTasksCancelRequest) -> Self {
        Self { task_id: request.task_id }
    }
}
