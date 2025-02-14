use crate::commands::engine_command::EngineCommand;

pub trait RequestSender {
    type ResponseType: Send;

    fn send<F>(
        &self,
        callback: F,
    ) where
        F: FnOnce(Self::ResponseType) + Send + Sync + 'static;

    fn to_command(&self) -> EngineCommand;
}
