# ic-rmcp

[![Internet Computer compatible](https://img.shields.io/badge/IC-compatible-blue.svg)](https://internetcomputer.org)
[![MCP Version](https://img.shields.io/badge/MCP%20Spec-2025--03--26;%202025--06--18-orange.svg)](https://modelcontextprotocol.io)

A lightweight Rust SDK for implementing **Model Context Protocol (MCP)** servers on the **Internet Computer**.

This SDK is specifically designed for the IC canister runtime, using the Streamable HTTP transport and focusing on the core `tools` capability. It allows developers to quickly expose canister functions as MCP tools for AI models to interact with.

## Features

- **Protocol Version**: Implements the `2025-03-26` & `2025-06-18` MCP specification versions.
- **Target Runtime**: Built exclusively for the Internet Computer (no `tokio` dependency).
- **Transport**: Supports the official **Streamable HTTP** transport.
- **Capabilities**:
    - ✅ `tools` (`tools/list`, `tools/call`)
- **Utilities**:
    - ✅ `ping`
- **Oauth**: Integrate with OAuth providers for secure tool access (see [Clock MCP example](./examples/clock/)).

## Limitations

- **Stateless**: No maintained sessions. Also no two-way communication between server and client. You should be aware of HTTP response size limitation on IC environment when designing and implementing tools.
- Your api key can seen by node in subnet
- Limited by the request/received size - 2MB
- HTTP outcall limited with IPv6

## Usage

### 1. Add to `Cargo.toml`

```toml
[dependencies]
ic-rmcp = { git = "https://github.com/ByteSmithLabs/ic-rmcp", tag = "v0.3.0" }

```

### 2. Implement the `Handler` Trait

Create a struct for your server and implement the `ic_rmcp::Handler` trait. This is where you define your server's logic for listing and calling tools.

The SDK provides default implementations, so you only need to override the methods you want to support.

```rust
use ic_rmcp::{model::*, schema_for_type, Error, Handler, Server, Context};


struct MyServer;

impl Handler for MyServer {
   fn get_info(&self, _: Context) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "My MCP server".to_string(),
                version: "1.0.0".to_string(),
            },
            ..Default::default()
        }
    }

   async fn list_tools(&self,_: Context, _: Option<PaginatedRequestParam>) -> Result<ListToolsResult, Error> {
        Ok(ListToolsResult {
            next_cursor: None,
            tools: vec![
                Tool::new(
                    "foo",
                    "A foo tool",
                    schema_for_type::<EmptyObject>(),
                )
            ],
        })
    }

   async fn call_tool(&self,_: Context, requests: CallToolRequestParam) -> Result<CallToolResult, Error> {
        match requests.name.as_ref() {
            "foo" => {
                Ok(CallToolResult::success(
                    Content::text("Call foo tool successfully").into_contents(),
                ))
            }
            _ => Err(Error::invalid_params("not found tool", None)),
        }
    }
}
```

### 3. Expose the Server in Your Canister

Use the standard `http_request` and `http_request_update` canister endpoints. The `Server` trait is automatically implemented on your `Handler`, giving you access to the appropriate handle method. See more at **[HTTP Gateway on Internet Computer](https://internetcomputer.org/docs/building-apps/network-features/using-http/gateways)**

```rust
use ic_cdk::{init, query, update};
use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};

// A constant for a simple API key auth
const API_KEY: &str = "a-secret-api-key";

// Tool results are dynamic. Hence need subnet concensus.
#[query]
fn http_request(_: HttpRequest) -> HttpResponse {
    HttpResponse::builder()
        .with_status_code(StatusCode::OK)
        .with_upgrade(true)
        .build()
}

#[update]
async fn http_request_update(req: HttpRequest<'_>) -> HttpResponse<'_> {
    MyServer {}
        .handle(&req, |headers| -> bool {
            headers
                .iter()
                .any(|(k, v)| k == "x-api-key" && *v == API_KEY.with_borrow(|k| k.clone()))
        })
        .await
}
```
> **About OAuth**: See our [Clock MCP server example](./examples/clock/) to learn about how to set up your MCP server with OAuth. 

### 4. Deploy your canister
Access your MCP server after deployment at: `https://<CANISTER_ID>.icp0.io/mcp`

## Full Canister Example
- See [examples](./examples/).
- Other advanced [examples](https://github.com/ByteSmithLabs/mcp-examples) 

### 5. Learning resources
[ByteSmithLabs YouTube Channel](https://www.youtube.com/@ByteSmithLabs)

## Related Resources

- **[Model Context Protocol Specification](https://modelcontextprotocol.io)**
- **[MCP Schema](https://github.com/modelcontextprotocol/specification/blob/main/schema/2025-03-26/schema.ts)**
- **[Use MCP servers in VS Code](https://code.visualstudio.com/docs/copilot/chat/mcp-servers)**
- **[MCP Inspector](https://github.com/modelcontextprotocol/inspector)**
- **[Prometheus Protocol](https://github.com/prometheus-protocol)**