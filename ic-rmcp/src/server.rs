use ic_http_certification::{HeaderField, HttpRequest, HttpResponse};
use std::future::Future;

pub trait Server {
    fn handle(
        &self,
        req: &HttpRequest,
        auth: impl Fn(&[HeaderField]) -> bool,
    ) -> impl Future<Output = HttpResponse>;
}
