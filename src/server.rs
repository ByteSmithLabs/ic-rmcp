use ic_http_certification::{HttpRequest, HttpResponse};

pub trait Server {
    fn handle(&self, req: HttpRequest
    ) -> impl Future<Output = HttpResponse>;
}