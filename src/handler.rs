use crate::{error::Error, model::*, server::Server};
use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
use serde_json::from_slice;
use serde::Serialize;
use std::cmp::Ordering;

pub type RxJsonRpcMessage = JsonRpcMessage<ClientRequest, ClientResult, ClientNotification>;

impl<H: Handler> Server for H {
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
                    let result = match request {
                        ClientRequest::InitializeRequest(request) => self
                            .initialize(request.params)
                            .await
                            .map(ServerResult::InitializeResult),
                        ClientRequest::PingRequest(_request) => {
                            self.ping().await.map(ServerResult::empty)
                        }
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
                    match notification {
                        ClientNotification::InitializedNotification(_notification) => {
                            self.on_initialized().await
                        }
                        _ => (),
                    };
                    return HttpResponse::builder()
                        .with_status_code(StatusCode::from_u16(202).unwrap())
                        .build();
                }
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
