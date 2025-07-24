use ic_rmcp::Server;
use std::collections::HashMap;

use ic_http_certification::Method;
use ic_pluto::{
    http::{HeaderField, HttpBody, HttpRequest, HttpResponse},
    router::Router,
};
use serde_json::json;

use crate::{adder_mcp::Adder, API_KEY};

// Custom conversion function from pluto to cert request
fn convert_pluto_to_cert(req: HttpRequest) -> ic_http_certification::HttpRequest<'static> {
    let method = match req.method.as_str() {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        "HEAD" => Method::HEAD,
        "OPTIONS" => Method::OPTIONS,
        "PATCH" => Method::PATCH,
        _ => Method::POST, // Default fallback
    };

    let cert_headers: Vec<(String, String)> = req
        .headers
        .into_iter()
        .map(|header_field| {
            let HeaderField(key, value) = header_field; // Takes ownership
            (key, value)
        })
        .collect();

    ic_http_certification::HttpRequest::builder()
        .with_method(method)
        .with_url(req.url)
        .with_headers(cert_headers)
        .with_body(req.body)
        .build()
}

fn convert_cert_to_pluto(response: ic_http_certification::HttpResponse) -> HttpResponse {
    HttpResponse {
        status_code: response.status_code().as_u16(),
        headers: response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        body: HttpBody::Raw(response.body().to_vec()),
    }
}

pub(crate) fn setup() -> Router {
    let mut router = Router::new();

    router.post("/mcp", false, |req: HttpRequest| async move {
        ic_cdk::println!("Received a POST request on /mcp");
        // Convert ic_pluto::HttpRequest to ic_http_certification::HttpRequest
        let cert_req = convert_pluto_to_cert(req);

        let response = Adder {}
            .handle(&cert_req, |headers| -> bool {
                headers
                    .iter()
                    .any(|(k, v)| k == "x-api-key" && *v == API_KEY.with_borrow(|k| k.clone()))
            })
            .await;

        ic_cdk::println!("Response from Adder MCP: {:#?}", response);
        ic_cdk::println!("API_KEY {}", API_KEY.with_borrow(|k| k.clone()));

        // Convert ic_http_certification::HttpResponse to ic_pluto::HttpResponse
        Ok(convert_cert_to_pluto(response))
    });

    router.get("/ping", false, |_req: HttpRequest| async move {
        ic_cdk::println!("Received a GET request on /ping");
        Ok(HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: json!({
                "statusCode": 200,
                "message": "Hello World from GET",
            })
            .into(),
        })
    });

    router
}
