This SDK is intended for supporting MCP server development on [Internet Computer](https://internetcomputer.org) canisters. For normal platforms, check out [official SDK](https://github.com/modelcontextprotocol/rust-sdk).

## Usage
### 1. Cargo.toml
```toml
[dependencies]
ic_rmcp = { git = "https://github.com/ByteSmithLabs/ic-rmcp", branch = "main" }
```
### 2. Server
See [adder](examples/adder).
### 3. Client
See [Use MCP servers in VS Code](https://code.visualstudio.com/docs/copilot/chat/mcp-servers).