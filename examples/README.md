**Note**: Examples reflect latest version of the source code. It may use unstable APIs. Checkout the exact revision for the version you're looking for. 

## Deployment:

```bash
dfx deploy <server_name> --argument '("YOUR_API_KEY")' --mode install
```

or run the bash file

```bash
bash deploy_server.sh adder
bash deploy_server.sh counter
```

After deployment on local network or playground, you can access it at: `https://<CANISTER_ID>.icp0.io/mcp` (for playground) or `https://<CANISTER_ID>.localhost:<BINDING_PORT>/mcp` (for local).

You could directly use our trait handler to construct HTTP endpoint (see [counter](./counter/)) or use [`ic-http`](https://github.com/ByteSmithLabs/ic-http) (see [adder](./adder/)).

Youtube code explain
- [Start building MCP server on Internet Computer](https://youtu.be/vFD2eKgNO0A)
- [MCP server with Oauth](https://youtu.be/YWJ99deCowQ)