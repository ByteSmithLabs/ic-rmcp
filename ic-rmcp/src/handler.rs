use crate::server::Server;
use ic_http_certification::{HeaderField, HttpRequest, HttpResponse, StatusCode};
use rmcp::{model::*, Error};
use serde::Serialize;
use serde_json::{from_slice, from_value, json, to_value, Value};
use std::future::Future;

type RxJsonRpcMessage = JsonRpcMessage<ClientRequest, ClientResult, ClientNotification>;

impl<S: Service> Server for S {
    async fn handle(&self, req: HttpRequest<'_>) -> HttpResponse {
        if req.method() != "POST" || req.url() != "/mcp" {
            return HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(404).unwrap())
                .with_body(b"Not Found or Method Not Allowed. Use POST to /mcp")
                .build();
        }

        match from_slice::<Value>(req.body()){
            Ok(Value::Array(req)) => {
                    let mut results = Vec::new();
                    for message in req {
                        match from_value::<JsonRpcBatchRequestItem<ClientRequest, ClientNotification>>(message) {
                            Ok(JsonRpcBatchRequestItem::Request(r)) => {
                                results.push(to_value(self.handle_request(r).await).unwrap_or(json!({"jsonrpc": "2.0", "error": {"code": -32603, "message": "Internal error"}})))
                            }
                            Ok(JsonRpcBatchRequestItem::Notification(n)) => {
                                self.handle_notification(n).await
                            },
                            Err(_) => {
                                results.push(json!({"jsonrpc": "2.0", "error": {"code": -32600, "message": "Invalid Request"}, "id": null}))
                            }
                        };
                    }

                    response(results)
            },
            Ok(Value::Object(req)) => {
                match from_value::<RxJsonRpcMessage>(Value::Object(req)) {
                    Ok(JsonRpcMessage::Request(request)) => response(self.handle_request(request).await),
                    Ok(JsonRpcMessage::Notification(notification)) => {
                            self.handle_notification(notification).await;
                            HttpResponse::builder()
                                            .with_status_code(StatusCode::from_u16(202).unwrap())
                                            .build()
                        }
                    _ => {
                        HttpResponse::builder()
                                .with_status_code(StatusCode::from_u16(200).unwrap())
                                .with_headers(vec![("Content-Type".to_string(), "application/json".to_string())])
                                .with_body(br#"{"jsonrpc": "2.0", "error": {"code": -32600, "message": "Invalid Request"}, "id": null}"#)
                                .build()
                        }
                }
            },
            _ => {
                 HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(200).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "application/json".to_string())])
                .with_body(br#"{"jsonrpc": "2.0", "error": {"code": -32700, "message": "Parse error"},"id": null}"#)
                .build()
            },

    }
    }

    async fn handle_with_auth(
        &self,
        req: HttpRequest<'_>,
        auth: impl Fn(&[HeaderField]) -> bool,
    ) -> HttpResponse {
        match auth(req.headers()) {
            true => self.handle(req).await,
            false => HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(401).unwrap())
                .with_body(br#"Unauthorized"#)
                .build(),
        }
    }
}

trait Service: Handler {
    async fn handle_request(
        &self,
        request: JsonRpcRequest<ClientRequest>,
    ) -> JsonRpcMessage<Request, ServerResult, Notification>;
    async fn handle_notification(&self, notification: JsonRpcNotification<ClientNotification>);
}

impl<H: Handler> Service for H {
    async fn handle_request(
        &self,
        request: JsonRpcRequest<ClientRequest>,
    ) -> JsonRpcMessage<Request, ServerResult, Notification> {
        let result = match request.request {
            ClientRequest::InitializeRequest(request) => {
                let info = self.get_info();
                match request
                    .params
                    .protocol_version
                    .partial_cmp(&info.protocol_version)
                {
                    None => Err(Error::internal_error("UnsupportedProtocolVersion", None)),
                    _ => Ok(ServerResult::InitializeResult(info)),
                }
            }
            ClientRequest::PingRequest(_) => Ok(ServerResult::empty(())),
            ClientRequest::CallToolRequest(request) => self
                .call_tool(request.params)
                .await
                .map(ServerResult::CallToolResult),
            ClientRequest::ListToolsRequest(request) => self
                .list_tools(request.params)
                .await
                .map(ServerResult::ListToolsResult),
            _ => Err(Error::new(
                ErrorCode::METHOD_NOT_FOUND,
                "Method not found",
                None,
            )),
        };

        match result {
            Ok(result) => JsonRpcMessage::response(result, request.id),
            Err(error) => JsonRpcMessage::error(error, request.id),
        }
    }
    async fn handle_notification(&self, notification: JsonRpcNotification<ClientNotification>) {
        if let ClientNotification::InitializedNotification(_) = notification.notification {}
    }
}

fn response<T: Serialize>(data: T) -> HttpResponse<'static> {
    let builder = HttpResponse::builder()
        .with_status_code(StatusCode::from_u16(200).unwrap())
        .with_headers(vec![(
            "Content-Type".to_string(),
            "application/json".to_string(),
        )]);
    match serde_json::to_string(&data) {
        Ok(body) => builder.with_body(body.into_bytes()).build(),
        Err(_) => builder
            .with_body(
                br#"{"jsonrpc": "2.0", "error": {"code": -32603, "message": "Internal error"}}"#,
            )
            .build(),
    }
}

#[allow(unused_variables)]
pub trait Handler {
    fn call_tool(
        &self,
        request: CallToolRequestParam,
    ) -> impl Future<Output = Result<CallToolResult, Error>> {
        std::future::ready(Err(Error::method_not_found::<CallToolRequestMethod>()))
    }
    fn list_tools(
        &self,
        request: Option<PaginatedRequestParam>,
    ) -> impl Future<Output = Result<ListToolsResult, Error>> {
        std::future::ready(Ok(ListToolsResult::default()))
    }
    fn get_info(&self) -> ServerInfo {
        ServerInfo::default()
    }
}
