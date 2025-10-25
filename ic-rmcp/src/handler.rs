use crate::server::Server;
use crate::state::fetch_jwks;
use crate::{model::*, Error};
use ic_cdk::eprintln;
use ic_http_certification::{HeaderField, HttpRequest, HttpResponse, StatusCode};
use serde::Serialize;
use serde_json::{from_slice, from_str, from_value, json, to_value, Value};
use std::cmp::Ordering;
use std::future::Future;
use url::Url;

pub mod oauth;
use oauth::{validate_token, OAuthConfig};

/// Request-scoped context passed to handler methods.
///
/// When OAuth is enabled via [`Server::handle_with_oauth`](crate::Server::handle_with_oauth),
/// [`Context::subject`] is populated with the `sub` claim from the validated access token.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Context {
    pub subject: Option<String>,
}

type RxJsonRpcMessage = JsonRpcMessage<ClientRequest, ClientResult, ClientNotification>;

impl<S: Service> Server for S {
    async fn handle(
        &self,
        req: &HttpRequest<'_>,
        auth: impl Fn(&[HeaderField]) -> bool,
    ) -> HttpResponse<'_> {
        match auth(req.headers()) {
            true => self.raw_handle(None, req).await,
            false => HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(401).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "text/plain".to_string())])
                .with_body(br#"Unauthorized"#)
                .build(),
        }
    }

    async fn handle_with_oauth(&self, req: &HttpRequest<'_>, cfg: OAuthConfig) -> HttpResponse<'_> {
        let metadata_path = match Url::parse(&cfg.metadata_url) {
            Ok(url) => url.path().to_string(),
            Err(err) => {
                eprintln!("Parse metadata url: {}", err);
                return HttpResponse::builder()
                    .with_status_code(StatusCode::from_u16(500).unwrap())
                    .with_headers(vec![("Content-Type".to_string(), "text/plain".to_string())])
                    .with_body(br#"Internal Error"#)
                    .build();
            }
        };

        if req.method() == "GET" && req.url() == metadata_path {
            #[derive(Serialize)]
            struct Metadata<'a> {
                resource: &'a str,
                authorization_servers: &'a [&'a str],
                scopes_supported: &'a [&'a str],
            }

            return response(Metadata {
                resource: &cfg.resource,
                authorization_servers: cfg
                    .issuer_configs
                    .authorization_server
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<&str>>()
                    .as_slice(),
                scopes_supported: cfg
                    .scopes_supported
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<&str>>()
                    .as_slice(),
            });
        }

        let token = match req
            .headers()
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case("Authorization"))
            .and_then(|(_, value)| value.strip_prefix("Bearer "))
        {
            Some(token) => token,
            None => {
                return HttpResponse::builder()
                    .with_status_code(StatusCode::from_u16(401).unwrap())
                    .with_headers(vec![
                        ("Content-Type".to_string(), "text/plain".to_string()),
                        (
                            "WWW-Authenticate".to_string(),
                            format!("Bearer resource_metadata=\"{}\"", cfg.metadata_url),
                        ),
                    ])
                    .with_body(br#"Unauthorized"#)
                    .build()
            }
        };

        let jwk_set = match fetch_jwks(&cfg.issuer_configs.jwks_url).await {
            Ok(set) => set,
            Err(err) => {
                eprintln!("fetch jwk set: {}", err);
                return HttpResponse::builder()
                    .with_status_code(StatusCode::from_u16(500).unwrap())
                    .with_headers(vec![("Content-Type".to_string(), "text/plain".to_string())])
                    .with_body(br#"Internal Error"#)
                    .build();
            }
        };

        match validate_token(token, &cfg.issuer_configs, jwk_set) {
            Ok(claims) => self.raw_handle(Some(claims.sub), req).await,
            Err(_err) => HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(401).unwrap())
                .with_headers(vec![
                    ("Content-Type".to_string(), "text/plain".to_string()),
                    (
                        "WWW-Authenticate".to_string(),
                        format!("Bearer resource_metadata=\"{}\"", cfg.metadata_url),
                    ),
                ])
                .with_body(br#"Token invalid"#)
                .build(),
        }
    }
}

trait Service: Handler {
    async fn raw_handle(&self, subject: Option<String>, req: &HttpRequest<'_>) -> HttpResponse<'_> {
        if req.method() != "POST" || !req.url().ends_with("/mcp") {
            return HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(404).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "text/plain".to_string())])
                .with_body(b"Not Found or Method Not Allowed. Use POST to /mcp")
                .build();
        }

        let version = match req
          .headers()
          .iter()
          .find(|(key, _)| key.eq_ignore_ascii_case("MCP-Protocol-Version"))
          .map(|(_, value)| value.trim()) {
              Some(version) => match from_str::<ProtocolVersion>(&format!("\"{version}\"")){
                  Ok(version) => Some(version),
                  Err(_) => return HttpResponse::builder()
                              .with_status_code(StatusCode::from_u16(200).unwrap())
                              .with_headers(vec![("Content-Type".to_string(), "application/json".to_string())])
                              .with_body(br#"{"jsonrpc": "2.0", "error": {"code": -32600, "message": "Invalid Request"}, "id": null}"#)
                              .build()
              },
              None => None,
          };

        match from_slice::<Value>(req.body()){
            Ok(Value::Array(req)) => {
                    if version.is_some_and(|ver| ver.partial_cmp(&protocol_version_2025_06_18()) != Some(Ordering::Less)) {
                        HttpResponse::builder()
                              .with_status_code(StatusCode::from_u16(200).unwrap())
                              .with_headers(vec![("Content-Type".to_string(), "application/json".to_string())])
                              .with_body(br#"{"jsonrpc": "2.0", "error": {"code": -32600, "message": "Invalid Request"}, "id": null}"#)
                              .build()
                    } else {
                        let mut results = Vec::new();
                    for message in req {
                        match from_value::<JsonRpcBatchRequestItem<ClientRequest, ClientNotification>>(message) {
                            Ok(JsonRpcBatchRequestItem::Request(r)) => {
                                results.push(to_value(self.handle_request(subject.clone(),r).await).unwrap_or(json!({"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error"}})))
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
                    }
            },
            Ok(Value::Object(req)) => {
                match from_value::<RxJsonRpcMessage>(Value::Object(req)) {
                    Ok(JsonRpcMessage::Request(request)) => response(self.handle_request(subject, request).await),
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
    async fn handle_request(
        &self,
        subject: Option<String>,
        request: JsonRpcRequest<ClientRequest>,
    ) -> JsonRpcMessage<Request, ServerResult, Notification>;
    async fn handle_notification(&self, notification: JsonRpcNotification<ClientNotification>);
}

impl<H: Handler> Service for H {
    async fn handle_request(
        &self,
        subject: Option<String>,
        request: JsonRpcRequest<ClientRequest>,
    ) -> JsonRpcMessage<Request, ServerResult, Notification> {
        let result = match request.request {
            ClientRequest::InitializeRequest(request) => {
                let mut info = self.get_info(Context { subject });
                info.protocol_version = protocol_version_2025_06_18();

                if let Some(Ordering::Equal) = request
                    .params
                    .protocol_version
                    .partial_cmp(&info.protocol_version)
                {
                    Ok(ServerResult::InitializeResult(info))
                } else if let Some(Ordering::Equal) = request
                    .params
                    .protocol_version
                    .partial_cmp(&ProtocolVersion::V_2025_03_26)
                {
                    info.protocol_version = ProtocolVersion::V_2025_03_26;
                    Ok(ServerResult::InitializeResult(info))
                } else {
                    Ok(ServerResult::InitializeResult(info))
                }
            }
            ClientRequest::PingRequest(_) => Ok(ServerResult::empty(())),
            ClientRequest::CallToolRequest(request) => self
                .call_tool(Context { subject }, request.params)
                .await
                .map(ServerResult::CallToolResult),
            ClientRequest::ListToolsRequest(request) => self
                .list_tools(Context { subject }, request.params)
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

fn protocol_version_2025_06_18() -> ProtocolVersion {
    from_str::<ProtocolVersion>("\"2025-06-18\"").unwrap()
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

/// Define your server's MCP behavior by implementing this trait.
///
/// You may override any combination of methods; defaults are provided for convenience.
/// - [`Handler::get_info`] should describe your server and enabled capabilities
/// - [`Handler::list_tools`] should return the tools your server exposes
/// - [`Handler::call_tool`] should execute a requested tool and return its result
#[allow(unused_variables)]
pub trait Handler {
    /// Handle a `tools/call` request.
    ///
    /// Default: returns `method_not_found`.
    fn call_tool(
        &self,
        context: Context,
        request: CallToolRequestParam,
    ) -> impl Future<Output = Result<CallToolResult, Error>> {
        std::future::ready(Err(Error::method_not_found::<CallToolRequestMethod>()))
    }
    /// Handle a `tools/list` request.
    ///
    /// Default: returns an empty tool list.
    fn list_tools(
        &self,
        context: Context,
        request: Option<PaginatedRequestParam>,
    ) -> impl Future<Output = Result<ListToolsResult, Error>> {
        std::future::ready(Ok(ListToolsResult::default()))
    }
    /// Provide server metadata and advertised capabilities.
    ///
    /// Default: returns [`ServerInfo::default`]. You typically want to set
    /// `capabilities` (e.g., enable tools) and identify your implementation.
    fn get_info(&self, context: Context) -> ServerInfo {
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
            block_on(H {}.call_tool(
                Context::default(),
                CallToolRequestParam {
                    name: Cow::from("foo"),
                    arguments: None
                }
            )),
            Err(Error::method_not_found::<CallToolRequestMethod>())
        );

        assert_eq!(
            block_on(H {}.list_tools(Context::default(), None)),
            Ok(ListToolsResult::default())
        );

        assert_eq!(H {}.get_info(Context::default()), ServerInfo::default());
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

        match block_on(S {}.handle_request(
            None,
            JsonRpcRequest {
                jsonrpc: JsonRpcVersion2_0,
                id: NumberOrString::Number(1),
                request: ClientRequest::InitializeRequest(Request {
                    method: InitializeResultMethod,
                    params: InitializeRequestParam {
                        protocol_version: ProtocolVersion::V_2025_03_26,
                        capabilities: ClientCapabilities::default(),
                        client_info: Implementation {
                            name: "foo".to_string(),
                            version: "bar".to_string(),
                        },
                    },
                    extensions: Extensions::new(),
                }),
            },
        )) {
            JsonRpcMessage::Response(res) => {
                assert_eq!(res.jsonrpc, JsonRpcVersion2_0 {});
                assert_eq!(res.id, NumberOrString::Number(1));

                match res.result {
                    ServerResult::InitializeResult(res) => {
                        assert_eq!(
                            ServerInfo {
                                protocol_version: ProtocolVersion::V_2025_03_26,
                                capabilities: ServerCapabilities::default(),
                                server_info: Implementation::from_build_env(),
                                instructions: None,
                            },
                            res
                        );
                    }
                    _ => panic!("Expected ServerResult::InitializeResult"),
                }
            }
            _ => panic!("Expected JsonRpcMessage::Response"),
        }

        match block_on(S {}.handle_request(
            None,
            JsonRpcRequest {
                jsonrpc: JsonRpcVersion2_0,
                id: NumberOrString::Number(1),
                request: ClientRequest::InitializeRequest(Request {
                    method: InitializeResultMethod,
                    params: InitializeRequestParam {
                        protocol_version: protocol_version_2025_06_18(),
                        capabilities: ClientCapabilities::default(),
                        client_info: Implementation {
                            name: "foo".to_string(),
                            version: "bar".to_string(),
                        },
                    },
                    extensions: Extensions::new(),
                }),
            },
        )) {
            JsonRpcMessage::Response(res) => {
                assert_eq!(res.jsonrpc, JsonRpcVersion2_0 {});
                assert_eq!(res.id, NumberOrString::Number(1));

                match res.result {
                    ServerResult::InitializeResult(res) => {
                        assert_eq!(
                            ServerInfo {
                                protocol_version: protocol_version_2025_06_18(),
                                capabilities: ServerCapabilities::default(),
                                server_info: Implementation::from_build_env(),
                                instructions: None,
                            },
                            res
                        );
                    }
                    _ => panic!("Expected ServerResult::InitializeResult"),
                }
            }
            _ => panic!("Expected JsonRpcMessage::Response"),
        }

        match block_on(S {}.handle_request(
            None,
            JsonRpcRequest {
                jsonrpc: JsonRpcVersion2_0,
                id: NumberOrString::Number(1),
                request: ClientRequest::InitializeRequest(Request {
                    method: InitializeResultMethod,
                    params: InitializeRequestParam {
                        protocol_version: from_str::<ProtocolVersion>("\"1970-01-01\"").unwrap(),
                        capabilities: ClientCapabilities::default(),
                        client_info: Implementation {
                            name: "foo".to_string(),
                            version: "bar".to_string(),
                        },
                    },
                    extensions: Extensions::new(),
                }),
            },
        )) {
            JsonRpcMessage::Response(res) => {
                assert_eq!(res.jsonrpc, JsonRpcVersion2_0 {});
                assert_eq!(res.id, NumberOrString::Number(1));

                match res.result {
                    ServerResult::InitializeResult(res) => {
                        assert_eq!(
                            ServerInfo {
                                protocol_version: protocol_version_2025_06_18(),
                                capabilities: ServerCapabilities::default(),
                                server_info: Implementation::from_build_env(),
                                instructions: None,
                            },
                            res
                        );
                    }
                    _ => panic!("Expected ServerResult::InitializeResult"),
                }
            }
            _ => panic!("Expected JsonRpcMessage::Response"),
        }

        match block_on(S {}.handle_request(
            None,
            JsonRpcRequest {
                jsonrpc: JsonRpcVersion2_0,
                id: NumberOrString::Number(1),
                request: ClientRequest::PingRequest(RequestNoParam {
                    method: PingRequestMethod,
                    extensions: Extensions::new(),
                }),
            },
        )) {
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

        match block_on(S {}.handle_request(
            None,
            JsonRpcRequest {
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
            },
        )) {
            JsonRpcMessage::Error(error) => {
                assert_eq!(error.jsonrpc, JsonRpcVersion2_0 {});
                assert_eq!(error.id, NumberOrString::Number(1));
            }
            _ => panic!("Expected JsonRpcMessage::Error"),
        }

        match block_on(S {}.handle_request(
            None,
            JsonRpcRequest {
                jsonrpc: JsonRpcVersion2_0,
                id: NumberOrString::Number(1),
                request: ClientRequest::ListToolsRequest(RequestOptionalParam {
                    method: ListToolsRequestMethod,
                    params: None,
                    extensions: Extensions::new(),
                }),
            },
        )) {
            JsonRpcMessage::Response(res) => {
                assert_eq!(res.jsonrpc, JsonRpcVersion2_0 {});
                assert_eq!(res.id, NumberOrString::Number(1));
            }
            _ => panic!("Expected JsonRpcMessage::Response"),
        }

        match block_on(S {}.handle_request(
            None,
            JsonRpcRequest {
                jsonrpc: JsonRpcVersion2_0,
                id: NumberOrString::Number(1),
                request: ClientRequest::ListResourcesRequest(RequestOptionalParam {
                    method: ListResourcesRequestMethod,
                    params: None,
                    extensions: Extensions::new(),
                }),
            },
        )) {
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
    fn test_service_raw_handle() {
        use ic_http_certification::Method;
        struct A;
        impl Handler for A {}

        assert_eq!(
            block_on(A {}.raw_handle(None, &HttpRequest::builder().with_url("/foo").build())),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(404).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "text/plain".to_string())])
                .with_body(b"Not Found or Method Not Allowed. Use POST to /mcp")
                .build()
        );

        assert_eq!(
            block_on(A {}.raw_handle(
                None,
                &HttpRequest::builder().with_method(Method::GET).build()
            )),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(404).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "text/plain".to_string())])
                .with_body(b"Not Found or Method Not Allowed. Use POST to /mcp")
                .build()
        );

        assert_eq!(
            block_on(A{}.raw_handle(None, &HttpRequest::builder().with_method(Method::POST).with_url("/foo/mcp").with_body(b"{").build())),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(200).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "application/json".to_string())])
                .with_body(br#"{"jsonrpc": "2.0", "error": {"code": -32700, "message": "Parse error"},"id": null}"#)
                .build());

        assert_eq!(
            block_on(A{}.raw_handle(None, &HttpRequest::builder().with_method(Method::POST).with_url("/mcp").with_body(b"12").build())),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(200).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "application/json".to_string())])
                .with_body(br#"{"jsonrpc": "2.0", "error": {"code": -32600, "message": "Invalid Request"}, "id": null}"#)
                .build());

        assert_eq!(
            block_on(A{}.raw_handle(None, &HttpRequest::builder().with_method(Method::POST).with_url("/mcp").with_body(br#""foo""#).build())),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(200).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "application/json".to_string())])
                .with_body(br#"{"jsonrpc": "2.0", "error": {"code": -32600, "message": "Invalid Request"}, "id": null}"#)
                .build());

        assert_eq!(
            block_on(A{}.raw_handle(None, &HttpRequest::builder().with_method(Method::POST).with_url("/mcp").with_body(br#"null"#).build())),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(200).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "application/json".to_string())])
                .with_body(br#"{"jsonrpc": "2.0", "error": {"code": -32600, "message": "Invalid Request"}, "id": null}"#)
                .build());

        assert_eq!(
            block_on(A{}.raw_handle(None, &HttpRequest::builder().with_method(Method::POST).with_url("/mcp").with_body(br#"true"#).build())),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(200).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "application/json".to_string())])
                .with_body(br#"{"jsonrpc": "2.0", "error": {"code": -32600, "message": "Invalid Request"}, "id": null}"#)
                .build());

        assert_eq!(
            block_on(A{}
                .raw_handle(None, &HttpRequest::builder().with_method(Method::POST).with_url("/mcp")
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
                A {}.raw_handle(
                    None,
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
                A {}.raw_handle(
                    None,
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
                A {}.raw_handle(
                    None,
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
                A {}.raw_handle(
                    None,
                    &HttpRequest::builder()
                        .with_method(Method::POST)
                        .with_headers(vec![(
                            "MCP-Protocol-Version".to_string(),
                            "2025-03-26".to_string()
                        )])
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
                A {}.raw_handle(None, &HttpRequest::builder()
                        .with_method(Method::POST)
                        .with_headers(vec![("MCP-Protocol-Version".to_string(), "2025-06-18".to_string())])
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
                .with_body(br#"{"jsonrpc": "2.0", "error": {"code": -32600, "message": "Invalid Request"}, "id": null}"#)
                .build()
        );

        assert_eq!(
            block_on(
                A {}.raw_handle(None, &HttpRequest::builder()
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
    fn test_server_handle() {
        struct A;
        impl Handler for A {}

        assert_eq!(
            block_on(A {}.handle(&HttpRequest::builder().with_url("/foo").build(), |_| false)),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(401).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "text/plain".to_string())])
                .with_body(b"Unauthorized")
                .build()
        );

        assert_eq!(
            block_on(A {}.handle(&HttpRequest::builder().with_url("/foo").build(), |_| true)),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(404).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "text/plain".to_string())])
                .with_body(b"Not Found or Method Not Allowed. Use POST to /mcp")
                .build()
        );
    }

    #[test]
    fn test_server_handle_with_oauth() {
        use crate::IssuerConfig;
        use ic_http_certification::Method;

        struct A;
        impl Handler for A {}

        assert_eq!(
            block_on(A {}.handle_with_oauth(
                &HttpRequest::builder().with_url("/foo").build(),
                OAuthConfig {
                    metadata_url: "foo".to_string(),
                    ..Default::default()
                }
            )),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(500).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "text/plain".to_string())])
                .with_body(br#"Internal Error"#)
                .build()
        );

        assert_eq!(
            block_on(
                A {}.handle_with_oauth(
                    &HttpRequest::builder()
                        .with_method(Method::GET)
                        .with_url("/.well-known/oauth-protected-resource")
                        .build(),
                    OAuthConfig {
                        metadata_url: "https://my-server.com/.well-known/oauth-protected-resource"
                            .to_string(),
                        resource: "https://my-server.com".to_string(),
                        scopes_supported: vec![],
                        issuer_configs: IssuerConfig {
                            authorization_server: vec!["https://authorization-server.com".to_string()],
                            ..Default::default()
                        },
                    }
                )
            ),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(200).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "application/json".to_string())])
                .with_body(br#"{"resource":"https://my-server.com","authorization_servers":["https://authorization-server.com"],"scopes_supported":[]}"#)
                .build()
        );

        assert_eq!(
            block_on(
                A {}.handle_with_oauth(
                    &HttpRequest::builder()
                        .with_method(Method::POST)
                        .with_url("/mcp")
                        .build(),
                    OAuthConfig {
                        metadata_url: "https://my-server.com/.well-known/oauth-protected-resource"
                            .to_string(),
                        ..Default::default()
                    }
                )
            ),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(401).unwrap())
                .with_headers(vec![
                    ("Content-Type".to_string(), "text/plain".to_string()),
                     ("WWW-Authenticate".to_string(), "Bearer resource_metadata=\"https://my-server.com/.well-known/oauth-protected-resource\"".to_string())
                    ])
                .with_body(br#"Unauthorized"#)
                .build()
        );
    }
}
