use crate::interprocess_ingress::{ExecutableRequest, InterprocessIngress};
use crate::shell::inter_process_unprivileged_host::InterProcessUnprivilegedHost;
use crate::typed_response::TypedResponse;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io;
use std::sync::Arc;

pub trait InterprocessRequest<RequestType: ExecutableRequest<ResponseType> + DeserializeOwned + Serialize, ResponseType: DeserializeOwned + Serialize + 'static>:
    Clone + Serialize + DeserializeOwned
{
    type ResponseType;

    fn execute(&self) -> Self::ResponseType;

    fn to_interprocess_command(&self) -> InterprocessIngress<RequestType, ResponseType>;

    fn send<F>(
        &self,
        host: Arc<InterProcessUnprivilegedHost<RequestType, ResponseType>>,
        callback: F,
    ) -> io::Result<()>
    where
        F: Fn(Self::ResponseType) + Clone + Send + Sync + 'static,
        Self::ResponseType: TypedResponse<ResponseType>,
    {
        let command = self.clone().to_interprocess_command();

        host.dispatch_command(command, move |interprocess_response| {
            if let Ok(response) = Self::ResponseType::from_response(interprocess_response) {
                callback(response);
            }
        })
    }
}
