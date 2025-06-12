This SDK is intended for supporting MCP server development on [Internet Computer](https://internetcomputer.org) canisters. For normal platforms, check out [official SDK](https://github.com/modelcontextprotocol/rust-sdk). Compatible with
any client support Streamable HTTP transport. MCP Servers built by the SDK are stateless.

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
This SDK supports:
-  `tools` capability. 
- `ping` utility.
- Batching message.

