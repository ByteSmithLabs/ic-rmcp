use ic_cdk::{post_upgrade, query, update};
use ic_http_certification::StatusCode;
use ic_pluto::{
    http::{HttpServe, RawHttpRequest, RawHttpResponse},
    http_serve,
    router::Router,
};
use std::cell::RefCell;

use crate::controller;

thread_local! {
    static ROUTER: RefCell<Router>  = RefCell::new(controller::setup());
}

// System functions
#[post_upgrade]
fn post_upgrade() {
    ROUTER.with(|r| *r.borrow_mut() = controller::setup())
}

#[query]
fn http_request(_: ic_http_certification::HttpRequest) -> ic_http_certification::HttpResponse {
    ic_http_certification::HttpResponse::builder()
        .with_status_code(StatusCode::OK)
        .with_upgrade(true)
        .build()
}

#[update]
async fn http_request_update(req: RawHttpRequest) -> RawHttpResponse {
    bootstrap(http_serve!(), req).await
}

async fn bootstrap(mut app: HttpServe, req: RawHttpRequest) -> RawHttpResponse {
    let router = ROUTER.with(|r| r.borrow().clone());
    app.set_router(router);
    app.serve(req).await
}
