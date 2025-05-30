//! This SDK is intended for supporting MCP server development on [Internet Computer](https://internetcomputer.org) canisters. For normal platforms, check out [official SDK](https://github.com/modelcontextprotocol/rust-sdk).

//! ```rust
//! use ic_cdk_macros::{query, update};
//! use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
//! use ic_rmcp::{Handler, Server};
//! use rmcp::{handler::server::tool::schema_for_type, model::*, Error};
//! use ic_cdk::api::time;
//!
//! #[query]
//! fn http_request(_: HttpRequest) -> HttpResponse {
//!     HttpResponse::builder()
//!         .with_status_code(StatusCode::OK)
//!         .with_upgrade(true)
//!         .build()
//! }
//!
//! struct Clock;
//!
//! impl Handler for Clock {
//!     fn get_info(&self) -> ServerInfo {
//!         ServerInfo {
//!             capabilities: ServerCapabilities::builder().enable_tools().build(),
//!             server_info: Implementation {
//!                 name: "Clock".to_string(),
//!                 version: "1.0.0".to_string(),
//!             },
//!             ..Default::default()
//!         }
//!     }
//!
//!     async fn list_tools(&self, _: Option<PaginatedRequestParam>) -> Result<ListToolsResult, Error> {
//!         Ok(ListToolsResult {
//!             next_cursor: None,
//!             tools: vec![
//!                 Tool::new(
//!                     "get_time",
//!                     "Get the current timestamp in nanoseconds.",
//!                     schema_for_type::<EmptyObject>(),
//!                 ),
//!             ],
//!         })
//!     }
//!
//!     async fn call_tool(&self, requests: CallToolRequestParam) -> Result<CallToolResult, Error> {
//!         match requests.name.as_ref() {
//!             "get_time" => {
//!                 Ok(CallToolResult::success(
//!                     Content::text(format!("{}", time())).into_contents(),
//!                 ))
//!             },
//!             _ => Err(Error::invalid_params("not found tool", None)),
//!         }
//!     }
//! }
//!
//! #[update]
//! async fn http_request_update(req: HttpRequest<'_>) -> HttpResponse<'_> {
//!     Clock {}.handle(req).await
//! }
//!
//! ic_cdk::export_candid!();
//! ```
mod handler;
pub use handler::Handler;

mod server;
pub use server::Server;

#[cfg(test)]
mod tests {
    use crate::server::Server;

    use super::handler::Handler;
    use futures::executor::block_on;
    use ic_http_certification::{HttpRequest, Method};
    use rmcp::model::*;
    use serde_json::{from_slice, json, Value};

    struct Adder;

    impl Handler for Adder {
        fn get_info(&self) -> ServerInfo {
            ServerInfo {
                protocol_version: ProtocolVersion::default(),
                capabilities: ServerCapabilities::default(),
                server_info: Implementation {
                    name: "Adder MCP".to_string(),
                    version: "1.0.0".to_string(),
                },
                instructions: None,
            }
        }
    }

    #[test]
    fn test_adder_mcp() {
        let result = block_on(
            Adder {}.handle(
                HttpRequest::builder()
                    .with_method(Method::POST)
                    .with_url("/mcp")
                    .with_body(
                        br#"[{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "initialize",
  "params": {
    "protocolVersion": "2025-03-26",
    "capabilities": {},
    "clientInfo": {
      "name": "ExampleClient",
      "version": "1.0.0"
    }
  }
},
{
  "jsonrpc": "2.0",
  "method": "notifications/initialized"
},
{
  "jsonrpc": "2.0",
  "id": "123",
  "method": "ping"
}]"#,
                    )
                    .build(),
            ),
        );

        assert_eq!(
            json!([{"jsonrpc":"2.0","id":4,"result":{"protocolVersion":"2025-03-26","capabilities":{},"serverInfo":{"name":"Adder MCP","version":"1.0.0"}}},{
              "jsonrpc": "2.0",
              "id": "123",
              "result": {}
            }]),
            from_slice::<Value>(result.body()).unwrap()
        );
    }
}
