use crate as api;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug)]
pub(crate) enum CommandLineRegistryCommand {
    GetMetadata {
        #[structopt(flatten)]
        registry_get_metadata_request: CommandLineRegistryGetMetadataRequest,
    },
}

#[derive(Clone, Default, StructOpt, Debug)]
pub(crate) struct CommandLineRegistryGetMetadataRequest {}

impl From<CommandLineRegistryCommand> for api::commands::registry::registry_command::RegistryCommand {
    fn from(command: CommandLineRegistryCommand) -> Self {
        match command {
            CommandLineRegistryCommand::GetMetadata { registry_get_metadata_request } => Self::GetMetadata {
                registry_get_metadata_request: registry_get_metadata_request.into(),
            },
        }
    }
}

impl From<CommandLineRegistryGetMetadataRequest> for api::commands::registry::get_metadata::registry_get_metadata_request::RegistryGetMetadataRequest {
    fn from(_: CommandLineRegistryGetMetadataRequest) -> Self {
        Self {}
    }
}
