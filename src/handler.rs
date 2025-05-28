use crate::server::Server;
use rmcp::{Error, model::*};
use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
use serde::Serialize;
use serde_json::from_slice;
use std::cmp::Ordering;

type RxJsonRpcMessage = JsonRpcMessage<ClientRequest, ClientResult, ClientNotification>;

impl<S: Service> Server for S {
    async fn handle(&self, req: HttpRequest<'_>) -> HttpResponse {
        if req.method() != "POST" || req.url() != "/mcp" {
            return HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(404).unwrap())
                .with_body(b"Not Found or Method Not Allowed. Use POST to /mcp")
                .build();
        }

        match from_slice::<RxJsonRpcMessage>(req.body()) {
            Ok(message) => match message {
                JsonRpcMessage::Request(request) => {
                    return response(self.handle_request(request).await);
                }
                JsonRpcMessage::Notification(notification) => {
                    self.handle_notification(notification).await;
                    return HttpResponse::builder()
                        .with_status_code(StatusCode::from_u16(202).unwrap())
                        .build();
                }
                JsonRpcMessage::BatchRequest(batch) => {
                    let mut results = Vec::new();
                    for message in batch {
                        match message {
                            JsonRpcBatchRequestItem::Request(r) => {
                                results.push(self.handle_request(r).await)
                            }
                            JsonRpcBatchRequestItem::Notification(n) => {
                                self.handle_notification(n).await
                            }
                        }
                    }

                    return response(results);
                }

                _ => {
                    return HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(200).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "application/json".to_string())])
                .with_body(br#"{"jsonrpc": "2.0", "error": {"code": -32600, "message": "Invalid Request"}, "id": null}"#)
                .build();
                }
            },
            Err(_) => {
                return HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(200).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "application/json".to_string())])
                .with_body(br#"{"jsonrpc": "2.0", "error": {"code": -32700, "message": "Parse error"},"id": null}"#)
                .build();
            }
        }
    }
}

trait Service: Handler {
    async fn handle_request(
        &self,
        request: JsonRpcRequest<ClientRequest>,
    ) -> JsonRpcMessage<Request, ServerResult, Notification>;
    async fn handle_notification(
        &self,
        notification: JsonRpcNotification<ClientNotification>,
    ) -> ();
}

impl<H: Handler> Service for H {
    async fn handle_request(
        &self,
        request: JsonRpcRequest<ClientRequest>,
    ) -> JsonRpcMessage<Request, ServerResult, Notification> {
        let result = match request.request {
            ClientRequest::InitializeRequest(request) => self
                .initialize(request.params)
                .await
                .map(ServerResult::InitializeResult),
            ClientRequest::PingRequest(_request) => self.ping().await.map(ServerResult::empty),
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
    async fn handle_notification(
        &self,
        notification: JsonRpcNotification<ClientNotification>,
    ) -> () {
        match notification.notification {
            ClientNotification::InitializedNotification(_notification) => {
                self.on_initialized().await
            }
            _ => (),
        }
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
    fn ping(&self) -> impl Future<Output = Result<(), Error>> {
        std::future::ready(Ok(()))
    }
    fn initialize(
        &self,
        request: InitializeRequestParam,
    ) -> impl Future<Output = Result<InitializeResult, Error>> {
        let mut info = self.get_info();
        let request_version = request.protocol_version.clone();

        let negotiated_protocol_version = match request_version.partial_cmp(&info.protocol_version)
        {
            Some(Ordering::Less) => request.protocol_version.clone(),
            Some(Ordering::Equal) => request.protocol_version.clone(),
            Some(Ordering::Greater) => info.protocol_version,
            None => {
                return std::future::ready(Err(Error::internal_error(
                    "UnsupportedProtocolVersion",
                    None,
                )));
            }
        };

        info.protocol_version = negotiated_protocol_version;
        std::future::ready(Ok(info))
    }
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
    fn on_initialized(&self) -> impl Future<Output = ()> {
        std::future::ready(())
    }
    fn get_info(&self) -> ServerInfo {
        ServerInfo::default()
    }
}
