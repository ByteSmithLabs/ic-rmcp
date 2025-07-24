use futures::executor::block_on;
use ic_http_certification::{HttpRequest, HttpResponse, Method, StatusCode};
use ic_rmcp::*;
use rmcp::handler::server::tool::schema_for_type;
use rmcp::{model::*, Error};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{from_value, Value};

struct MagicSum;

#[derive(Deserialize, JsonSchema)]
struct MagicSumRequest {
    a: f64,
    b: f64,
}

impl Handler for MagicSum {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "Magic sum calculator".to_string(),
                version: "1.0.0".to_string(),
            },
            instructions: Some(String::from("This server provides a `calculate_magic_sum` tool returning a magic sum between two numbers")),
            ..Default::default()
        }
    }

    async fn list_tools(&self, _: Option<PaginatedRequestParam>) -> Result<ListToolsResult, Error> {
        Ok(ListToolsResult {
            next_cursor: None,
            tools: vec![Tool::new(
                "calculate_magic_sum",
                "Calculate a magic sum between two numbers",
                schema_for_type::<MagicSumRequest>(),
            )],
        })
    }

    async fn call_tool(&self, request: CallToolRequestParam) -> Result<CallToolResult, Error> {
        match request.name.as_ref() {
            "calculate_magic_sum" => match request.arguments {
                None => Err(Error::invalid_params("invalid arguments to tool add", None)),
                Some(data) => match from_value::<MagicSumRequest>(Value::Object(data)) {
                    Err(_) => Err(Error::invalid_params("invalid arguments to tool add", None)),
                    Ok(args) => Ok(CallToolResult::success(
                        Content::text(format!("{:.2}", args.a + 3.0 * args.b)).into_contents(),
                    )),
                },
            },
            _ => Err(Error::invalid_params("not found tool", None)),
        }
    }
}

#[test]
fn test_auth() {
    assert_eq!(
        block_on(MagicSum {}.handle(&HttpRequest::builder().build(), |_| false)),
        HttpResponse::builder()
            .with_status_code(StatusCode::from_u16(401).unwrap())
            .with_headers(vec![("Content-Type".to_string(), "text/plain".to_string())])
            .with_body(br#"Unauthorized"#)
            .build()
    );

    assert_eq!(
        block_on(MagicSum {}.handle(&HttpRequest::builder().build(), |_| true)),
        HttpResponse::builder()
            .with_status_code(StatusCode::from_u16(404).unwrap())
            .with_headers(vec![("Content-Type".to_string(), "text/plain".to_string())])
            .with_body(b"Not Found or Method Not Allowed. Use POST to /mcp")
            .build()
    );
}

#[test]
fn test_initialization() {
    assert_eq!(
            block_on(
                MagicSum {}.handle(
                    &HttpRequest::builder()
                        .with_method(Method::POST)
                        .with_url("/mcp")
                        .with_body(
                            br#"
                            {
                            "jsonrpc": "2.0",
                            "id": 1,
                            "method": "initialize",
                            "params": {
                                "protocolVersion": "2025-03-26",
                                "capabilities": {},
                                "clientInfo": {
                                "name": "ExampleClient",
                                "version": "1.0.0"
                                }
                            }
                            }      
                "#
                        )
                        .build(), |_| true
                )
            ),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(200).unwrap())
                .with_headers(vec![(
                    "Content-Type".to_string(),
                    "application/json".to_string()
                )])
                .with_body(br#"{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-03-26","capabilities":{"tools":{}},"serverInfo":{"name":"Magic sum calculator","version":"1.0.0"},"instructions":"This server provides a `calculate_magic_sum` tool returning a magic sum between two numbers"}}"#)
                .build()
        );

    assert_eq!(
        block_on(
            MagicSum {}.handle(
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
                    .build(),
                |_| true
            )
        ),
        HttpResponse::builder()
            .with_status_code(StatusCode::from_u16(202).unwrap())
            .build()
    );
}

#[test]
fn test_tools() {
    assert_eq!(
            block_on(
                MagicSum {}.handle(
                    &HttpRequest::builder()
                        .with_method(Method::POST)
                        .with_url("/mcp")
                        .with_body(
                            br#"
                            {
                            "jsonrpc": "2.0",
                            "id": 1,
                            "method": "tools/list",
                            "params": {}
                            }    
                "#
                        )
                        .build(), |_| true
                )
            ),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(200).unwrap())
                .with_headers(vec![(
                    "Content-Type".to_string(),
                    "application/json".to_string()
                )])
                .with_body(br#"{"jsonrpc":"2.0","id":1,"result":{"tools":[{"name":"calculate_magic_sum","description":"Calculate a magic sum between two numbers","inputSchema":{"$schema":"http://json-schema.org/draft-07/schema#","properties":{"a":{"format":"double","type":"number"},"b":{"format":"double","type":"number"}},"required":["a","b"],"title":"MagicSumRequest","type":"object"}}]}}"#)
                .build()
        );

    assert_eq!(
            block_on(
                MagicSum {}.handle(
                    &HttpRequest::builder()
                        .with_method(Method::POST)
                        .with_url("/mcp")
                        .with_body(
                            br#"
                            {
                            "jsonrpc": "2.0",
                            "id": 1,
                            "method": "tools/call",
                            "params": {
                                "name": "calculate_magic_sum",
                                "arguments": {
                                        "a": 4, "b": 6
                                    }  
                                }
                            }    
                "#
                        )
                        .build(), |_| true
                )
            ),
            HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(200).unwrap())
                .with_headers(vec![(
                    "Content-Type".to_string(),
                    "application/json".to_string()
                )])
                .with_body(br#"{"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"22.00"}],"isError":false}}"#)
                .build()
        );

    assert_eq!(
        block_on(
            MagicSum {}.handle(
                &HttpRequest::builder()
                    .with_method(Method::POST)
                    .with_url("/mcp")
                    .with_body(
                        br#"
                            {
                            "jsonrpc": "2.0",
                            "id": 1,
                            "method": "tools/call",
                            "params": {
                                "name": "unknown_tool",
                                "arguments": {}
                            }
                        }    
                "#
                    )
                    .build(),
                |_| true
            )
        ),
        HttpResponse::builder()
            .with_status_code(StatusCode::from_u16(200).unwrap())
            .with_headers(vec![(
                "Content-Type".to_string(),
                "application/json".to_string()
            )])
            .with_body(
                br#"{"jsonrpc":"2.0","id":1,"error":{"code":-32602,"message":"not found tool"}}"#
            )
            .build()
    );
}

#[test]
fn test_unsupported_method() {
    assert_eq!(
        block_on(
            MagicSum {}.handle(
                &HttpRequest::builder()
                    .with_method(Method::POST)
                    .with_url("/mcp")
                    .with_body(
                        br#"
                            {
                            "jsonrpc": "2.0",
                            "id": 1,
                            "method": "resources/list",
                            "params": {}
                        }    
                "#
                    )
                    .build(),
                |_| true
            )
        ),
        HttpResponse::builder()
            .with_status_code(StatusCode::from_u16(200).unwrap())
            .with_headers(vec![(
                "Content-Type".to_string(),
                "application/json".to_string()
            )])
            .with_body(
                br#"{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"Method not found"}}"#
            )
            .build()
    );
}

#[test]
fn test_batch() {
    assert_eq!(
        block_on(
            MagicSum {}.handle(
                &HttpRequest::builder()
                    .with_method(Method::POST)
                    .with_url("/mcp")
                    .with_body(
                        br#"[
                            {
                            "jsonrpc": "2.0",
                            "id": 1,
                            "method": "prompts/list",
                            "params": {}
                        } ,{
                            "jsonrpc": "2.0",
                            "id": "123",
                            "method": "ping"
                            }
                        ]   
                "#
                    )
                    .build(), |_| true
            )
        ),
        HttpResponse::builder()
            .with_status_code(StatusCode::from_u16(200).unwrap())
            .with_headers(vec![(
                "Content-Type".to_string(),
                "application/json".to_string()
            )])
            .with_body(
                br#"[{"error":{"code":-32601,"message":"Method not found"},"id":1,"jsonrpc":"2.0"},{"id":"123","jsonrpc":"2.0","result":{}}]"#
            )
            .build()
    );
}
