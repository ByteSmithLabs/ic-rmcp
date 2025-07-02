This SDK is intended for supporting MCP server development on [Internet Computer](https://internetcomputer.org) canisters. For normal platforms, check out [official SDK](https://github.com/modelcontextprotocol/rust-sdk). Compatible with
any client support Streamable HTTP transport.

## Usage
### 1. Cargo.toml
```toml
[dependencies]
ic-rmcp = { git = "https://github.com/ByteSmithLabs/ic-rmcp", branch = "main" }
```
### 2. Server
See [examples](examples).
### 3. Example client
See [Use MCP servers in VS Code](https://code.visualstudio.com/docs/copilot/chat/mcp-servers) for quickstart.

## Features
This SDK supports version `2025-03-26`:
-  `tools` capability. 
- `ping` utility.
- Batching message.

### Auth
This SDK doesn't support OAuth authentication by the official specs. Rather, it provides a simpler, versatile authentication mechanism by HTTP headers. See [examples](examples) for more details. 
### Transport:
MCP Servers built by the SDK are stateless. No maintained sessions. Also no two-way communication between server and client. You should be aware of HTTP response size limitation on IC environment when designing and implementing tools.
