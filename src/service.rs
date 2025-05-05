use crate::Error;
use crate::model::{ClientNotification, ClientRequest, ServerResult};
use ic_http_certification::{HttpRequest, HttpResponse};
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