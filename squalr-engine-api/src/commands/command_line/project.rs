use crate as api;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug)]
pub(crate) enum CommandLineProjectCommand {
    List {
        #[structopt(flatten)]
        project_list_request: CommandLineProjectListRequest,
    },
    Open {
        #[structopt(flatten)]
        project_open_request: CommandLineProjectOpenRequest,
    },
    Close {
        #[structopt(flatten)]
        project_close_request: CommandLineProjectCloseRequest,
    },
    Create {
        #[structopt(flatten)]
        project_create_request: CommandLineProjectCreateRequest,
    },
    Delete {
        #[structopt(flatten)]
        project_delete_request: CommandLineProjectDeleteRequest,
    },
    Rename {
        #[structopt(flatten)]
        project_rename_request: CommandLineProjectRenameRequest,
    },
    Save {
        #[structopt(flatten)]
        project_save_request: CommandLineProjectSaveRequest,
    },
    Export {
        #[structopt(flatten)]
        project_export_request: CommandLineProjectExportRequest,
    },
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineProjectListRequest {}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineProjectOpenRequest {
    #[structopt(short = "b", long)]
    pub open_file_browser: bool,
    #[structopt(short = "p", long)]
    pub project_directory_path: Option<PathBuf>,
    #[structopt(short = "n", long)]
    pub project_name: Option<String>,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineProjectCloseRequest {}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineProjectCreateRequest {
    #[structopt(short = "p", long)]
    pub project_directory_path: Option<PathBuf>,
    #[structopt(short = "n", long)]
    pub project_name: Option<String>,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineProjectDeleteRequest {
    #[structopt(short = "p", long)]
    pub project_directory_path: Option<PathBuf>,
    #[structopt(short = "n", long)]
    pub project_name: Option<String>,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineProjectRenameRequest {
    #[structopt(short = "p", long)]
    pub project_directory_path: PathBuf,
    #[structopt(short = "n", long)]
    pub new_project_name: String,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineProjectSaveRequest {}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineProjectExportRequest {
    #[structopt(short = "p", long)]
    pub project_directory_path: Option<PathBuf>,
    #[structopt(short = "n", long)]
    pub project_name: Option<String>,
    #[structopt(short = "o", long)]
    pub open_export_folder: bool,
}

impl From<CommandLineProjectCommand> for api::commands::project::project_command::ProjectCommand {
    fn from(command: CommandLineProjectCommand) -> Self {
        match command {
            CommandLineProjectCommand::List { project_list_request } => Self::List {
                project_list_request: project_list_request.into(),
            },
            CommandLineProjectCommand::Open { project_open_request } => Self::Open {
                project_open_request: project_open_request.into(),
            },
            CommandLineProjectCommand::Close { project_close_request } => Self::Close {
                project_close_request: project_close_request.into(),
            },
            CommandLineProjectCommand::Create { project_create_request } => Self::Create {
                project_create_request: project_create_request.into(),
            },
            CommandLineProjectCommand::Delete { project_delete_request } => Self::Delete {
                project_delete_request: project_delete_request.into(),
            },
            CommandLineProjectCommand::Rename { project_rename_request } => Self::Rename {
                project_rename_request: project_rename_request.into(),
            },
            CommandLineProjectCommand::Save { project_save_request } => Self::Save {
                project_save_request: project_save_request.into(),
            },
            CommandLineProjectCommand::Export { project_export_request } => Self::Export {
                project_export_request: project_export_request.into(),
            },
        }
    }
}

impl From<CommandLineProjectListRequest> for api::commands::project::list::project_list_request::ProjectListRequest {
    fn from(_: CommandLineProjectListRequest) -> Self {
        Self {}
    }
}

impl From<CommandLineProjectOpenRequest> for api::commands::project::open::project_open_request::ProjectOpenRequest {
    fn from(request: CommandLineProjectOpenRequest) -> Self {
        Self {
            open_file_browser: request.open_file_browser,
            project_directory_path: request.project_directory_path,
            project_name: request.project_name,
        }
    }
}

impl From<CommandLineProjectCloseRequest> for api::commands::project::close::project_close_request::ProjectCloseRequest {
    fn from(_: CommandLineProjectCloseRequest) -> Self {
        Self {}
    }
}

impl From<CommandLineProjectCreateRequest> for api::commands::project::create::project_create_request::ProjectCreateRequest {
    fn from(request: CommandLineProjectCreateRequest) -> Self {
        Self {
            project_directory_path: request.project_directory_path,
            project_name: request.project_name,
        }
    }
}

impl From<CommandLineProjectDeleteRequest> for api::commands::project::delete::project_delete_request::ProjectDeleteRequest {
    fn from(request: CommandLineProjectDeleteRequest) -> Self {
        Self {
            project_directory_path: request.project_directory_path,
            project_name: request.project_name,
        }
    }
}

impl From<CommandLineProjectRenameRequest> for api::commands::project::rename::project_rename_request::ProjectRenameRequest {
    fn from(request: CommandLineProjectRenameRequest) -> Self {
        Self {
            project_directory_path: request.project_directory_path,
            new_project_name: request.new_project_name,
        }
    }
}

impl From<CommandLineProjectSaveRequest> for api::commands::project::save::project_save_request::ProjectSaveRequest {
    fn from(_: CommandLineProjectSaveRequest) -> Self {
        Self {}
    }
}

impl From<CommandLineProjectExportRequest> for api::commands::project::export::project_export_request::ProjectExportRequest {
    fn from(request: CommandLineProjectExportRequest) -> Self {
        Self {
            project_directory_path: request.project_directory_path,
            project_name: request.project_name,
            open_export_folder: request.open_export_folder,
        }
    }
}
