use crate::Error;
use crate::model::{JsonRpcMessage, ClientNotification, ClientRequest, ClientResult, ServerResult};
use ic_http_certification::{HttpRequest, HttpResponse};

mod server;

pub type RxJsonRpcMessage = JsonRpcMessage<
    ClientRequest,
    ClientResult,
    ClientNotification,
>;

pub trait Service {
    fn handle_request(
        &self,
        request: ClientRequest,
    ) -> impl Future<Output = Result<ServerResult, Error>>;
    fn handle_notification(
        &self,
        notification: ClientNotification,
    ) -> impl Future<Output = Result<(), Error>>;
}

pub trait ServiceExt: Service {
    fn handle(&self, req: HttpRequest
    ) -> impl Future<Output = HttpResponse>;
}