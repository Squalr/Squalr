use crate::responses::engine_response::EngineResponse;
use crate::responses::engine_response::ExtractArgs;
use crate::responses::engine_response::TypedEngineResponse;
use serde::{Deserialize, Serialize};
use squalr_engine_processes::process_info::{OpenedProcessInfo, ProcessInfo};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProcessResponse {
    List { processes: Vec<ProcessInfo> },
    Close { process_info: OpenedProcessInfo },
    Open { process_info: OpenedProcessInfo },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessListResponse {
    pub processes: Vec<ProcessInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessOpenResponse {
    pub process_info: OpenedProcessInfo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessCloseResponse {
    pub process_info: OpenedProcessInfo,
}

impl TypedEngineResponse for ProcessListResponse {
    fn from_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Process(ProcessResponse::List { processes }) = response {
            Ok(Self { processes })
        } else {
            Err(response)
        }
    }
}

impl TypedEngineResponse for ProcessOpenResponse {
    fn from_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Process(ProcessResponse::Open { process_info }) = response {
            Ok(Self { process_info })
        } else {
            Err(response)
        }
    }
}

impl TypedEngineResponse for ProcessCloseResponse {
    fn from_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Process(ProcessResponse::Close { process_info }) = response {
            Ok(Self { process_info })
        } else {
            Err(response)
        }
    }
}

impl ExtractArgs for ProcessListResponse {
    type Args = Vec<ProcessInfo>;

    fn extract_args(self) -> Self::Args {
        self.processes
    }
}

impl ExtractArgs for ProcessOpenResponse {
    type Args = OpenedProcessInfo;

    fn extract_args(self) -> Self::Args {
        self.process_info
    }
}

impl ExtractArgs for ProcessCloseResponse {
    type Args = OpenedProcessInfo;

    fn extract_args(self) -> Self::Args {
        self.process_info
    }
}
