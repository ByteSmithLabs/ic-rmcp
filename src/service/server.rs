use super::RxJsonRpcMessage;
use crate::model::{
    JsonRpcMessage, JsonRpcNotification, JsonRpcRequest, Notification, Request, ServerResult,
};
use crate::service::{Service, ServiceExt};
use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
use serde::Serialize;
use serde_json::from_slice;

impl<H: Service> ServiceExt for H {
    async fn handle(&self, req: HttpRequest<'_>) -> HttpResponse {
        if req.method() != "POST" || req.url() != "/mcp" {
            return HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(404).unwrap())
                .with_body(b"Not Found or Method Not Allowed. Use POST to /mcp")
                .build();
        }

        match from_slice::<RxJsonRpcMessage>(req.body()) {
            Ok(message) => match message {
                JsonRpcMessage::Request(JsonRpcRequest { id, request, .. }) => {
                    match self.handle_request(request).await {
                        Ok(result) => {
                            let data: JsonRpcMessage<Request, ServerResult, Notification> =
                                JsonRpcMessage::response(result, id);
                            return response(data);
                        }
                        Err(error) => {
                            let data: JsonRpcMessage<Request, ServerResult, Notification> =
                                JsonRpcMessage::error(error, id);
                            return response(data);
                        }
                    }
                }
                JsonRpcMessage::Notification(JsonRpcNotification { notification, .. }) => {
                    _ = self.handle_notification(notification).await;
                    return HttpResponse::builder()
                        .with_status_code(StatusCode::from_u16(202).unwrap())
                        .build();
                },
                _ => {
                    return HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(400).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "application/json".to_string())])
                .with_body(br#"{"jsonrpc": "2.0", "error": {"code": -32600, "message": "Invalid Request"}}"#)
                .build();
                }
            },
            Err(_) => {
                return HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(400).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "application/json".to_string())])
                .with_body(br#"{"jsonrpc": "2.0", "error": {"code": -32600, "message": "Invalid Request"}}"#)
                .build();
            }
        }
    }
}

fn response<T: Serialize>(data: T) -> HttpResponse<'static> {
    match serde_json::to_string(&data) {
        Ok(body) => HttpResponse::builder()
            .with_status_code(StatusCode::from_u16(400).unwrap())
            .with_headers(vec![(
                "Content-Type".to_string(),
                "application/json".to_string(),
            )])
            .with_body(body.into_bytes())
            .build(),
        Err(_) => HttpResponse::builder()
            .with_status_code(StatusCode::from_u16(500).unwrap())
            .with_headers(vec![("Content-Type".to_string(), "text/plain".to_string())])
            .with_body("Internal Server Error".as_bytes())
            .build(),
    }
}
