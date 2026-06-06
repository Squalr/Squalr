use crate as api;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug)]
pub(crate) enum CommandLinePatchesCommand {
    Apply {
        #[structopt(flatten)]
        patch_apply_request: CommandLinePatchApplyRequest,
    },
    #[structopt(alias = "nop", alias = "no-op")]
    NoOperation {
        #[structopt(flatten)]
        patch_no_operation_request: CommandLinePatchNoOperationRequest,
    },
    Restore {
        #[structopt(flatten)]
        patch_restore_request: CommandLinePatchRestoreRequest,
    },
    RestoreAddress {
        #[structopt(flatten)]
        patch_restore_address_request: CommandLinePatchRestoreAddressRequest,
    },
    List {
        #[structopt(flatten)]
        patch_list_request: CommandLinePatchListRequest,
    },
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLinePatchApplyRequest {
    #[structopt(short = "a", long, parse(try_from_str = api::conversions::conversions_from_primitives::Conversions::parse_hex_or_int))]
    pub address: u64,
    #[structopt(short = "m", long, default_value = "")]
    pub module_name: String,
    #[structopt(short = "b", long = "byte", parse(try_from_str = parse_patch_byte))]
    pub patched_bytes: Vec<u8>,
    #[structopt(short = "k", long = "kind", default_value = "code")]
    pub kind: api::structures::patches::PatchKind,
    #[structopt(long)]
    pub label: Option<String>,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLinePatchNoOperationRequest {
    #[structopt(short = "a", long, parse(try_from_str = api::conversions::conversions_from_primitives::Conversions::parse_hex_or_int))]
    pub address: u64,
    #[structopt(short = "m", long, default_value = "")]
    pub module_name: String,
    #[structopt(long)]
    pub label: Option<String>,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLinePatchRestoreRequest {
    #[structopt(long = "patch-id")]
    pub patch_id: String,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLinePatchRestoreAddressRequest {
    #[structopt(short = "a", long, parse(try_from_str = api::conversions::conversions_from_primitives::Conversions::parse_hex_or_int))]
    pub address: u64,
    #[structopt(short = "m", long, default_value = "")]
    pub module_name: String,
    #[structopt(long = "expected-kind")]
    pub expected_kind: Option<api::structures::patches::PatchKind>,
}

#[derive(Clone, StructOpt, Debug, Default)]
pub(crate) struct CommandLinePatchListRequest {}

impl From<CommandLinePatchesCommand> for api::commands::patches::patches_command::PatchesCommand {
    fn from(command: CommandLinePatchesCommand) -> Self {
        match command {
            CommandLinePatchesCommand::Apply { patch_apply_request } => Self::Apply {
                patch_apply_request: patch_apply_request.into(),
            },
            CommandLinePatchesCommand::NoOperation { patch_no_operation_request } => Self::NoOperation {
                patch_no_operation_request: patch_no_operation_request.into(),
            },
            CommandLinePatchesCommand::Restore { patch_restore_request } => Self::Restore {
                patch_restore_request: patch_restore_request.into(),
            },
            CommandLinePatchesCommand::RestoreAddress { patch_restore_address_request } => Self::RestoreAddress {
                patch_restore_address_request: patch_restore_address_request.into(),
            },
            CommandLinePatchesCommand::List { patch_list_request } => Self::List {
                patch_list_request: patch_list_request.into(),
            },
        }
    }
}

impl From<CommandLinePatchApplyRequest> for api::commands::patches::apply::patch_apply_request::PatchApplyRequest {
    fn from(request: CommandLinePatchApplyRequest) -> Self {
        Self {
            address: request.address,
            module_name: request.module_name,
            patched_bytes: request.patched_bytes,
            kind: request.kind,
            label: request.label,
        }
    }
}

impl From<CommandLinePatchNoOperationRequest> for api::commands::patches::no_operation::patch_no_operation_request::PatchNoOperationRequest {
    fn from(request: CommandLinePatchNoOperationRequest) -> Self {
        Self {
            address: request.address,
            module_name: request.module_name,
            label: request.label,
        }
    }
}

impl From<CommandLinePatchRestoreRequest> for api::commands::patches::restore::patch_restore_request::PatchRestoreRequest {
    fn from(request: CommandLinePatchRestoreRequest) -> Self {
        Self { patch_id: request.patch_id }
    }
}

impl From<CommandLinePatchRestoreAddressRequest> for api::commands::patches::restore_address::patch_restore_address_request::PatchRestoreAddressRequest {
    fn from(request: CommandLinePatchRestoreAddressRequest) -> Self {
        Self {
            address: request.address,
            module_name: request.module_name,
            expected_kind: request.expected_kind,
        }
    }
}

impl From<CommandLinePatchListRequest> for api::commands::patches::list::patch_list_request::PatchListRequest {
    fn from(_: CommandLinePatchListRequest) -> Self {
        Self
    }
}

fn parse_patch_byte(value: &str) -> Result<u8, String> {
    let parsed_value = api::conversions::conversions_from_primitives::Conversions::parse_hex_or_int(value)
        .map_err(|error| format!("Failed to parse patch byte '{}': {}.", value, error))?;

    u8::try_from(parsed_value).map_err(|_| format!("Patch byte '{}' is outside the 0-255 range.", value))
}
