mod error;
pub use error::Error;

pub mod handler;
pub mod model;
pub mod service;

#[cfg(test)]
mod tests {
    use crate::service::ServiceExt;

    use super::handler::ServerHandler;
    use super::model::*;
    use futures::executor::block_on;
    use ic_http_certification::{HttpRequest, Method};
    use serde_json::{from_slice, json, Value};

    struct Adder;

    impl ServerHandler for Adder {
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
                        br#"{
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
}"#,
                    )
                    .build(),
            ),
        );

        assert_eq!(
            json!({"jsonrpc":"2.0","id":4,"result":{"protocolVersion":"2025-03-26","capabilities":{},"serverInfo":{"name":"Adder MCP","version":"1.0.0"}}}),
            from_slice::<Value>(result.body()).unwrap()
        );
    }
}
