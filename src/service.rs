use crate::Error;
use crate::model::{ClientNotification, ClientRequest, ServerInfo, ServerResult};

pub trait Service {
    fn handle_request(
        &self,
        request: ClientRequest,
    ) -> impl Future<Output = Result<ServerResult, Error>>;
    fn handle_notification(
        &self,
        notification: ClientNotification,
    ) -> impl Future<Output = Result<(), Error>>;
    fn get_info(&self) -> ServerInfo;
}
