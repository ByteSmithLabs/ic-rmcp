use crate::server::Server;
use ic_cdk::eprintln;
use ic_http_certification::{HeaderField, HttpRequest, HttpResponse, StatusCode};
use rmcp::{model::*, Error};
use serde::Serialize;
use serde_json::{from_slice, from_value, json, to_value, Value};
use std::future::Future;

type RxJsonRpcMessage = JsonRpcMessage<ClientRequest, ClientResult, ClientNotification>;

impl<S: Service> Server for S {
    async fn handle(&self, req: &HttpRequest<'_>) -> HttpResponse {
        if req.method() != "POST" || req.url() != "/mcp" {
            return HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(404).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "text/plain".to_string())])
                .with_body(b"Not Found or Method Not Allowed. Use POST to /mcp")
                .build();
        }

        match from_slice::<Value>(req.body()){
            Ok(Value::Array(req)) => {
                    let mut results = Vec::new();
                    for message in req {
                        match from_value::<JsonRpcBatchRequestItem<ClientRequest, ClientNotification>>(message) {
                            Ok(JsonRpcBatchRequestItem::Request(r)) => {
                                results.push(to_value(self.handle_request(r).await).unwrap_or(json!({"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error"}})))
                            }
                            Ok(JsonRpcBatchRequestItem::Notification(n)) => {
                                self.handle_notification(n).await
                            },
                            Err(e) => {
                                eprintln!("Parse JSON: {}", e);
                                results.push(json!({"jsonrpc":"2.0","error":{"code":-32600,"message":"Invalid Request"},"id":null}))
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
            Ok(Value::Number(_)) | Ok(Value::Bool(_)) | Ok(Value::String(_)) | Ok(Value::Null) => HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(200).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "application/json".to_string())])
                .with_body(br#"{"jsonrpc": "2.0", "error": {"code": -32600, "message": "Invalid Request"}, "id": null}"#)
                .build(),
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
        req: &HttpRequest<'_>,
        auth: impl Fn(&[HeaderField]) -> bool,
    ) -> HttpResponse {
        match auth(req.headers()) {
            true => self.handle(req).await,
            false => HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(401).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "text/plain".to_string())])
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
                    Some(_) => Ok(ServerResult::InitializeResult(info)),
                    _ => Err(Error::internal_error("UnsupportedProtocolVersion", None)),
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
        Err(e) => {
            eprintln!("Serialize response: {}", e);
            builder
            .with_body(
                br#"{"jsonrpc": "2.0", "error": {"code": -32603, "message": "Internal error"}}"#,
            )
            .build()
        }
    }
}

/// A handler holds MCP message execution logic.
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

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;

    #[test]
    fn test_default_handler() {
        use std::borrow::Cow;
        struct H;

        impl Handler for H {}

        assert_eq!(
            block_on(H {}.call_tool(CallToolRequestParam {
                name: Cow::from("foo"),
                arguments: None
            })),
            Err(Error::method_not_found::<CallToolRequestMethod>())
        );

        assert_eq!(
            block_on(H {}.list_tools(None)),
            Ok(ListToolsResult::default())
        );

        assert_eq!(H {}.get_info(), ServerInfo::default());
    }

    #[test]
    fn test_response() {
        use ic_http_certification::{HttpResponse, StatusCode};
        use serde::{Serialize, Serializer};
        use serde_json::json;
        use std::error::Error;
        use std::fmt::{Debug, Display, Formatter};

        #[derive(Debug)]
        struct SerError;

        impl Display for SerError {
            fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
                write!(f, "This is a custom serialization error!")
            }
        }

        impl Error for SerError {}

        struct SerializableThatFails;

        impl Serialize for SerializableThatFails {
            fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                Err(serde::ser::Error::custom(SerError))
            }
        }

        assert_eq!(
            response(json!({"foo":"bar"})),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(200).unwrap())
                .with_headers(vec![(
                    "Content-Type".to_string(),
                    "application/json".to_string(),
                )])
                .with_body(br#"{"foo":"bar"}"#)
                .build()
        );

        assert_eq!(
            response(SerializableThatFails{}),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(200).unwrap())
                .with_headers(vec![(
                    "Content-Type".to_string(),
                    "application/json".to_string(),
                )])
                .with_body(br#"{"jsonrpc": "2.0", "error": {"code": -32603, "message": "Internal error"}}"#)
                .build()
        )
    }

    #[test]
    fn test_service() {
        use std::borrow::Cow;

        struct S;
        impl Handler for S {}

        // no panic
        block_on(S {}.handle_notification(JsonRpcNotification {
            jsonrpc: JsonRpcVersion2_0,
            notification: ClientNotification::InitializedNotification(NotificationNoParam {
                method: InitializedNotificationMethod,
                extensions: Extensions::default(),
            }),
        }));

        match block_on(S {}.handle_request(JsonRpcRequest {
            jsonrpc: JsonRpcVersion2_0,
            id: NumberOrString::Number(1),
            request: ClientRequest::InitializeRequest(Request {
                method: InitializeResultMethod,
                params: InitializeRequestParam {
                    protocol_version: ProtocolVersion::LATEST,
                    capabilities: ClientCapabilities::default(),
                    client_info: Implementation {
                        name: "foo".to_string(),
                        version: "bar".to_string(),
                    },
                },
                extensions: Extensions::new(),
            }),
        })) {
            JsonRpcMessage::Response(res) => {
                assert_eq!(res.jsonrpc, JsonRpcVersion2_0 {});
                assert_eq!(res.id, NumberOrString::Number(1));

                match res.result {
                    ServerResult::InitializeResult(res) => {
                        assert_eq!(InitializeResult::default(), res);
                    }
                    _ => panic!("Expected ServerResult::InitializeResult"),
                }
            }
            _ => panic!("Expected JsonRpcMessage::Response"),
        }

        match block_on(S {}.handle_request(JsonRpcRequest {
            jsonrpc: JsonRpcVersion2_0,
            id: NumberOrString::Number(1),
            request: ClientRequest::PingRequest(RequestNoParam {
                method: PingRequestMethod,
                extensions: Extensions::new(),
            }),
        })) {
            JsonRpcMessage::Response(res) => {
                assert_eq!(res.jsonrpc, JsonRpcVersion2_0 {});
                assert_eq!(res.id, NumberOrString::Number(1));

                match res.result {
                    ServerResult::EmptyResult(res) => {
                        assert_eq!(EmptyObject {}, res);
                    }
                    _ => panic!("Expected ServerResult::EmptyResult"),
                }
            }
            _ => panic!("Expected JsonRpcMessage::Response"),
        }

        match block_on(S {}.handle_request(JsonRpcRequest {
            jsonrpc: JsonRpcVersion2_0,
            id: NumberOrString::Number(1),
            request: ClientRequest::CallToolRequest(Request {
                method: CallToolRequestMethod,
                params: CallToolRequestParam {
                    name: Cow::from("foo"),
                    arguments: None,
                },
                extensions: Extensions::new(),
            }),
        })) {
            JsonRpcMessage::Error(error) => {
                assert_eq!(error.jsonrpc, JsonRpcVersion2_0 {});
                assert_eq!(error.id, NumberOrString::Number(1));
            }
            _ => panic!("Expected JsonRpcMessage::Error"),
        }

        match block_on(S {}.handle_request(JsonRpcRequest {
            jsonrpc: JsonRpcVersion2_0,
            id: NumberOrString::Number(1),
            request: ClientRequest::ListToolsRequest(RequestOptionalParam {
                method: ListToolsRequestMethod,
                params: None,
                extensions: Extensions::new(),
            }),
        })) {
            JsonRpcMessage::Response(res) => {
                assert_eq!(res.jsonrpc, JsonRpcVersion2_0 {});
                assert_eq!(res.id, NumberOrString::Number(1));
            }
            _ => panic!("Expected JsonRpcMessage::Response"),
        }

        match block_on(S {}.handle_request(JsonRpcRequest {
            jsonrpc: JsonRpcVersion2_0,
            id: NumberOrString::Number(1),
            request: ClientRequest::ListResourcesRequest(RequestOptionalParam {
                method: ListResourcesRequestMethod,
                params: None,
                extensions: Extensions::new(),
            }),
        })) {
            JsonRpcMessage::Error(error) => {
                assert_eq!(error.jsonrpc, JsonRpcVersion2_0 {});
                assert_eq!(error.id, NumberOrString::Number(1));

                assert_eq!(
                    error.error,
                    Error::new(ErrorCode::METHOD_NOT_FOUND, "Method not found", None,)
                )
            }
            _ => panic!("Expected JsonRpcMessage::Error"),
        }
    }

    #[test]
    fn test_server_handle() {
        use ic_http_certification::Method;
        struct A;
        impl Handler for A {}

        assert_eq!(
            block_on(A {}.handle(&HttpRequest::builder().with_url("/foo").build())),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(404).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "text/plain".to_string())])
                .with_body(b"Not Found or Method Not Allowed. Use POST to /mcp")
                .build()
        );

        assert_eq!(
            block_on(A {}.handle(&HttpRequest::builder().with_method(Method::GET).build())),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(404).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "text/plain".to_string())])
                .with_body(b"Not Found or Method Not Allowed. Use POST to /mcp")
                .build()
        );

        assert_eq!(
            block_on(A{}.handle(&HttpRequest::builder().with_method(Method::POST).with_url("/mcp").with_body(b"{").build())),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(200).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "application/json".to_string())])
                .with_body(br#"{"jsonrpc": "2.0", "error": {"code": -32700, "message": "Parse error"},"id": null}"#)
                .build());

        assert_eq!(
            block_on(A{}.handle(&HttpRequest::builder().with_method(Method::POST).with_url("/mcp").with_body(b"12").build())),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(200).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "application/json".to_string())])
                .with_body(br#"{"jsonrpc": "2.0", "error": {"code": -32600, "message": "Invalid Request"}, "id": null}"#)
                .build());

        assert_eq!(
            block_on(A{}.handle(&HttpRequest::builder().with_method(Method::POST).with_url("/mcp").with_body(br#""foo""#).build())),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(200).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "application/json".to_string())])
                .with_body(br#"{"jsonrpc": "2.0", "error": {"code": -32600, "message": "Invalid Request"}, "id": null}"#)
                .build());

        assert_eq!(
            block_on(A{}.handle(&HttpRequest::builder().with_method(Method::POST).with_url("/mcp").with_body(br#"null"#).build())),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(200).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "application/json".to_string())])
                .with_body(br#"{"jsonrpc": "2.0", "error": {"code": -32600, "message": "Invalid Request"}, "id": null}"#)
                .build());

        assert_eq!(
            block_on(A{}.handle(&HttpRequest::builder().with_method(Method::POST).with_url("/mcp").with_body(br#"true"#).build())),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(200).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "application/json".to_string())])
                .with_body(br#"{"jsonrpc": "2.0", "error": {"code": -32600, "message": "Invalid Request"}, "id": null}"#)
                .build());

        assert_eq!(
            block_on(A{}
                .handle(&HttpRequest::builder().with_method(Method::POST).with_url("/mcp")
                .with_body(br#"
                    {
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": {
                        "tools": [
                        {
                            "name": "get_weather",
                            "description": "Get current weather information for a location",
                            "inputSchema": {
                            "type": "object",
                            "properties": {
                                "location": {
                                "type": "string",
                                "description": "City name or zip code"
                                }
                            },
                            "required": ["location"]
                            }
                        }
                        ],
                        "nextCursor": "next-page-cursor"
                    }
                    }                
                "#).build())),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(200).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "application/json".to_string())])
                .with_body(br#"{"jsonrpc": "2.0", "error": {"code": -32600, "message": "Invalid Request"}, "id": null}"#)
                .build());

        assert_eq!(
            block_on(
                A {}.handle(
                    &HttpRequest::builder()
                        .with_method(Method::POST)
                        .with_url("/mcp")
                        .with_body(
                            br#"
                    {
                    "jsonrpc": "2.0",
                    "method": "notifications/initialized"
                    }           
                "#
                        )
                        .build()
                )
            ),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(202).unwrap())
                .build()
        );

        assert_eq!(
            block_on(
                A {}.handle(
                    &HttpRequest::builder()
                        .with_method(Method::POST)
                        .with_url("/mcp")
                        .with_body(
                            br#"
                        {
                        "jsonrpc": "2.0",
                        "id": "123",
                        "method": "ping"
                        }        
                "#
                        )
                        .build()
                )
            ),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(200).unwrap())
                .with_headers(vec![(
                    "Content-Type".to_string(),
                    "application/json".to_string()
                )])
                .with_body(br#"{"jsonrpc":"2.0","id":"123","result":{}}"#)
                .build()
        );

        assert_eq!(
            block_on(
                A {}.handle(
                    &HttpRequest::builder()
                        .with_method(Method::POST)
                        .with_url("/mcp")
                        .with_body(
                            br#"[
                        {
                        "jsonrpc": "2.0",
                        "id": "123",
                        "method": "ping"
                        },
                        {
                        "jsonrpc": "2.0",
                        "method": "notifications/initialized"
                        } 
                        ]        
                "#
                        )
                        .build()
                )
            ),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(200).unwrap())
                .with_headers(vec![(
                    "Content-Type".to_string(),
                    "application/json".to_string()
                )])
                .with_body(br#"[{"id":"123","jsonrpc":"2.0","result":{}}]"#)
                .build()
        );

        assert_eq!(
            block_on(
                A {}.handle(
                    &HttpRequest::builder()
                        .with_method(Method::POST)
                        .with_url("/mcp")
                        .with_body(
                            br#"[
                        {
                        "jsonrpc": "2.0",
                        "method": "notifications/initialized"
                        },
                        {
                        "foo": "bar"
                        } 
                        ]        
                "#
                        )
                        .build()
                )
            ),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(200).unwrap())
                .with_headers(vec![(
                    "Content-Type".to_string(),
                    "application/json".to_string()
                )])
                .with_body(br#"[{"error":{"code":-32600,"message":"Invalid Request"},"id":null,"jsonrpc":"2.0"}]"#)
                .build()
        );
    }

    #[test]
    fn test_server_handle_auth() {
        struct A;
        impl Handler for A {}

        assert_eq!(
            block_on(
                A {}.handle_with_auth(&HttpRequest::builder().with_url("/foo").build(), |_| false)
            ),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(401).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "text/plain".to_string())])
                .with_body(b"Unauthorized")
                .build()
        );

        assert_eq!(
            block_on(
                A {}.handle_with_auth(&HttpRequest::builder().with_url("/foo").build(), |_| true)
            ),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(404).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "text/plain".to_string())])
                .with_body(b"Not Found or Method Not Allowed. Use POST to /mcp")
                .build()
        );
    }
}
